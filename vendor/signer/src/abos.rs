use secp256k1::{Message, SecretKey};
use rustc_hex::ToHex;
use protobuf::{Message as OtherMessage};
use crate::keypair::PrivKey;
use crate::hasher::{Hasher, Hash};
use crate::types::{H256, U256, H160};
use crate::transaction::{Transaction, UnverifiedTransaction, Crypto};
use super::sign;
use crate::eth::Eth;

/// Description of a Transaction for abos.
#[derive(Clone)]
pub struct AbosTransaction {
    /// Recipient (None when contract creation)
    pub to: Option<H160>,
    /// Nonce
    pub nonce: String,
    /// Quota amount
    pub quota: u64,
    /// Valid until block number
    pub valid_until_block: u64,
    /// Input data
    pub data: Vec<u8>,
    /// Transfered value
    pub value: U256,
    /// Chain id
    pub chain_id: U256,
    /// distinguish transaction default is 0, > v0.20 is not 0
    pub version: u32,
}

impl Transaction {
    pub fn protobuf_hash(&self) -> Hash {
        self.write_to_bytes().unwrap().hash()
    }
}

impl From<&AbosTransaction> for Transaction {
    fn from(trans: &AbosTransaction) -> Self {
        let mut tx = Transaction::new();
        let trans = trans.clone();
        tx.set_data(trans.data);
        tx.set_nonce(trans.nonce);
        tx.set_valid_until_block(trans.valid_until_block);
        tx.set_quota(trans.quota);
        tx.set_value(H256::from(U256::from(trans.value)).to_vec());
        match trans.version {
            0 => {
                tx.set_to(trans.to.map_or(String::new(), |t| t.to_hex()));
                tx.set_chain_id(trans.chain_id.low_u32());
            },
            _ => {
                tx.set_to_v1(trans.to.map_or(vec![], |t| t.to_vec()));
                tx.set_chain_id_v1(H256::from(trans.chain_id).to_vec());
            }
        }
        tx.set_version(trans.version);
        tx
    }
}

/// ABOS mode
pub struct Abos;

impl Abos {
    /// 
    pub fn sign_message(priv_key: &PrivKey, message:& Vec<u8>) -> Vec<u8> {
        // use ethereum mode directly.
        Eth::sign_message(priv_key, message)
    }

    /// 
    pub fn sign_transaction(priv_key: &PrivKey, raw: &AbosTransaction) -> Vec<u8> {
        let sec: &SecretKey = unsafe { std::mem::transmute(priv_key) };
        // convert to protobuf transaction, to bytes, use keccak hash.
        let tx = Transaction::from(raw);
        let message = Message::from_slice(&tx.protobuf_hash()).unwrap();
        let signature = sign(sec, &message);

        let mut unverified_tx = UnverifiedTransaction::new();
        unverified_tx.set_transaction(tx);
        unverified_tx.set_signature(signature.to_vec());
        unverified_tx.set_crypto(Crypto::DEFAULT);

        unverified_tx.write_to_bytes().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use rustc_hex::ToHex;

    #[test]
    fn signed_contract_test(){
        let priv_key = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
        let tar = "0a5b120331323318c0c40720a08d062a0636303830353032200000000000000000000000000000000000000000000000000000000000000000400152200000000000000000000000000000000000000000000000000000000000000000124138534fabe28a3b83d92ceb5e2535ceec45c17356cf669cd5b5adafcc4695199868b908ae58d38d27f48f142ce13fc620aa414ad066f385323943aa281316b6f401".to_owned();
        let tx = AbosTransaction {
            to: None,
            nonce: "123".to_owned(),
            quota: 123456,
            valid_until_block: 100000,
            data: b"608050".to_vec(),
            value: U256::from(0),
            chain_id: U256::from(0),
            version: 1,
        };
        let signed = Abos::sign_transaction(&priv_key, &tx);
        println!("signed {:?}", signed.to_hex());
        assert_eq!(tar, signed.to_hex());
    }

    #[test]
    fn signed_tx_test(){
        let tar = "0a69120331323318c0c40720a08d063220000000000000000000000000000000000000000000000000000000000000000040014a149dfc655e741a7f1cfea75c9b25e5197c24b67cb6522000000000000000000000000000000000000000000000000000000000000000001241192fde8f018453b3d14c35f0ea05acfc9caf6a3584079ed00811e6146e1bb2a36aeb33f7070c2e3f8fe5571b3a286640063e402ff3dc0868d4f702cb10e74d8900".to_owned();
        let priv_key = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
        let tx = AbosTransaction {
            to: H160::from_str("9dfc655e741a7f1cfea75c9b25e5197c24b67cb6").ok(),
            nonce: "123".to_owned(),
            quota: 123456,
            valid_until_block: 100000,
            data: b"".to_vec(),
            value: U256::from(0),
            chain_id: U256::from(0),
            version: 1,
        };
        let signed = Abos::sign_transaction(&priv_key, &tx);
        println!("transaction with sign: {:?}", signed.to_hex());
        assert_eq!(tar, signed.to_hex());
    }
}