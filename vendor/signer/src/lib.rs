#![allow(dead_code)]
#![feature(try_from)]

mod keypair;
mod transaction;
mod hasher;
mod types;
pub mod eth;
pub mod abos;

pub use keypair::{KeyPair, Keyring, PrivKey, PubKey};
pub use eth::{Eth, EthTransaction};
pub use abos::{Abos, AbosTransaction};
pub use types::*;

use lazy_static::lazy_static;
use secp256k1::{Secp256k1, SecretKey, Message, All};

lazy_static! {
    pub static ref SECP256K1: Secp256k1<All> = Secp256k1::new();
}

pub fn sign(secret_key: &SecretKey, message: &Message) -> [u8; 65] {
    let secp = &SECP256K1;
    let signture = secp.sign_recoverable(message, &secret_key);
    let (rec_id, data) = signture.serialize_compact();
    let mut sgn = [0u8; 65];
    sgn[0..64].copy_from_slice(&data[..]);
    sgn[64] = rec_id.to_i32() as u8;
    sgn
}