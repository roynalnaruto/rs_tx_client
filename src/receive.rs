use std::path::PathBuf;
use std::str::FromStr;

use parity_crypto::Keccak256;
use parity_crypto::publickey::ec_math_utils;
use parity_crypto::publickey::{KeyPair, Secret};
use web3::types::{Address, U256};

use crate::errors::Error;
use crate::key;

pub struct Receipt {
    address: Address,
    balance: U256,
}

pub fn receive(
    mut master_path: &mut PathBuf,
    master_address: &str,
    nonce_point_str: &str
) -> Result<Receipt, Error> {
    // load master keypair
    let mut keys_path = master_path.clone();
    let master_keypair = key::load(&mut master_path, &master_address)?;
    let master_secret_key = master_keypair.secret().clone();

    // calculate the ecdh shared secret
    let (nonce_point, _) = key::public_key_from_str(nonce_point_str)?;
    let mut ecdh_shared_secret = nonce_point.clone();
    ec_math_utils::public_mul_secret(&mut ecdh_shared_secret, &master_secret_key)?;
    let ecdh_shared_secret_number = Secret::from_str(&hex::encode(ecdh_shared_secret.keccak256()))?;

    // calculate recipient secret key
    let mut recipient_secret_key = master_secret_key.clone();
    recipient_secret_key.add(&ecdh_shared_secret_number)?;
    let recipient_keypair = KeyPair::from_secret(recipient_secret_key)?;

    // store this key along with the master key
    key::store(&mut keys_path, &recipient_keypair)?;

    // query balance and form receipt
    todo!();

    Err(Error::Custom(String::from("[receive] dummy error")))
}
