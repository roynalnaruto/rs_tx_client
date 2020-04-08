# RsTx
RsTx supports [Stealth Addresses](https://www.investopedia.com/terms/s/stealth-address-cryptocurrency.asp) for Ethereum

# Features
* Create a new master key-pair
* Transfer funds to a master public key
* Receive funds sent to a master public key
* Scan the blockchain for new transactions (and receive them)

# Concept
The RsTx architecture consists of the following repositories:
* [Rust Client](https://github.com/roynalnaruto/rs_tx_client) to implement the logic
* [Smart contracts](https://github.com/roynalnaruto/rs_tx_contracts) to communicate the nonce point
* [Subgraph](https://github.com/roynalnaruto/rs_tx_subgraph) to query Ethereum blockchain

RsTx implements stealth addresses based on the Diffie-Hellman key agreement protocol. The implementation follows the [mechanism described here](https://en.bitcoin.it/wiki/ECDH_address).

Both sender and recipient must have an already generated key-pair. The recipient's key-pair will act as a base (master) key, used to calculate the new one-time keys.

After a successful transfer, the sender must communicate the `Nonce point` (which is logged after a transfer) to the recipient. The recipient later uses this point to generate the key for the received funds. Without the nonce point, it is not possible to generate the new key.

Once a transfer has been made, the `nonce_point` is broadcasted to the [RsTx Smart Contract](https://github.com/roynalnaruto/rs_tx_contracts) along with a bytes encoded encrypted form of the recipient's address. The `encrypted_recipient` is used by the recipients in their client to catch or ignore the new RsTx transactions.

Senders can simply run the `scan` command, with an additional block number filter to query RsTx transactions. The GraphQL schema is generated using The Graph Protocol, and the subgraphs can be found here in the [RsTx Subgraph](https://github.com/roynalnaruto/rs_tx_subgraph) repository.

# Getting started
### Setup
* Clone this repository
```
git clone git@github.com:roynalnaruto/rs_tx_client.git
```
* Install dependencies
```
cargo build
```
* Get [Ganache](https://github.com/trufflesuite/ganache-cli) and run
```
npm install -g ganache-cli
ganache-cli -a=20
```
* Help related to the CLI
```
./target/debug/rs_tx_client --help
```
* Create a new key-pair
```
./target/debug/rs_tx_client create -s ~/path/to/keys/directory
```
* Transfer funds
```
./target/debug/rs_tx_client transfer -s ~/path/to/keys/directory -f <eth-address-to-send-from> -t <recipient-master-public-key> -v <eth-value-in-gwei>
```
* Receive funds
```
./target/debug/rs_tx_client receive -s ~/path/to/keys/directory -a <eth-address-of-master-key> -n <nonce-point-as-provided-by-sender>
```
* Scan Ethereum for new transactions
```
./target/debug/rs_tx_client scan -s ~/path/to/keys/directory -a <eth-address-of-master-key> -b <block-number-to-start-scan-from>
```

# Example
* Recipient generates key-pair
```
mkdir .keys
./target/debug/rs_tx_client create -s .keys/
```
* Recipient makes public their master public key (compressed form)
```
Secret Key: xxx
Public Key (Uncompressed): xxx
Public Key (Compressed): 03109b604bbe55ec2eefdb00828ba806dabedc0096d7f6857078e9365535b52812
Address: 0x20a291cdd831b721a7eef53f8b5a15817a2fced1
```
* Sender generates key-pair
```
./target/debug/rs_tx_client create -s .keys/

------------ generated ------------
Secret Key: xxx
Public Key (Uncompressed): xxx
Public Key (Compressed): xxx
Address: 0x123456cdd831b721a7eef53f8b5a15817a123456
-----------------------------------
```
* Sender transfers funds
```
./target/debug/rs_tx_client transfer -s .keys/ -f 0x123456cdd831b721a7eef53f8b5a15817a123456 -t 03109b604bbe55ec2eefdb00828ba806dabedc0096d7f6857078e9365535b52812 -v 1000000000000000000

-------------- log ---------------
Successfully transferred
Tx hash: xxx
Share this nonce point with recipient: 027dcf1ff03797edd8dbc46be97045e8f4fbf6629de08d73854058b8ebe669f3f9
nonce point (x) = xxx
nonce point (y) = xxx
----------------------------------
```
* Recipient generates new key-pair for the received funds
```
./target/debug/rs_tx_client receive -s .keys/ -a 0x20a291cdd831b721a7eef53f8b5a15817a2fced1 -n 027dcf1ff03797edd8dbc46be97045e8f4fbf6629de08d73854058b8ebe669f3f9

-------------- log ---------------
Successfully claimed receipt
Recipient address: 0xffa291cdd831b721a7eef53f8b5a15817a2fceff
Recipient balance: 1000000000000000000
----------------------------------
```
* Recipient can also simply scan the new transactions by running the `scan` sub-command (this runs as a daemon)
The `-b` flag specifies which Ethereum block to scan from. If not provided, the client scans from the current block
```
./target/debug/rs_tx_client scan -s .keys/ -a 0x20a291cdd831b721a7eef53f8b5a15817a2fced1 -b 100
```

# License
[In detail here](https://github.com/roynalnaruto/rs_tx_client/blob/master/LICENSE.md)
