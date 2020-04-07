use std::path::PathBuf;
use structopt::StructOpt;

extern crate aead;
extern crate daemonize;
extern crate ethabi;
extern crate hex;
extern crate job_scheduler;
extern crate parity_crypto;
extern crate secp256k1;
extern crate uint;
extern crate web3;

mod errors;
mod key;
mod receive;
mod scan;
mod transfer;
mod utils;

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
        to: String,
        /// Value to be transferred
        /// (in wei)
        #[structopt(short = "v")]
        value: String
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
        /// Directory in which master
        /// keypair file is saved
        #[structopt(short = "s", parse(from_os_str))]
        storage_dir: PathBuf,
        /// Master key address
        #[structopt(short = "a")]
        address: String,
        /// Block number to
        /// scan from
        #[structopt(short = "b")]
        block: Option<u64>
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
        Cli::Transfer { mut storage_dir, from, to, value } => {
            println!("Handle Transfer [dir] = {:?}, [from] = {}, [to] = {}, value = {}", storage_dir, from, to, value);
            match transfer::transfer(&mut storage_dir, &from, &to, &value) {
                Ok(transfer_receipt) => {
                    println!("Successfully transferred");
                    println!("Transfer tx hash: {:?}", transfer_receipt.tx1_hash);
                    println!("Nonce broadcasted tx hash: {:?}", transfer_receipt.tx2_hash);
                    println!("Share this nonce point with recipient: {:x}", transfer_receipt.nonce_point_compressed);
                    println!("nonce point (x) = {:?}", transfer_receipt.nonce_point_x);
                    println!("nonce point (y) = {:?}", transfer_receipt.nonce_point_y);
                },
                Err(error) => panic!("[Error in transfer]: {:?}", error)
            }
        },
        Cli::Receive { mut storage_dir, address, nonce_point } => {
            println!("Handle receive [dir] = {:?}, [master] = {}, [nonce point] = {}", storage_dir, address, nonce_point);
            match receive::receive(&mut storage_dir, &address, &nonce_point) {
                Ok(receipt) => {
                    println!("Successfully claimed receipt");
                    println!("Recipient address: {:?}", receipt.address);
                    println!("Recipient balance: {:?}", receipt.balance);
                },
                Err(error) => panic!("[Error in receiving]: {:?}", error)
            }
        },
        Cli::Scan { mut storage_dir, address, block } => {
            println!("Handle Scan");
            scan::scan(&mut storage_dir, &address, block);
        }
    }
}
