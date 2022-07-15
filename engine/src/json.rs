use crate::prelude::{BTreeMap, String, Vec};

use crate::errors;
use core::convert::From;
use rjson::{Array, Null, Object, Value};

#[derive(PartialEq)]
pub enum JsonValue {
    Null,
    F64(f64),
    I64(i64),
    U64(u64),
    Bool(bool),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum JsonError {
    NotJsonType,
    MissingValue,
    InvalidU8,
    InvalidU64,
    InvalidU128,
    InvalidBool,
    InvalidString,
    InvalidArray,
    ExpectedStringGotNumber,
    OutOfRange(JsonOutOfRangeError),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum JsonOutOfRangeError {
    OutOfRangeU8,
    OutOfRangeU128,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ParseError {
    InvalidAccountId,
}

pub struct JsonArray(Vec<JsonValue>);
pub struct JsonObject(BTreeMap<String, JsonValue>);

impl JsonValue {
    #[allow(dead_code)]
    pub fn string(&self, key: &str) -> Result<String, JsonError> {
        match self {
            JsonValue::Object(o) => match o.get(key).ok_or(JsonError::MissingValue)? {
                JsonValue::String(s) => Ok(s.into()),
                _ => Err(JsonError::InvalidString),
            },
            _ => Err(JsonError::NotJsonType),
        }
    }

    #[allow(dead_code)]
    pub fn u64(&self, key: &str) -> Result<u64, JsonError> {
        match self {
            JsonValue::Object(o) => match o.get(key).ok_or(JsonError::MissingValue)? {
                JsonValue::U64(n) => Ok(*n),
                _ => Err(JsonError::InvalidU64),
            },
            _ => Err(JsonError::NotJsonType),
        }
    }

    #[allow(dead_code)]
    pub fn u128(&self, key: &str) -> Result<u128, JsonError> {
        match self {
            JsonValue::Object(o) => o.get(key).ok_or(JsonError::MissingValue)?.try_into(),
            _ => Err(JsonError::NotJsonType),
        }
    }

    #[allow(dead_code)]
    pub fn bool(&self, key: &str) -> Result<bool, JsonError> {
        match self {
            JsonValue::Object(o) => match o.get(key).ok_or(JsonError::MissingValue)? {
                JsonValue::Bool(n) => Ok(*n),
                _ => Err(JsonError::InvalidBool),
            },
            _ => Err(JsonError::NotJsonType),
        }
    }

    #[allow(dead_code)]
    pub fn parse_u8(v: &JsonValue) -> Result<u8, JsonError> {
        match v {
            JsonValue::U64(n) => {
                if *n > u8::MAX as u64 {
                    Err(JsonError::OutOfRange(JsonOutOfRangeError::OutOfRangeU8))
                } else {
                    Ok(*n as u8)
                }
            }
            _ => Err(JsonError::InvalidU8),
        }
    }
}

impl AsRef<[u8]> for JsonError {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::NotJsonType => errors::ERR_NOT_A_JSON_TYPE,
            Self::MissingValue => errors::ERR_JSON_MISSING_VALUE,
            Self::InvalidU8 => errors::ERR_FAILED_PARSE_U8,
            Self::InvalidU64 => errors::ERR_FAILED_PARSE_U64,
            Self::InvalidU128 => errors::ERR_FAILED_PARSE_U128,
            Self::InvalidBool => errors::ERR_FAILED_PARSE_BOOL,
            Self::InvalidString => errors::ERR_FAILED_PARSE_STRING,
            Self::InvalidArray => errors::ERR_FAILED_PARSE_ARRAY,
            Self::ExpectedStringGotNumber => errors::ERR_EXPECTED_STRING_GOT_NUMBER,
            Self::OutOfRange(err) => err.as_ref(),
        }
    }
}

impl AsRef<[u8]> for JsonOutOfRangeError {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::OutOfRangeU8 => errors::ERR_OUT_OF_RANGE_U8,
            Self::OutOfRangeU128 => errors::ERR_OUT_OF_RANGE_U128,
        }
    }
}

#[cfg(test)]
impl std::fmt::Debug for JsonError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            std::str::from_utf8(self.as_ref()).unwrap()
        ))
    }
}

#[cfg(test)]
impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", *self))
    }
}

impl Array<JsonValue, JsonObject, JsonValue> for JsonArray {
    fn new() -> Self {
        JsonArray(Vec::new())
    }
    fn push(&mut self, v: JsonValue) {
        self.0.push(v)
    }
}

impl Object<JsonValue, JsonArray, JsonValue> for JsonObject {
    fn new<'b>() -> Self {
        JsonObject(BTreeMap::new())
    }
    fn insert(&mut self, k: String, v: JsonValue) {
        self.0.insert(k, v);
    }
}

impl Null<JsonValue, JsonArray, JsonObject> for JsonValue {
    fn new() -> Self {
        JsonValue::Null
    }
}

impl Value<JsonArray, JsonObject, JsonValue> for JsonValue {}

impl From<f64> for JsonValue {
    fn from(v: f64) -> Self {
        JsonValue::F64(v)
    }
}

impl From<i64> for JsonValue {
    fn from(v: i64) -> Self {
        JsonValue::I64(v)
    }
}

impl From<u64> for JsonValue {
    fn from(v: u64) -> Self {
        JsonValue::U64(v)
    }
}

impl From<bool> for JsonValue {
    fn from(v: bool) -> Self {
        JsonValue::Bool(v)
    }
}

impl From<String> for JsonValue {
    fn from(v: String) -> Self {
        JsonValue::String(v)
    }
}

impl From<JsonArray> for JsonValue {
    fn from(v: JsonArray) -> Self {
        JsonValue::Array(v.0)
    }
}

impl From<JsonObject> for JsonValue {
    fn from(v: JsonObject) -> Self {
        JsonValue::Object(v.0)
    }
}

impl TryFrom<&JsonValue> for u128 {
    type Error = JsonError;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::String(n) => {
                if let Ok(x) = n.parse::<u128>() {
                    Ok(x)
                } else if n.parse::<i128>().is_ok() {
                    Err(JsonError::OutOfRange(JsonOutOfRangeError::OutOfRangeU128))
                } else {
                    Err(JsonError::InvalidU128)
                }
            }
            JsonValue::F64(_) => Err(JsonError::ExpectedStringGotNumber),
            JsonValue::I64(_) => Err(JsonError::ExpectedStringGotNumber),
            JsonValue::U64(_) => Err(JsonError::ExpectedStringGotNumber),
            _ => Err(JsonError::InvalidU128),
        }
    }
}

impl core::fmt::Debug for JsonValue {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            JsonValue::Null => f.write_str("null"),
            JsonValue::String(v) => f.write_fmt(format_args!("\"{}\"", v)),
            JsonValue::F64(v) => f.write_fmt(format_args!("{}", v)),
            JsonValue::I64(v) => f.write_fmt(format_args!("{}", v)),
            JsonValue::U64(v) => f.write_fmt(format_args!("{}", v)),
            JsonValue::Bool(v) => f.write_fmt(format_args!("{}", v)),
            JsonValue::Array(arr) => {
                f.write_str("[")?;
                let mut items = arr.iter();
                if let Some(item) = items.next() {
                    f.write_fmt(format_args!("{:?}", item))?;
                }
                for item in items {
                    f.write_fmt(format_args!(", {:?}", item))?;
                }
                f.write_str("]")
            }
            JsonValue::Object(kvs) => {
                f.write_str("{")?;
                let mut pairs = kvs.iter();
                if let Some((key, value)) = pairs.next() {
                    f.write_fmt(format_args!("\"{}\": {:?}", key, value))?;
                }
                for (key, value) in pairs {
                    f.write_fmt(format_args!(", \"{}\": {:?}", key, value))?;
                }
                f.write_str("}")
            }
        }
    }
}

impl core::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", *self))
    }
}

pub fn parse_json(data: &[u8]) -> Option<JsonValue> {
    let data_array: Vec<char> = data.iter().map(|b| *b as char).collect::<Vec<_>>();
    let mut index = 0;
    rjson::parse::<JsonValue, JsonArray, JsonObject, JsonValue>(&*data_array, &mut index)
}
