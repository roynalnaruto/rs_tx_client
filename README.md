# RsTx
RsTx is a Rust client to support [Stealth Addresses](https://www.investopedia.com/terms/s/stealth-address-cryptocurrency.asp) for Ethereum

# Features
* Create a new master key-pair
* Transfer funds to a master public key
* Receive funds sent to a master public key

# Concept
RsTx implements stealth addresses based on the Diffie-Hellman key agreement protocol. The implementation follows the [mechanism described here](https://en.bitcoin.it/wiki/ECDH_address).

Both sender and recipient must have an already generated key-pair. The recipient's key-pair will act as a base (master) key, used to calculate the new one-time keys.

After a successful transfer, the sender must communicate the `Nonce point` (which is logged after a transfer) to the recipient. The recipient later uses this point to generate the key for the received funds. Without the nonce point, it is not possible to generate the new key.

A UX issue that needs to be addressed is the communication of nonce point. For the most simplistic implementation (as done in this code), we expect the sender to communicate the nonce point to the recipient off-chain. Future developments could be based around the sender also broadcasting this nonce point on-chain (to a smart contract), and the recipient's client scanning blockchain transactions to fetch and try if the generated key contains any funds.

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
