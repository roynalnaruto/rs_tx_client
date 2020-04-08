use std::convert::TryFrom;

use hex::FromHex;
use parity_crypto::publickey::Secret;
use web3::types::{H160, H256};

use crate::errors::Error;

pub fn convert_h160(h: primitive_types::H160) -> H160 {
    let bytes = h.as_bytes();
    H160::from_slice(&bytes)
}

pub fn convert_h256(h: &Secret) -> H256 {
    let bytes = h.as_bytes();
    H256::from_slice(&bytes)
}

pub fn convert_u64_i64(v: u64) -> Option<i64> {
    i64::try_from(v).ok()
}

pub fn convert_str_h256(v: &str) -> Result<H256, Error> {
    let u = &v[2..66];
    let u_hex = <[u8; 32]>::from_hex(u)?;
    Ok(H256::from(u_hex))
}
