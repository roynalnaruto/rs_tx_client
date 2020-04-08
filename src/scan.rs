use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use std::env;

use aes_gcm::Aes256Gcm;
use aead::{Aead, NewAead, generic_array::GenericArray};

use job_scheduler::{JobScheduler, Job};

use daemonize::Daemonize;

use parity_crypto::Keccak256;
use parity_crypto::publickey::ec_math_utils;
use parity_crypto::publickey::KeyPair;

use web3::futures::Future;
use web3::types::{H256, TransactionId::Hash, U64};

use crate::errors::Error;
use crate::key;
use crate::query;
use crate::query::RsTxTransaction;
use crate::receive;
use crate::utils::{convert_u64_i64, convert_str_h256};

pub fn scan(
    storage_dir: &PathBuf,
    master_address: &str,
    from_block: Option<u64>
) -> Result<(), Error> {
    // instantiate web3
    let (_eloop, transport) = web3::transports::Http::new("http://127.0.0.1:8545").unwrap();
    let web3 = web3::Web3::new(transport);
    let latest_block = web3.eth().block_number().wait().unwrap();

    let block_number = match from_block {
        Some(b) => U64::from(b),
        None => latest_block - U64::from(2)
    };

    // create log files
    let cwd = env::current_dir()?;
    let mut stdout_path = cwd.clone();
    stdout_path.push(".logs/scan.out");
    let mut stderr_path = cwd.clone();
    stderr_path.push(".logs/scan.err");
    let stdout = File::create(stdout_path).unwrap();
    let stderr = File::create(stderr_path).unwrap();

    // daemonize the scan process
    let daemonize = Daemonize::new()
        .pid_file("/tmp/rs_tx_scan.pid")
        .chown_pid_file(true)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            let master_keypair = key::load(&storage_dir, &master_address)?;
            _scan(&storage_dir, &master_keypair, block_number);

            Ok(())
        },
        Err(e) => Err(Error::Daemonize(e))
    }
}

fn _scan(
    storage_dir: &PathBuf,
    keypair: &KeyPair,
    mut block_number: U64
) {
    let mut sched = JobScheduler::new();
    sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
        match convert_u64_i64(block_number.as_u64()) {
            Some(b) => {
                let txs = query::query(b);

                let my_txs: Vec<RsTxTransaction> = txs
                    .iter()
                    .filter_map(|tx| is_my_tx(tx, &keypair).ok())
                    .collect();

                for tx in my_txs {
                    let own_address = format!("{:?}", keypair.address());
                    match receive::receive(
                        &storage_dir,
                        &own_address,
                        &tx.nonce_point.as_str()
                    ) {
                        Ok(receipt) => {
                            println!("Successfully claimed receipt");
                            println!("Recipient address: {:?}", receipt.address);
                            println!("Recipient balance: {:?}", receipt.balance);
                        },
                        Err(e) => eprintln!("error = {:?}", e)
                    }
                }

                // increment the block_number to
                // the block number of the last tx
                if let Some(last_tx) = txs.last() {
                    block_number = last_tx.block;
                }
            },
            None => eprintln!("[scan] Error converting block number from u64 to i64")
        }
    }));
    loop {
        sched.tick();
        std::thread::sleep(Duration::from_secs(5));
    }
}

fn is_my_tx(
    tx: &RsTxTransaction,
    keypair: &KeyPair
) -> Result<RsTxTransaction, Error> {
    // check if the encrypted recipient when decrypted is actually
    // the address of your own keypair. if not, return Err
    let (nonce_point, _) = key::public_key_from_str(&tx.nonce_point)?;
    let mut ecdh_shared_secret = nonce_point.clone();
    ec_math_utils::public_mul_secret(&mut ecdh_shared_secret, &keypair.secret())?;
    let shared_secret = ecdh_shared_secret.keccak256();
    let key = GenericArray::clone_from_slice(&shared_secret);
    let aead = Aes256Gcm::new(key);

    // instantiate web3
    let (_eloop, transport) = web3::transports::Http::new("http://127.0.0.1:8545").unwrap();
    let web3 = web3::Web3::new(transport);

    // get tx nonce for calculating encryption nonce
    let tx_hash = convert_str_h256(&tx.id)?;
    let web3_tx = web3.eth()
        .transaction(Hash(H256::from(tx_hash)))
        .wait()
        .unwrap()
        .unwrap();

    // decrypt the recipient address
    let tx_nonce = web3_tx.nonce;
    let mut tx_nonce_slice = [0u8; 32];
    tx_nonce.to_big_endian(&mut tx_nonce_slice);
    let popped_tx_nonce = {
		let mut temp = [0u8; 12];
		(&mut temp[0..12]).copy_from_slice(&tx_nonce_slice[0..12]);
		temp
	};
    let encryption_nonce = GenericArray::from_slice(&popped_tx_nonce);
    let decrypted_recipient = aead.decrypt(encryption_nonce, tx.encrypted_recipient.as_ref())?;

    // if intended address matches own address
    // include this tx to be received
    let intended_recipient_address = hex::encode(decrypted_recipient);
    let own_address = format!("{:x}", keypair.address());

    if intended_recipient_address == own_address {
        let my_tx = tx.clone();
        Ok(my_tx)
    } else {
        Err(Error::Custom(String::from("[Dummy error] None of the tx were yours")))
    }
}
