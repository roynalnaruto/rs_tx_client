use std::path::PathBuf;
use std::str::FromStr;

use parity_crypto::Keccak256;
use parity_crypto::publickey::ec_math_utils;
use parity_crypto::publickey::{KeyPair, Secret};

use web3::futures::Future;
use web3::types::{H160, U256};

use crate::errors::Error;
use crate::key;
use crate::utils::convert_h160;

pub struct Receipt {
    pub address: H160,
    pub balance: U256,
}

pub fn receive(
    mut master_path: &mut PathBuf,
    master_address: &str,
    nonce_point_str: &str
) -> Result<Receipt, Error> {
    // instantiate web3
    let (_eloop, transport) = web3::transports::Http::new("http://127.0.0.1:8545").unwrap();
    let web3 = web3::Web3::new(transport);

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
    let address = convert_h160(recipient_keypair.address());
    let balance = web3.eth().balance(address, None).wait().unwrap();
    let receipt = Receipt {
        address: address,
        balance: balance
    };

    Ok(receipt)
}
