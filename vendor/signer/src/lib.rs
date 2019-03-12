#![allow(dead_code)]

extern crate secp256k1;
extern crate tiny_keccak;
#[macro_use]
extern crate lazy_static;
extern crate web3;
extern crate rlp;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rustc_hex;

mod keypair;

use web3::{types::{U256, H160}};
use rlp::{RlpStream};

pub use secp256k1::{Secp256k1, Message, SecretKey, PublicKey, All};
pub use keypair::{KeyPair, Keyring, PrivKey, PubKey};

lazy_static! {
    pub static ref SECP256K1: Secp256k1<All> = Secp256k1::new();
}

/// Description of a Transaction, pending or in the chain.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct RawTransaction {
    /// Nonce
    pub nonce: U256,
    /// Recipient (None when contract creation)
    pub to: Option<H160>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,
    /// Gas amount
    pub gas: U256,
    /// Input data
    pub data: Vec<u8>
}

trait Hasher {
    fn hash(&self) -> [u8; 32];
}

impl<T> Hasher for T where T: AsRef<[u8]> {
    fn hash(&self) -> [u8; 32] {
        let mut result = [0u8; 32];
        let input: &[u8] = self.as_ref();
        tiny_keccak::Keccak::keccak256(input, &mut result);
        result
    }
}

impl Hasher for RawTransaction {
    fn hash(&self) -> [u8; 32] {
        let mut hash = RlpStream::new();
        hash.begin_unbounded_list();
        encode(&self, &mut hash);
        hash.complete_unbounded_list();
        hash.out().hash()
    }
}

fn encode(raw: &RawTransaction, s: &mut RlpStream) {
    s.append(&raw.nonce);
    s.append(&raw.gas_price);
    s.append(&raw.gas);
    if let Some(ref t) = raw.to {
        s.append(t);
    } else {
        s.append(&vec![]);
    }
    s.append(&raw.value);
    s.append(&raw.data);
}

fn sign(secret_key: &SecretKey, message: &Message) -> [u8; 65] {
    let secp = &SECP256K1;
    let signture = secp.sign_recoverable(message, &secret_key);
    let (rec_id, data) = signture.serialize_compact();
    let mut sgn = [0u8; 65];
    sgn[0..64].copy_from_slice(&data[..]);
    sgn[64] = rec_id.to_i32() as u8;
    sgn
}

pub fn sign_message(secret_key: &SecretKey, message:& Vec<u8>) -> Vec<u8> {
    let mut message_data =
        format!("\x19Ethereum Signed Message:\n{}" ,message.len())
            .into_bytes();
    message_data.append(&mut message.clone());
    let message = Message::from_slice(&message_data.hash()).unwrap();
    let sgn = sign(secret_key, &message);
    sgn.to_vec()
}

pub fn sign_transaction(secret_key: &SecretKey, raw: &RawTransaction) -> Vec<u8> {
    let message = Message::from_slice(&raw.hash()).unwrap();
    let mut sgn = sign(secret_key, &message);
    sgn[64] += 27;

    let mut tx = RlpStream::new();
    tx.begin_unbounded_list();
    encode(raw, &mut tx);
    tx.append(&sgn[64]);
    tx.append(&&sgn[0..32]);
    tx.append(&&sgn[32..64]);
    tx.complete_unbounded_list();
    tx.out()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use rustc_hex::ToHex;

    #[test]
    fn sign_message_test() {
        let nonce = vec![0x30];
        let priv_key = SecretKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
        let out = sign_message(&priv_key, &nonce);
        println!("signture: {}", out.to_hex());
        assert_eq!(out.len(), 65);
        let signture = "e3e044fd77db535d8e2ab9d064b8a7d99a0cd99a7af307607e21610ebed5a9aa4acd9c190c8f8d8cb8b074dde0a539a37c9f802d95a44997febd0acff5d6f56d01".to_owned();
        assert_eq!(out.to_hex(), signture);
    }

    #[test]
    fn eth_sign_empty_test() {
        let priv_key = SecretKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
        let trans = RawTransaction {
            nonce: 7.into(),
            to: Some(H160::from_str("4b5Ae4567aD5D9FB92Bc9aFd6A657e6fA13a2523").unwrap()),
            value: 0.into(),
            data: vec![],
            gas_price: 20.into(),
            gas: 21000.into(),
        };
        let signed = sign_transaction(&priv_key, &trans);
        let target = String::from("f85f0614825208944b5ae4567ad5d9fb92bc9afd6a657e6fa13a252380801ca06832fa6a99f05ec7418e2bb94ecc6e1bbacb4bb2df3fbe563fabcb8faac9507fa068890efb97f4dcc26fb646f841dbfc680337b47e6f41c1cd3bd23edadaf42b60");
        println!("signed:{}\ntarget:{}", signed.to_hex(), target);
    }
}