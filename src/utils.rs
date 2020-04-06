use parity_crypto::publickey::Secret;
use web3::types::{H160, H256};

pub fn convert_h160(h: primitive_types::H160) -> H160 {
    let bytes = h.as_bytes();
    H160::from_slice(&bytes)
}

pub fn convert_h256(h: &Secret) -> H256 {
    let bytes = h.as_bytes();
    H256::from_slice(&bytes)
}
