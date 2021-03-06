use std::path::PathBuf;
use std::str::FromStr;

use parity_crypto::Keccak256;
use parity_crypto::publickey::ec_math_utils;
use parity_crypto::publickey::public_to_address;
use parity_crypto::publickey::{Address, KeyPair, Generator, Random, Secret};

use aes_gcm::Aes256Gcm;
use aead::{Aead, NewAead, generic_array::GenericArray};

use secp256k1::PublicKey;

use ethereum_tx_sign::RawTransaction;

use hex::FromHex;

use web3::Web3;
use web3::contract::tokens::Tokenize;
use web3::futures::Future;
use web3::transports::Http;
use web3::types::{Bytes, H160, H256, U256};

use crate::errors::Error;
use crate::key;
use crate::utils::{convert_h160, convert_h256};

static RS_TX_CONTRACT_ADDRESS: &'static str = "a3b67474A27Ba4bb28eE22e5f1C4529c07A45287";

pub struct Transfer {
    pub nonce_point: String,
    pub tx1_hash: H256,
    pub tx2_hash: H256,
}

pub fn transfer(
    from_path: &PathBuf,
    from_address: &str,
    to: &str,
    value: &str
) -> Result<Transfer, Error> {
    // instantiate web3
    let (_eloop, transport) = web3::transports::Http::new("http://127.0.0.1:8545").unwrap();
    let web3 = web3::Web3::new(transport);

    // parse recipient's public key and address
    // create a copy to use for EC math
    let (public_key, address) = key::public_key_from_str(to)?;

    // generate random nonce and calculate nonce point
    // nonce point is then shared with the recipient
    let nonce = Random.generate().secret().clone();
    let mut nonce_point = ec_math_utils::generation_point();
    ec_math_utils::public_mul_secret(&mut nonce_point, &nonce)?;
    let secp_nonce_point = key::to_secp256k1_public(&nonce_point)?;

    // generate ECDH shared secret
    // this secret can also be generated by Recipient
    // with the knowledge of the above `nonce_point`
    let mut ecdh_shared_secret = public_key.clone();
    ec_math_utils::public_mul_secret(&mut ecdh_shared_secret, &nonce)?;
    let ecdh_shared_secret_hash = ecdh_shared_secret.keccak256();

    // get the secret number from ecdh secret's hash
    // calculate recipient's address
    let secret_number = Secret::from_str(&hex::encode(ecdh_shared_secret_hash))?;
    let mut recipient_public_key = ec_math_utils::generation_point();
    ec_math_utils::public_mul_secret(&mut recipient_public_key, &secret_number)?;
    ec_math_utils::public_add(&mut recipient_public_key, &public_key)?;
    let recipient_address = public_to_address(&recipient_public_key);
    println!("recipient address = {:?}", recipient_address);

    // form signed transactions for
    // both Transfer and Broadcasting nonce
    let sender_keypair = key::load(&from_path, &from_address)?;
    let transfer_nonce = web3.eth().transaction_count(convert_h160(sender_keypair.address()), None).wait().unwrap();
    let broadcast_nonce = transfer_nonce + 1;
    let transfer_signed_tx = transfer_tx(&web3, &sender_keypair, transfer_nonce, &recipient_address, value)?;
    let broadcast_signed_tx = broadcast_tx(&web3, &sender_keypair, broadcast_nonce, &secp_nonce_point, &ecdh_shared_secret_hash, &address)?;

    // broadcast both transactions
    let transfer_tx_hash = web3.eth().send_raw_transaction(Bytes::from(transfer_signed_tx)).wait().unwrap();
    let broadcast_tx_hash = web3.eth().send_raw_transaction(Bytes::from(broadcast_signed_tx)).wait().unwrap();

    // return the Transfer object
    let transfer = Transfer {
        nonce_point: secp_nonce_point.to_string(),
        tx1_hash: transfer_tx_hash,
        tx2_hash: broadcast_tx_hash
    };

    Ok(transfer)
}

fn transfer_tx(
    web3: &Web3<Http>,
    from: &KeyPair,
    tx_nonce: U256,
    recipient_address: &Address,
    value: &str
) -> Result<Vec<u8>, Error> {
    // parse transfer value into U256
    let amount = U256::from_dec_str(value)?;

    // form raw transaction object
    let tx = RawTransaction {
        nonce: tx_nonce,
        to: Some(convert_h160(*recipient_address)),
        value: amount,
        gas_price: U256::from(1000000000),
        gas: U256::from(21000),
        data: Vec::new()
    };

    // sign transaction
    let chain_id = web3.eth().chain_id().wait().unwrap().as_u64();
    let signed_tx = tx.sign(&convert_h256(from.secret()), &chain_id);

    Ok(signed_tx)
}

fn broadcast_tx(
    web3: &Web3<Http>,
    from: &KeyPair,
    tx_nonce: U256,
    secp_nonce_point: &PublicKey,
    shared_secret: &[u8; 32],
    recipient_address: &Address
) -> Result<Vec<u8>, Error> {
    // form contract abi
    let json_abi: &[u8] = include_bytes!("contracts/RsTx.abi");
    let abi = ethabi::Contract::load(json_abi)?;

    // encrypt the recipient address
    let key = GenericArray::clone_from_slice(shared_secret);
    let aead = Aes256Gcm::new(key);
    let mut tx_nonce_slice = [0u8; 32];
    tx_nonce.to_big_endian(&mut tx_nonce_slice);
    let popped_tx_nonce = {
		let mut temp = [0u8; 12];
		(&mut temp[0..12]).copy_from_slice(&tx_nonce_slice[0..12]);
		temp
	};
    let encryption_nonce = GenericArray::from_slice(&popped_tx_nonce);
    let encrypted_recipient = aead.encrypt(encryption_nonce, recipient_address.as_ref())?;
    println!("encrypted recipient = {:?}", encrypted_recipient);

    // get params for transaction
    let nonce_point: Vec<u8> = secp_nonce_point.serialize().iter().cloned().collect();
    let params = (Bytes::from(nonce_point), Bytes::from(encrypted_recipient));
    let contract_address = <[u8; 20]>::from_hex(RS_TX_CONTRACT_ADDRESS)?;

    // encode function call params
    // sign tx and return the raw signed tx
    match abi.function("rsTx")
       .and_then(|function| function.encode_input(&params.into_tokens()))
       .and_then(|data| {
           let tx = RawTransaction {
               nonce: tx_nonce,
               to: Some(H160::from(contract_address)),
               value: U256::from(0),
               gas_price: U256::from(1000000000),
               gas: U256::from(1000000),
               data: data.into()
           };

           let chain_id = web3.eth().chain_id().wait().unwrap().as_u64();
           let signed_tx = tx.sign(&convert_h256(from.secret()), &chain_id);

           Ok(signed_tx)
       }) {
        Ok(signed_tx) => Ok(signed_tx),
        Err(e) => Err(Error::Ethabi(e))
    }
}
