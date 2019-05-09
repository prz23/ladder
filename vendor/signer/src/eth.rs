use rlp::RlpStream;
use secp256k1::{Message, SecretKey};
use crate::keypair::PrivKey;
use crate::hasher::{Hasher, Hash};
use crate::types::{U256, H160};
use super::sign;

/// Description of a Transaction for ethereum.
#[derive(Debug, Default, Clone)]
pub struct EthTransaction {
    /// Nonce
    pub nonce: U256,
    /// Recipient (None when contract creation)
    pub to: Option<H160>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    pub gas_price: U256,
    /// Gas amount
    pub gas: U256,
    /// Input data
    pub data: Vec<u8>
}

impl EthTransaction {
    pub fn rlp_hash(&self) -> Hash {
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();
        self.rlp_encode(&mut rlp);
        rlp.complete_unbounded_list();
        rlp.out().hash()
    }

    pub fn rlp_encode(&self, s: &mut RlpStream) {
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas);
        if let Some(ref t) = self.to {
            s.append(t);
        } else {
            s.append(&vec![]);
        }
        s.append(&self.value);
        s.append(&self.data);
    }
}

/// ethereum mode
pub struct Eth;

impl Eth {

    /// 
    pub fn sign_message(priv_key: &PrivKey, message:& Vec<u8>) -> Vec<u8> {
        let secret_key: &SecretKey = unsafe { std::mem::transmute(priv_key) };
        let mut message_data =
            format!("\x19Ethereum Signed Message:\n{}" ,message.len())
                .into_bytes();
        message_data.append(&mut message.clone());
        let message = Message::from_slice(&message_data.hash()).unwrap();
        let sgn = sign(secret_key, &message);
        sgn.to_vec()
    }

    /// 
    pub fn sign_transaction(priv_key: &PrivKey, raw: &EthTransaction) -> Vec<u8> {
        // 
        let secret_key: &SecretKey = unsafe { std::mem::transmute(priv_key) };
        let message = Message::from_slice(&raw.rlp_hash()).unwrap();
        let mut sgn = sign(secret_key, &message);
        // ethereum style
        sgn[64] += 27;

        let mut tx = RlpStream::new();
        tx.begin_unbounded_list();
        raw.rlp_encode(&mut tx);
        tx.append(&sgn[64]);
        tx.append(&&sgn[0..32]);
        tx.append(&&sgn[32..64]);
        tx.complete_unbounded_list();
        tx.out()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use rustc_hex::ToHex;

    #[test]
    fn sign_message_test() {
        let nonce = vec![0x30];
        let priv_key = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
        let out = Eth::sign_message(&priv_key, &nonce);
        println!("signture: {}", out.to_hex());
        assert_eq!(out.len(), 65);
        let signture = "e3e044fd77db535d8e2ab9d064b8a7d99a0cd99a7af307607e21610ebed5a9aa4acd9c190c8f8d8cb8b074dde0a539a37c9f802d95a44997febd0acff5d6f56d01".to_owned();
        assert_eq!(out.to_hex(), signture);
    }

    #[test]
    fn eth_sign_empty_test() {
        let trans = EthTransaction {
            nonce: 7.into(),
            to: Some(H160::from_str("4b5Ae4567aD5D9FB92Bc9aFd6A657e6fA13a2523").unwrap()),
            value: 0.into(),
            data: vec![],
            gas_price: 20.into(),
            gas: 21000.into(),
        };
        let priv_key = PrivKey::from_str("5f0258a4778057a8a7d97809bd209055b2fbafa654ce7d31ec7191066b9225e6").unwrap();
        let signed = Eth::sign_transaction(&priv_key, &trans);
        let target = String::from("f85f0714825208944b5ae4567ad5d9fb92bc9afd6a657e6fa13a252380801ca06634d0a028da65301368c238e88185c048be3cb56c3ec1928f8ec7a7736813c0a0568a101dfc3c975390226a3a16cf12544cd8777aebc41e2d36f6f7ce570bea13");
        assert_eq!(signed.to_hex(), target);
    }
}