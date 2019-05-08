#![allow(dead_code)]
#![feature(try_from)]

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
extern crate protobuf;


mod keypair;
mod transaction;

use web3::{types::{U256, H160, H256, Address}};
use rlp::{RlpStream};
use transaction::{Transaction, UnverifiedTransaction, SignedTransaction, Crypto};
use rustc_hex::FromHex;
use std::str::FromStr;
use std::convert::TryInto;
use protobuf::{Message as MessageTrait};

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

#[derive(Copy, Clone, Debug, Default)]
pub struct TryIntoConvertError(());

impl TryInto<Vec<u8>> for Transaction {
    type Error = TryIntoConvertError;
    fn try_into(self) -> Result<Vec<u8>, TryIntoConvertError> {
        self.write_to_bytes().map_err(|_| { TryIntoConvertError(()) })
    }
}

impl TryInto<Vec<u8>> for UnverifiedTransaction {
    type Error = TryIntoConvertError;
    fn try_into(self) -> Result<Vec<u8>, TryIntoConvertError> {
        self.write_to_bytes().map_err(|_| { TryIntoConvertError(()) })
    }
}

impl TryInto<Vec<u8>> for &Transaction {
    type Error = TryIntoConvertError;
    fn try_into(self) -> Result<Vec<u8>, TryIntoConvertError> {
        self.write_to_bytes().map_err(|_| { TryIntoConvertError(()) })
    }
}

impl TryInto<Vec<u8>> for &UnverifiedTransaction {
    type Error = TryIntoConvertError;
    fn try_into(self) -> Result<Vec<u8>, TryIntoConvertError> {
        self.write_to_bytes().map_err(|_| { TryIntoConvertError(()) })
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

impl Transaction {
    /// Signs the transaction by PrivKey.
    pub fn sign(&self, sk: PrivKey) -> SignedTransaction {
        let keypair = KeyPair::from_privkey(sk);
        let pubkey = keypair.pubkey();
        let unverified_tx = self.build_unverified(sk);

        // Build SignedTransaction
        let mut signed_tx = SignedTransaction::new();
        println!("pubkey: {}", keypair.pubkey());
        signed_tx.set_signer(pubkey.to_vec());
        let bytes: Vec<u8> = (&unverified_tx).try_into().unwrap();
        signed_tx.set_tx_hash(bytes.hash().to_vec());
        signed_tx.set_transaction_with_sig(unverified_tx);
        signed_tx
    }

    /// Build UnverifiedTransaction
    pub fn build_unverified(&self, sk: PrivKey) -> UnverifiedTransaction {
        let sec: &SecretKey = unsafe { std::mem::transmute(&sk) };
        let mut unverified_tx = UnverifiedTransaction::new();
        let bytes: Vec<u8> = self.try_into().unwrap();
        let message = Message::from_slice(&bytes.hash()).unwrap();
        unverified_tx.set_transaction(self.clone());
        let signature = sign(sec, &message);
        unverified_tx.set_signature(signature.to_vec());
        unverified_tx.set_crypto(Crypto::DEFAULT);
        unverified_tx
    }
}

/// generate signed transaction
pub fn generate_signed_tx(
    pk: &PrivKey,
    code: String,
    to: String,
    height: u64,
    quota: u64,
    value: u64,
    nonce: String,
) -> SignedTransaction {
    let version = 1u32;
    let data = code.from_hex().unwrap();
    let mut tx = Transaction::new();
    tx.set_data(data);
    tx.set_nonce(nonce);
    tx.set_valid_until_block(height + 88);
    tx.set_quota(quota);
    tx.set_value(H256::from(U256::from(value)).to_vec());
    match version {
        0 => {
            tx.set_to(to);
            tx.set_chain_id(1);
        },
        _ => {
            if !to.is_empty() {
                tx.set_to_v1(Address::from_str(&to).unwrap().to_vec());
            }
            tx.set_chain_id_v1(H256::from(U256::from(version)).to_vec());
        }
    }
    tx.set_version(version);
    tx.sign(*pk)
}

/// generate UnverifiedTransaction
pub fn generate_unverified_tx(
    pk: &PrivKey,
    code: String,
    to: String,
    height: u64,
    quota: u64,
    value: u64,
    nonce: String,
) -> UnverifiedTransaction {
    generate_signed_tx(pk, code, to, height, quota, value, nonce)
        .take_transaction_with_sig()
}

/// generate UnverifiedTransaction vector
pub fn generate_unverified_tx_vec(
    pk: &PrivKey,
    code: String,
    to: String,
    height: u64,
    quota: u64,
    value: u64,
    nonce: String,
) -> Vec<u8> {
    generate_signed_tx(pk, code, to, height, quota, value, nonce)
        .take_transaction_with_sig().write_to_bytes().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use rustc_hex::ToHex;

    // #[test]
    // fn sign_message_test() {
    //     let nonce = vec![0x30];
    //     let priv_key = SecretKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
    //     let out = sign_message(&priv_key, &nonce);
    //     println!("signture: {}", out.to_hex());
    //     assert_eq!(out.len(), 65);
    //     let signture = "e3e044fd77db535d8e2ab9d064b8a7d99a0cd99a7af307607e21610ebed5a9aa4acd9c190c8f8d8cb8b074dde0a539a37c9f802d95a44997febd0acff5d6f56d01".to_owned();
    //     assert_eq!(out.to_hex(), signture);
    // }

    // #[test]
    // fn eth_sign_empty_test() {
    //     let priv_key = SecretKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
    //     let trans = RawTransaction {
    //         nonce: 7.into(),
    //         to: Some(H160::from_str("4b5Ae4567aD5D9FB92Bc9aFd6A657e6fA13a2523").unwrap()),
    //         value: 0.into(),
    //         data: vec![],
    //         gas_price: 20.into(),
    //         gas: 21000.into(),
    //     };
    //     let signed = sign_transaction(&priv_key, &trans);
    //     let target = String::from("f85f0614825208944b5ae4567ad5d9fb92bc9afd6a657e6fa13a252380801ca06832fa6a99f05ec7418e2bb94ecc6e1bbacb4bb2df3fbe563fabcb8faac9507fa068890efb97f4dcc26fb646f841dbfc680337b47e6f41c1cd3bd23edadaf42b60");
    //     println!("signed:{}\ntarget:{}", signed.to_hex(), target);
    // }

    // #[test]
    // fn signed_contract_test(){
    //     let tar = "0x0a36120331323318c0843d20c0c4072a03608050322000000000000000000000000000000000000000000000000000000000000000003801124157c1eded88c25788cd62e1e894d4afe919a5f81fc7d2f053ebef253535eecc646ce95b9eb45fe0f5775421db2db3f8c5634983c0ab640f701bccd0fee499a59801".to_string();
    //     let code = "608050".to_string();
    //     let to = "".to_owned();
    //     let nonce = "123".to_owned();
    //     let priv_key = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
    //     let mut signed = generate_signed_tx(&priv_key, code, to, 123456, 1000000, 0, nonce);
    //     // let signed = helpers::serialize(&Bytes::from(signed.take_transaction_with_sig().write_to_bytes().unwrap()));
    //     // assert_eq!(tar, signed);
    // }

    #[test]
    fn signed_tx_test(){
        let tar = "0x0a5b0a2839646663363535653734316137663163666561373563396232356535313937633234623637636236120331323318c0843d20c0c4073220000000000000000000000000000000000000000000000000000000000000000038011241ce7e09e8a6311b1c3c432f1c1f6ddf2704c1a7206d7b3fa7751a73f0cc6f09ce4205d2d6cf135965a303e6bf8b6d7cfd4a950f27e98d5d1a43288f5d2e72001c01".to_string();
        let code = "".to_string();
        let to = "9dfc655e741a7f1cfea75c9b25e5197c24b67cb6".to_owned();
        let nonce = "123".to_owned();
        let priv_key = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
        let mut signed = generate_signed_tx(&priv_key, code, to, 123456, 1000000, 0, nonce);
        let d = signed.take_transaction_with_sig().write_to_bytes().unwrap();
        println!("{:?}", d.to_hex());
        // let signed = helpers::serialize(&Bytes::from(signed.take_transaction_with_sig().write_to_bytes().unwrap()));
        // assert_eq!(tar, signed);
    }
}