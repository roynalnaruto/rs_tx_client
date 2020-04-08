use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use std::env;

use job_scheduler::{JobScheduler, Job};

use daemonize::Daemonize;

use parity_crypto::publickey::KeyPair;

use web3::futures::Future;
use web3::transports::Http;
use web3::types::U64;

use crate::errors::Error;
use crate::key;
use crate::query;

pub fn scan(
    mut storage_dir: &mut PathBuf,
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
        .working_directory(cwd)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            let master_keypair = key::load(&mut storage_dir, &master_address)?;
            _scan(&master_keypair, block_number);

            Ok(())
        },
        Err(e) => Err(Error::Daemonize(e))
    }
}

fn _scan(keypair: &KeyPair, block_number: U64) {
    let mut sched = JobScheduler::new();
    sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
        query::query();
    }));
    loop {
        sched.tick();
        std::thread::sleep(Duration::from_secs(5));
    }
}
