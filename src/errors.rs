#[derive(Debug)]
pub enum Error {
    InvalidPublicKey,
    InvalidSecretKey,
    Secp(secp256k1::Error),
    ParityCrypto(parity_crypto::publickey::Error),
    Io(std::io::Error),
    Daemonize(daemonize::DaemonizeError),
    Ethabi(ethabi::Error),
    HexEncDecError(hex::FromHexError),
    Aead(aead::Error),
    UintParsing(uint::FromDecStrErr),
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

impl From<daemonize::DaemonizeError> for Error {
    fn from(e: daemonize::DaemonizeError) -> Error {
        Error::Daemonize(e)
    }
}

impl From<ethabi::Error> for Error {
    fn from(e: ethabi::Error) -> Error {
        Error::Ethabi(e)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Error {
        Error::HexEncDecError(e)
    }
}

impl From<aead::Error> for Error {
    fn from(e: aead::Error) -> Error {
        Error::Aead(e)
    }
}

impl From<uint::FromDecStrErr> for Error {
    fn from(e: uint::FromDecStrErr) -> Error {
        Error::UintParsing(e)
    }
}
