extern crate signer;
extern crate web3;
extern crate tokio_core;

use web3::futures::Future;
use signer::{PrivKey, KeyPair, EthTransaction, H256, U256, Bytes};
use std::str::FromStr;

fn main() {
    let mut event_loop = tokio_core::reactor::Core::new().unwrap();
    let eth = web3::Web3::new(
        web3::transports::Http::with_event_loop(
            "https://kovan.infura.io/v3/5b83a690fa934df09253dd2843983d89",
            &event_loop.handle(),
            64,
        ).unwrap(),
    ).eth();

    let key_pair = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").map(|privKey| {
        KeyPair::from_privkey(privKey)
    }).unwrap();
    let height: u64 = event_loop.run(eth.block_number()).unwrap().into();
    let nonce = event_loop.run(eth.transaction_count(key_pair.address(), None)).unwrap();
    let transaction = EthTransaction {
                                    nonce: nonce.into(),
                                    to: Some(key_pair.address()),
                                    value: 0.into(),
                                    data: b"608050".to_vec(),
                                    gas_price: 2000000000.into(),
                                    gas: 41000.into(),
                                };
    let data = signer::Eth::sign_transaction(key_pair.privkey(), &transaction);

    let bytes = Bytes::from(data);
    let send_raw_transaction = eth.send_raw_transaction(bytes)
        .map(|hash| {
            println!("transaction hash: {:?}", hash);
            hash
        });
    let hash = event_loop.run(send_raw_transaction).unwrap();
    let hash = H256::from_slice(hash.as_ref());
    loop {
        let receipt = event_loop.run(eth.transaction_receipt(hash)).unwrap();
        println!("receipt {:?}", receipt);
        if receipt.is_some() {break;}
        std::thread::sleep_ms(1000);
    }
}