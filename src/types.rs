#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec, vec::Vec};

#[cfg(not(feature = "contract"))]
use sha3::{Digest, Keccak256};

use borsh::{BorshDeserialize, BorshSerialize};
use primitive_types::{H160, H256, U256};

use evm::backend::Log;

#[cfg(feature = "contract")]
use crate::sdk;

pub type RawAddress = [u8; 20];
pub type RawU256 = [u8; 32];
pub type RawH256 = [u8; 32];

/// Borsh-encoded parameters for the `call` function.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct FunctionCallArgs {
    pub contract: RawAddress,
    pub input: Vec<u8>,
}

/// Borsh-encoded parameters for the `view` function.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ViewCallArgs {
    pub sender: RawAddress,
    pub address: RawAddress,
    pub amount: RawU256,
    pub input: Vec<u8>,
}

/// Borsh-encoded parameters for the `get_storage_at` function.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct GetStorageAtArgs {
    pub address: RawAddress,
    pub key: RawH256,
}

/// Borsh-encoded parameters for the `begin_chain` function.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct BeginChainArgs {
    pub chain_id: RawU256,
}

/// Borsh-encoded parameters for the `begin_block` function.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct BeginBlockArgs {
    /// The current block's hash (for replayer use).
    pub hash: RawU256,
    /// The current block's beneficiary address.
    pub coinbase: RawU256,
    /// The current block's timestamp (in seconds since the Unix epoch).
    pub timestamp: RawU256,
    /// The current block's number (the genesis block is number zero).
    pub number: RawU256,
    /// The current block's difficulty.
    pub difficulty: RawU256,
    /// The current block's gas limit.
    pub gaslimit: RawU256,
}

pub enum KeyPrefix {
    Config = 0x0,
    Nonce = 0x1,
    Balance = 0x2,
    Code = 0x3,
    Storage = 0x4,
}

pub fn address_to_key(prefix: KeyPrefix, address: &H160) -> [u8; 21] {
    let mut result = [0u8; 21];
    result[0] = prefix as u8;
    result[1..].copy_from_slice(&address.0);
    result
}

pub fn storage_to_key(address: &H160, key: &H256) -> [u8; 53] {
    let mut result = [0u8; 53];
    result[0] = KeyPrefix::Storage as u8;
    result[1..21].copy_from_slice(&address.0);
    result[21..].copy_from_slice(&key.0);
    result
}

pub fn u256_to_arr(value: &U256) -> [u8; 32] {
    let mut result = [0u8; 32];
    value.to_big_endian(&mut result);
    result
}

pub fn log_to_bytes(log: Log) -> Vec<u8> {
    let mut result = vec![0u8; 1 + log.topics.len() * 32 + log.data.len()];
    result[0] = log.topics.len() as u8;
    let mut index = 1;
    for topic in log.topics.iter() {
        result[index..index + 32].copy_from_slice(&topic.0);
        index += 32;
    }
    result[index..].copy_from_slice(&log.data);
    result
}

const HEX_ALPHABET: &[u8; 16] = b"0123456789abcdef";

pub fn bytes_to_hex(v: &[u8]) -> String {
    let mut result = String::new();
    for x in v {
        result.push(HEX_ALPHABET[(x / 16) as usize] as char);
        result.push(HEX_ALPHABET[(x % 16) as usize] as char);
    }
    result
}

#[cfg(feature = "contract")]
#[inline]
pub fn keccak(data: &[u8]) -> H256 {
    sdk::keccak(data)
}

#[cfg(not(feature = "contract"))]
#[inline]
pub fn keccak(data: &[u8]) -> H256 {
    H256::from_slice(Keccak256::digest(data).as_slice())
}

pub fn near_account_to_evm_address(addr: &[u8]) -> H160 {
    H160::from_slice(&keccak(addr)[12..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex() {
        assert_eq!(
            bytes_to_hex(&[0u8, 1u8, 255u8, 16u8]),
            "0001ff10".to_string()
        );
    }
}
