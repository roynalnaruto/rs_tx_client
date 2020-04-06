use std::fs::File;
use std::io::{BufReader, Write};
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use parity_crypto::publickey::{Address, Generator, KeyPair, Public, Random, Secret};
use parity_crypto::publickey::public_to_address;

use secp256k1::PublicKey;

use crate::errors::Error;

pub fn new(mut path: &mut PathBuf) -> Result<(), Error> {
    // generate random keypair
    let keypair = Random.generate();

    // store keypair
    store(&mut path, &keypair);

    Ok(())
}

pub fn store(path: &mut PathBuf, keypair: &KeyPair) -> Result<(), Error> {
    // create file to write keypair data
    let filename = format!("{:?}.json", keypair.address());
    path.push(filename);
    let mut file = File::create(path)?;
    let public_key = keypair.public();
    let secp_public_key = to_secp256k1_public(&public_key).unwrap();

    // content of the key file
    let content = format!(
        "Secret Key: {:x}\nPublic Key (Uncompressed): {:x}\nPublic Key (Compressed): {}\nAddress: {:?}",
        keypair.secret(), keypair.public(), secp_public_key.to_string(), keypair.address()
    );
    write!(file, "{}", content)?;

    Ok(())
}

pub fn load(path: &mut PathBuf, address: &str) -> Result<KeyPair, Error> {
    // get filepath of stored keypair
    let filename = format!("{:?}.json", address);
    path.push(filename);

    // open file and read the first line (secret key)
    let file = File::open(path)?;
    let mut buffer = BufReader::new(file);
    let mut secret_key_line = String::new();
    buffer.read_line(&mut secret_key_line).expect("Unable to read line");
    let secret_key_str = str::replace(&secret_key_line, "Secret Key: ", "");

    // form keyapair
    let secret_key = Secret::from_str(&secret_key_str).unwrap();
    let keypair = KeyPair::from_secret(secret_key).unwrap();

    Ok(keypair)
}

pub fn public_key_from_str(pk: &str) -> Result<(Public, Address), Error> {
    let secp_public_key = PublicKey::from_str(pk)?;
    let mut public_key = Public::default();
    set_public(&mut public_key, &secp_public_key);

    let secp_public_key = PublicKey::from_str(pk)?;
    let mut public_key = Public::default();
    set_public(&mut public_key, &secp_public_key);
    let address = public_to_address(&public_key);

    Ok((public_key, address))
}

pub fn to_secp256k1_public(public: &Public) -> Result<PublicKey, Error> {
	let public_data = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(&public[0..64]);
		temp
	};

    let secp_public_key = PublicKey::from_slice(&public_data)?;

    Ok(secp_public_key)
}

pub fn set_public(public: &mut Public, key_public: &PublicKey) {
	let key_public_serialized = key_public.serialize_uncompressed();
	public.as_bytes_mut().copy_from_slice(&key_public_serialized[1..65]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_from_str() {
        let pk_valid = public_key_from_str("029a86a8531249e6bf23b0b1792e5983d5eabf3389977423fe4c9d82fb27b347e9").unwrap();
        let pk_invalid = public_key_from_str("029a86a8531249e6bf23b0b1792e5983d5eabf3389977423fe4c9d82fb27b347e");
        assert!(format!("{:x}", pk_valid) == "9a86a8531249e6bf23b0b1792e5983d5eabf3389977423fe4c9d82fb27b347e9fd2167dded063e20e461e7cf417f39174da9212a6c3f533e04b5df59075c3866");
        assert!(pk_invalid == Err(secp256k1::Error::InvalidPublicKey));
    }
}
