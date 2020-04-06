use std::path::PathBuf;
use structopt::StructOpt;

extern crate hex;
extern crate parity_crypto;
extern crate secp256k1;
extern crate web3;

mod key;
mod transfer;
mod receive;
mod errors;

/// RsTx client for Stealth Addresses in Ethereum
#[derive(StructOpt, Debug)]
#[structopt(name = "RsTx Client")]
enum Cli {
    /// Create a new keypair
    #[structopt(name = "create")]
    Create {
        /// Directory to store
        /// the created keypair
        #[structopt(short = "s", parse(from_os_str))]
        storage_dir: PathBuf
    },
    /// List all ECDH keypairs
    #[structopt(name = "list")]
    List {
        /// Directory from which
        /// to list keypairs
        #[structopt(short = "s", parse(from_os_str))]
        storage_dir: PathBuf
    },
    /// Transfer ether
    #[structopt(name = "transfer")]
    Transfer {
        /// Directory in which
        /// keypair file is saved
        #[structopt(short = "s", parse(from_os_str))]
        storage_dir: PathBuf,
        /// Sender address, will be loaded
        /// from the key in storage_dir
        #[structopt(short = "f")]
        from: String,
        /// Recipient public key
        /// in compressed form
        #[structopt(short = "t")]
        to: String
    },
    /// Receive ether
    #[structopt(name = "receive")]
    Receive {
        /// Directory in which master
        /// keypair file is saved
        #[structopt(short = "s", parse(from_os_str))]
        storage_dir: PathBuf,
        /// Master key address
        #[structopt(short = "a")]
        address: String,
        /// Nonce point (in compressed form)
        /// of the stealth transaction
        #[structopt(short = "n")]
        nonce_point: String
    },
    /// Scan the blockchain
    /// for incoming txs
    #[structopt(name = "scan")]
    Scan {
        /// Block number to
        /// scan from
        #[structopt(short = "b")]
        block: Option<i32>
    }
}

fn main() {
    match Cli::from_args() {
        Cli::Create { mut storage_dir } => {
            println!("Handle Create {:?}", storage_dir);
            if let Err(error) = key::new(&mut storage_dir) {
                panic!("[Error in creating/storing keypair]: {:?}", error)
            }
        },
        Cli::List { storage_dir } => println!("Handle List {:?}", storage_dir),
        Cli::Transfer { mut storage_dir, from, to } => {
            println!("Handle Transfer [dir] = {:?}, [from] = {}, [to] = {}", storage_dir, from, to);
            transfer::transfer(&mut storage_dir, &from, &to);
        },
        Cli::Receive { mut storage_dir, address, nonce_point } => {
            println!("Handle receive [dir] = {:?}, [master] = {}, [nonce point] = {}", storage_dir, address, nonce_point);
            receive::receive(&mut storage_dir, &address, &nonce_point);
        },
        Cli::Scan { block } => {
            if let Some(b) = block {
                println!("Handle Scan from block {:?}", b);
            } else {
                println!("Handle Scan from last block");
            }
        }
    }
}
