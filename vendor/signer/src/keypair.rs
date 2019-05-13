use rustc_hex::ToHex;
use secp256k1::{SecretKey, PublicKey};
use std::fmt;
use crate::types::{H256, H512, Address};
use crate::hasher::Hasher;
use super::SECP256K1;


pub type PrivKey = H256;
pub type PubKey = H512;

#[derive(Default)]
pub struct Keyring {
    key: Vec<u8>,
}

impl Keyring {
    pub fn from(data: &[u8]) -> Self {
        Self {
            key: data.to_vec(),
        }
    }

    pub fn to_key(&self) -> PrivKey {
        let mut key = PrivKey::from([8u8;32]);
        let len = if self.key.len() > 32 { 32 } else { self.key.len() };
        key[0..len].copy_from_slice(&self.key[0..len]);
        key
    }

    pub fn to_hex(&self) -> String {
        self.to_key().to_hex()
    }

    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }
}

/// key pair struct
#[derive(Clone)]
pub struct KeyPair {
    privkey: PrivKey,
    pubkey: PubKey,
}

impl fmt::Display for KeyPair {
   fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
       writeln!(f, "PrivKey:  {}", self.privkey.0.to_hex())?;
       writeln!(f, "PubKey:  {}", self.pubkey.0.to_hex())?;
       write!(f, "Address:  {}", self.address().to_hex())
   }
}

impl KeyPair {
    /// Create a pair from private key
    pub fn from_privkey(privkey: PrivKey) -> Self {
        let secp = &SECP256K1;
        let secret_key = SecretKey::from_slice(&privkey).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(secp, &secret_key);

        let serialized = public_key.serialize_uncompressed();
        let mut pubkey = PubKey::default();
        pubkey.0.copy_from_slice(&serialized[1..65]);

        KeyPair {
            privkey: privkey.into(),
            pubkey: pubkey,
        }
    }

    pub fn privkey(&self) -> &PrivKey {
        &self.privkey
    }

    pub fn pubkey(&self) -> &PubKey {
        &self.pubkey
    }

    pub fn address(&self) -> Address {
        pubkey_to_address(&self.pubkey)
    }
}

pub fn pubkey_to_address(pubkey: &PubKey) -> Address {
    let mut ret = Address::default();
    let value = pubkey.hash();
    ret.0.copy_from_slice(&value[12..32]);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /*
    {
    "address": "0x59dc5d8803b482ddbf361ebaccbacc413925ab28",
    "private": "0xde0a3ae1674c881e96659ba568d06c6807876c41d13f63444111d57d4754a866",
    "public": "0x6f9f3e73053d41eba3f3b376cfe81586a6487f65588d1f3198bf14b776dc2faba521d6e9a9bfcbc16eb20f7d909c31eab8ce839f014bbb56d9c8416644c17519"
    }
    */
    #[test]
    fn from_privkey() {
        let privkey = PrivKey::from(
            H256::from_str("de0a3ae1674c881e96659ba568d06c6807876c41d13f63444111d57d4754a866")
                .unwrap(),
        );
        let pair = KeyPair::from_privkey(privkey);
        assert_eq!(pair.address(), Address::from_str("59dc5d8803b482ddbf361ebaccbacc413925ab28").unwrap())
    }

    #[test]
    fn test_keyring() {
        let privkey = PrivKey::from(
            H256::from_str("de0a3ae1674c881e96659ba568d06c6807876c41d13f63444111d57d4754a866")
                .unwrap(),
        );
        let keyring = Keyring::from(&privkey);
        let pair = KeyPair::from_privkey(keyring.to_key());
        assert_eq!(pair.address(), Address::from_str("59dc5d8803b482ddbf361ebaccbacc413925ab28").unwrap());

        let keyring = Keyring::from(b"Alice");
        let pair = KeyPair::from_privkey(keyring.to_key());
        println!("Alice pair: {}", pair);
    }
}