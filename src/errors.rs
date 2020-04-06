#[derive(Debug)]
pub enum Error {
    InvalidPublicKey,
    InvalidSecretKey,
    Secp(secp256k1::Error),
    ParityCrypto(parity_crypto::publickey::Error),
    Io(std::io::Error),
    Custom(String),
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Error {
        match e {
            secp256k1::Error::InvalidPublicKey => Error::InvalidPublicKey,
            secp256k1::Error::InvalidSecretKey => Error::InvalidSecretKey,
            _ => Error::Custom(String::from("Unmatched error from secp256k1")),
        }
    }
}

impl From<parity_crypto::publickey::Error> for Error {
    fn from(e: parity_crypto::publickey::Error) -> Error {
        match e {
            parity_crypto::publickey::Error::InvalidPublicKey => Error::InvalidPublicKey,
            parity_crypto::publickey::Error::InvalidSecretKey => Error::InvalidSecretKey,
            _ => Error::Custom(String::from("Unmatched error from parity_crypto"))
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}
