extern crate signer;
extern crate web3;
extern crate tokio_core;

use web3::futures::Future;
use std::str::FromStr;
use signer::{PrivKey, AbosTransaction, H256, U256, Bytes};

fn main() {
    let mut event_loop = tokio_core::reactor::Core::new().unwrap();
    let abos = web3::Web3::new(
        web3::transports::Http::with_event_loop(
            "http://47.99.236.158:1339",
            &event_loop.handle(),
            64,
        ).unwrap(),
    ).abos();

    // get meta data.
    let meta_data = event_loop.run(abos.meta_data(None)).unwrap();
    println!("metadata: {:?}", meta_data);
    let height: u64 = event_loop.run(abos.block_number()).unwrap().into();
    println!("height: {}", height);
    let priv_key = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
    let tx = AbosTransaction {
            to: None,
            nonce: "123".to_owned(),
            quota: 123456,
            valid_until_block: height + 88,
            data: b"608050".to_vec(),
            value: U256::from(0),
            chain_id: U256::from(1),
            version: 1,
        };
    let raw_tx = signer::Abos::sign_transaction(&priv_key, &tx);
    let send_raw_transaction = abos.send_raw_transaction(Bytes::from(raw_tx))
        .map(|hash| {hash.hash});
    let hash = event_loop.run(send_raw_transaction).unwrap();
    let hash = H256::from_slice(hash.as_ref());
    
    loop {
        let receipt = event_loop.run(abos.transaction_receipt(hash)).unwrap();
        println!("receipt {:?}", receipt);
        if receipt.is_some() {break;}
        std::thread::sleep_ms(1000);
    }
}