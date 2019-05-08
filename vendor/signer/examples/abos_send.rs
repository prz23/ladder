extern crate signer;
extern crate web3;
extern crate tokio_core;
extern crate serde_json;
extern crate uuid;

use web3::futures::Future;
use web3::types::{Bytes, H256};
use std::str::FromStr;
use signer::{PrivKey};
use uuid::Uuid;

const MAX_PARALLEL_REQUESTS: usize = 64;

fn main() {

    let mut event_loop = tokio_core::reactor::Core::new().unwrap();
    let web3 = web3::Web3::new(
        web3::transports::Http::with_event_loop(
            "http://47.99.236.158:1339",
            &event_loop.handle(),
            MAX_PARALLEL_REQUESTS,
        ).unwrap(),
    );

    let meta_data = event_loop.run(web3.abos().meta_data(None)).unwrap();
    println!("metadata: {:?}", meta_data);
    let height: u64 = event_loop.run(web3.abos().block_number()).unwrap().into();
    println!("height: {}", height);
    let privKey = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
    let code = "608050".to_string();
    let to = "".to_owned();
    let nonce = Uuid::new_v4().to_string();
    let value:u64 = 0;
    let mut raw_tx = signer::generate_unverified_tx_vec(&privKey,code, to, height, 1000000, value, nonce);
    let bytes = Bytes::from(raw_tx);
    let send_raw_transaction = web3.abos().send_raw_transaction(bytes)
        .map(|hash| {
            println!("transaction hash: {:?}", hash);
            hash.hash
        });
    let hash = event_loop.run(send_raw_transaction).unwrap();
    let hash = H256::from_slice(hash.as_ref());
    loop {
        let receipt = event_loop.run(web3.abos().transaction_receipt(hash)).unwrap();
        println!("receipt {:?}", receipt);
        if receipt.is_some() {break;}
        std::thread::sleep_ms(1000);
    }
}