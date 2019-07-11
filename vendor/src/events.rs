use crate::error::Error;
use crate::label::ChainAlias;
use crate::utils::IntoRawLog;
use contracts;
use error_chain::bail;
use std::str::FromStr;
use web3::types::{Address, Log, H256, U256};

pub const ORACLE_LENTH: usize = 116; // 8 8
pub const LOCKTOKEN_LENGTH: usize = 180;
pub const UNLOCKTOKEN_LENGTH: usize = 64;

impl ChainAlias {
    // TODO refactor
    fn coin(&self) -> u64 {
        match &self {
            ChainAlias::ETH => 1,
            ChainAlias::ABOS => 2,
        }
    }
}

pub fn array_to_u32(arr: [u8; 4]) -> u32 {
    let u = unsafe { std::mem::transmute::<[u8; 4], u32>(arr) };
    u
}

pub fn u32_to_array(data: u32) -> [u8; 4] {
    let bytes: [u8; 4] = unsafe { std::mem::transmute(data.to_le()) };
    bytes
}


pub fn u64_to_array(data: u64) -> [u8; 8] {
    let bytes: [u8; 8] = unsafe { std::mem::transmute(data.to_le()) };
    bytes
}

pub fn array_to_u64(arr: [u8; 8]) -> u64 {
    let u = unsafe { std::mem::transmute::<[u8; 8], u64>(arr) };
    u
}

#[derive(Debug)]
pub struct DepositEvent {
    pub coin: u64,
    pub sender: Address,
    pub recipient: H256,
    pub value: U256,
    pub tx_hash: H256,
}

impl DepositEvent {
    const BANKER_LENGTH:usize = 124;
    pub fn from_log(raw_log: &Log, chain: &ChainAlias) -> Result<Self, Error> {
        let hash = raw_log
            .transaction_hash
            .ok_or_else(|| "`log` must be mined and contain `transaction_hash`")?;
        let log = contracts::bridge::events::deposit::parse_log(raw_log.into_raw_log())?;
        Ok(Self {
            coin: chain.coin(),
            sender: log.account,
            recipient: log.beneficiary,
            value: log.amount,
            tx_hash: hash,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != Self::BANKER_LENGTH {
            bail!("`bytes`.len() must be {}", Self::BANKER_LENGTH);
        }
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(&bytes[0..8]);
        Ok(Self {
            coin: u64::from_be_bytes(tmp),
            sender: bytes[8..28].into(),
            recipient: bytes[28..60].into(),
            value: U256::from_big_endian(&bytes[60..92]),
            tx_hash: bytes[92..Self::BANKER_LENGTH].into(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![0u8; Self::BANKER_LENGTH];
        result[0..8].copy_from_slice(&self.coin.to_be_bytes());
        result[8..28].copy_from_slice(&self.sender.0[..]);
        result[28..60].copy_from_slice(&self.recipient.0[..]);
        self.value.to_big_endian(&mut result[60..92]);
        result[92..Self::BANKER_LENGTH].copy_from_slice(&self.tx_hash.0[..]);
        return result;
    }
}

#[derive(Debug)]
pub struct RequestEvent {
    pub coin: u64,
    pub id: U256,
    pub sender: Address,
    pub recipient: H256,
    pub value: U256,
    pub tx_hash: H256,
}

impl RequestEvent {
    const REQUEST_LENGTH:usize = 156;
    pub fn from_log(raw_log: &Log, chain: &ChainAlias) -> Result<Self, Error> {
        let hash = raw_log
            .transaction_hash
            .ok_or_else(|| "`log` must be mined and contain `transaction_hash`")?;
        let log = contracts::bridge::events::request::parse_log(raw_log.into_raw_log())?;
        Ok(Self {
            coin: chain.coin(),
            id: log.id,
            sender: log.account,
            recipient: log.beneficiary,
            value: log.amount,
            tx_hash: hash,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != Self::REQUEST_LENGTH {
            bail!("`bytes`.len() must be {}", Self::REQUEST_LENGTH);
        }
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(&bytes[0..8]);
        Ok(Self {
            coin: u64::from_be_bytes(tmp),
            id: U256::from_big_endian(&bytes[8..40]),
            sender: bytes[40..60].into(),
            recipient: bytes[60..92].into(),
            value: U256::from_big_endian(&bytes[92..124]),
            tx_hash: bytes[124..Self::REQUEST_LENGTH].into(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![0u8; Self::REQUEST_LENGTH];
        result[0..8].copy_from_slice(&self.coin.to_be_bytes());
        self.id.to_big_endian(&mut result[8..40]);
        result[40..60].copy_from_slice(&self.sender.0[..]);
        result[60..92].copy_from_slice(&self.recipient.0[..]);
        self.value.to_big_endian(&mut result[92..124]);
        result[124..Self::REQUEST_LENGTH].copy_from_slice(&self.tx_hash.0[..]);
        return result;
    }
}

#[derive(Debug)]
pub struct WithdrawEvent {
    pub coin: u64,
    pub id: U256,
    pub sender: Address,
    pub recipient: H256,
    pub value: U256,
    pub tx_hash: H256,
}

impl WithdrawEvent {
    const WITHDRAW_LENGTH:usize = 156;

    pub fn from_log(raw_log: &Log, chain: &ChainAlias) -> Result<Self, Error> {
        let hash = raw_log
            .transaction_hash
            .ok_or_else(|| "`log` must be mined and contain `transaction_hash`")?;
        let log = contracts::bridge::events::withdraw::parse_log(raw_log.into_raw_log())?;
        Ok(Self {
            coin: chain.coin(),
            id: log.id,
            sender: log.account,
            recipient: log.beneficiary,
            value: log.amount,
            tx_hash: hash,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != Self::WITHDRAW_LENGTH {
            bail!("`bytes`.len() must be {}", Self::WITHDRAW_LENGTH);
        }
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(&bytes[0..8]);
        Ok(Self {
            coin: u64::from_be_bytes(tmp),
            id: U256::from_big_endian(&bytes[8..40]),
            sender: bytes[40..60].into(),
            recipient: bytes[60..92].into(),
            value: U256::from_big_endian(&bytes[92..124]),
            tx_hash: bytes[124..Self::WITHDRAW_LENGTH].into(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![0u8; Self::WITHDRAW_LENGTH];
        result[0..8].copy_from_slice(&self.coin.to_be_bytes());
        self.id.to_big_endian(&mut result[8..40]);
        result[40..60].copy_from_slice(&self.sender.0[..]);
        result[60..92].copy_from_slice(&self.recipient.0[..]);
        self.value.to_big_endian(&mut result[92..124]);
        result[124..Self::WITHDRAW_LENGTH].copy_from_slice(&self.tx_hash.0[..]);
        return result;
    }
}

#[derive(Debug)]
pub struct AuthorityEvent {
    pub coin: u64,
    pub last_len: u32,
    pub last: Vec<Address>,
    pub next_len: u32,
    pub next: Vec<Address>,
    pub tx_hash: H256,
}

impl AuthorityEvent {
    pub const AUTHORITY_MINIMUM_LENGTH: usize = 72;

    pub fn from_log(raw_log: &Log, chain: &ChainAlias) -> Result<Self, Error> {
        let hash = raw_log
            .transaction_hash
            .ok_or_else(|| "`log` must be mined and contain `transaction_hash`")?;
        let log = contracts::bridge::events::replace_auths::parse_log(raw_log.into_raw_log())?;
        Ok(Self {
            coin: chain.coin(),
            last_len: (log.last.len() as u32),
            last: log.last,
            next_len: (log.next.len() as u32),
            next: log.next,
            tx_hash: hash,
        })
    }

    /*
    0:  8               bytes  H256  coin
    8: 12               bytes  u32   last_len
    12: 12+20*last_len   bytes  [Address]
    a: a+4               bytes  u32   next_len
    a+4: a+4+20*next_len bytes  [address]
    b: b+32              bytes H256   tx_hash

    min = 8 + 4 + 4 + 32 = 48;
    */
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < Self::AUTHORITY_MINIMUM_LENGTH {
            bail!(
                "`bytes`.len() must be more than {}",
                Self::AUTHORITY_MINIMUM_LENGTH
            );
        }
        let mut tmp: [u8; 8] = [0u8; 8];
        let mut index: usize = 0;
        tmp.copy_from_slice(&bytes[index..(index + 8)]);
        let coin = u64::from_be_bytes(tmp);
        index += 8;
        let mut tmp: [u8; 4] = [0u8; 4];
        tmp.copy_from_slice(&bytes[index..(index + 4)]);
        let last_len = u32::from_be_bytes(tmp);
        index += 4;

        let mut last: Vec<Address> = Vec::with_capacity(last_len as usize);
        for _ in 0..last_len {
            let address: Address = bytes[index..(index + 20)].into();
            index += 20;
            last.push(address);
        }

        tmp.copy_from_slice(&bytes[index..(index + 4)]);
        let next_len = u32::from_be_bytes(tmp);
        index += 4;

        let next: Vec<Address> = (0..next_len)
            .map(|_i| {
                let address: Address = bytes[index..(index + 20)].into();
                index += 20;
                address
            })
            .collect();

        let tx_hash = bytes[index..(index + 32)].into();
        Ok(Self {
            coin: coin,
            last_len: last_len,
            last: last,
            next_len: next_len,
            next: next,
            tx_hash: tx_hash,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let capacity = Self::AUTHORITY_MINIMUM_LENGTH
            + (self.last_len as usize) * 20
            + (self.next_len as usize) * 20;
        let mut result: Vec<u8> = Vec::with_capacity(capacity);
        let mut index = 0;
        result[index..(index + 8)].copy_from_slice(&self.coin.to_be_bytes());
        index += 8;
        result[index..(index + 4)].copy_from_slice(&self.last_len.to_be_bytes());
        index += 4;
        for i in 0..self.last_len as usize {
            result[index..(index + 20)].copy_from_slice(&(self.last[i].0[..]));
            index += 20;
        }
        result[index..(index + 4)].copy_from_slice(&self.next_len.to_be_bytes());
        index += 4;
        for i in 0..self.next_len as usize {
            result[index..(index + 20)].copy_from_slice(&self.next[i].0[..]);
            index += 20;
        }
        result[index..(index + 32)].copy_from_slice(&self.tx_hash.0[..]);
        return result;
    }
}

#[derive(Debug)]
pub struct MatchEvent {
    pub tag: u64,
    pub bill: u64,
    pub from: Address,
    pub from_bond: H256,
    pub to: Address,
    pub to_bond: H256,
    pub value: U256,
    pub reserved: u8,
}

impl MatchEvent {
    const SETTLE_LENGTH: usize = 153;
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != Self::SETTLE_LENGTH {
            bail!("`bytes`.len() must be {}", Self::SETTLE_LENGTH);
        }
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(&bytes[0..8]);
        let tag = u64::from_be_bytes(tmp);
        tmp.copy_from_slice(&bytes[8..16]);
        let bill = u64::from_be_bytes(tmp);
        Ok(Self {
            tag: tag,
            bill: bill,
            from: bytes[16..36].into(),
            from_bond: bytes[36..68].into(),
            to: bytes[68..88].into(),
            to_bond: bytes[88..120].into(),
            value: U256::from_big_endian(&bytes[120..152]),
            reserved: bytes[152],
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![0u8; Self::SETTLE_LENGTH];
        result[0..8].copy_from_slice(&self.tag.to_be_bytes());
        result[8..16].copy_from_slice(&self.bill.to_be_bytes());
        result[16..36].copy_from_slice(&self.from.0[..]);
        result[36..68].copy_from_slice(&self.from_bond.0[..]);
        result[68..88].copy_from_slice(&self.to.0[..]);
        result[88..120].copy_from_slice(&self.to_bond.0[..]);
        self.value.to_big_endian(&mut result[120..152]);
        result[152] = self.reserved;
        return result;
    }
}

#[derive(Debug)]
pub struct ExchangeRateEvent {
    pub pair: u64, //组合类型 1-ETHUSD  2-BITUSD  ……
    pub time: u64, //该汇率的时间
    pub rate: u64,
    pub tx_hash: H256,
}

impl ExchangeRateEvent {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![0u8; 56];
        result[0..8].copy_from_slice(&u64_to_array(self.rate));
        result[8..16].copy_from_slice(&u64_to_array(self.time));
        result[16..24].copy_from_slice(&u64_to_array(self.pair));

        result[24..56].copy_from_slice(&self.tx_hash.0[..]);
        return result;
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 56 {
            bail!("`bytes`.len() must be {}", 56);
        }
        let mut tmp: [u8; 8] = [0u8; 8];

        let _raw_pair = tmp.copy_from_slice(&bytes[0..8]);
        let pair = array_to_u64(tmp);

        let _raw_time = tmp.copy_from_slice(&bytes[8..16]);
        let time = array_to_u64(tmp);

        let _raw_rate = tmp.copy_from_slice(&bytes[16..24]);
        let rate = array_to_u64(tmp);

        Ok(Self {
            pair: pair,
            time: time,
            rate: rate,
            tx_hash: bytes[24..56].into(),
        })
    }
}

#[derive(Debug)]
pub struct LockTokenEvent {
    pub id: U256,
    pub sender: Address,
    pub beneficiary: H256,
    pub value: U256,
    pub cycle: U256,
    pub tx_hash: H256,
}

impl LockTokenEvent {
    pub fn from_log(raw_log: &Log) -> Result<Self, Error> {
        let hash = raw_log
            .transaction_hash
            .ok_or_else(|| "`log` must be mined and contain `transaction_hash`")?;
        let mut log = contracts::mapper::events::lock_token::parse_log(raw_log.into_raw_log())?;
        Ok(Self {
            id: log.id,
            sender: log.sender,
            beneficiary: log.beneficiary,
            value: log.amount,
            cycle: log.cycle,
            tx_hash: hash,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![0u8; LOCKTOKEN_LENGTH];
        self.id.to_big_endian(&mut result[0..32]);
        result[32..52].copy_from_slice(&self.sender.0[..]);
        result[52..84].copy_from_slice(&self.beneficiary.0[..]);
        self.value.to_big_endian(&mut result[84..116]);
        self.cycle.to_big_endian(&mut result[116..148]);
        result[148..LOCKTOKEN_LENGTH].copy_from_slice(&self.tx_hash.0[..]);
        return result;
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != LOCKTOKEN_LENGTH {
            bail!("`bytes`.len() must be {}", LOCKTOKEN_LENGTH);
        }

        Ok(Self {
            id: U256::from_big_endian(&bytes[0..32]),
            sender: bytes[32..52].into(),
            beneficiary: bytes[52..84].into(),
            value: U256::from_big_endian(&bytes[84..116]),
            cycle: U256::from_big_endian(&bytes[116..148]),
            tx_hash: bytes[148..LOCKTOKEN_LENGTH].into(),
        })
    }
}

#[derive(Debug)]
pub struct UnlockTokenEvent {
    pub id: U256,
    pub tx_hash: H256,
}

impl UnlockTokenEvent {
    pub fn from_log(raw_log: &Log) -> Result<Self, Error> {
        let hash = raw_log
            .transaction_hash
            .ok_or_else(|| "`log` must be mined and contain `transaction_hash`")?;
        let mut log = contracts::mapper::events::unlock_token::parse_log(raw_log.into_raw_log())?;
        Ok(Self {
            id: log.id,
            tx_hash: hash,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![0u8; UNLOCKTOKEN_LENGTH];
        self.id.to_big_endian(&mut result[0..32]);
        result[32..UNLOCKTOKEN_LENGTH].copy_from_slice(&self.tx_hash.0[..]);
        return result;
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != UNLOCKTOKEN_LENGTH {
            bail!("`bytes`.len() must be {}", UNLOCKTOKEN_LENGTH);
        }

        Ok(Self {
            id: U256::from_big_endian(&bytes[0..32]),
            tx_hash: bytes[32..UNLOCKTOKEN_LENGTH].into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::{FromHex, ToHex};
    use web3::types::Bytes;

    fn prepare_data() -> (H256, Address, U256, H256, &'static str) {
        let tag: H256 = "0x0000000000000000000000000000000000000000000000000000000000000002".into();
        let recipient: Address = "0x74241db5f3ebaeecf9506e4ae988186093341604".into();
        // 0x00000000000000000000000000000000000000000000000000000000054c5638
        let value: U256 = U256::from_dec_str("88888888").unwrap();
        let tx_hash: H256 =
            "0x1045bfe274b88120a6b1e5d01b5ec00ab5d01098346e90e7c7a3c9b8f0181c80".into();
        let bytes_str: &'static str = "000000000000000000000000000000000000000000000000000000000000000274241db5f3ebaeecf9506e4ae98818609334160400000000000000000000000000000000000000000000000000000000054c56381045bfe274b88120a6b1e5d01b5ec00ab5d01098346e90e7c7a3c9b8f0181c80";
        (tag, recipient, value, tx_hash, bytes_str)
    }

    #[test]
    fn test_message_to_bytes() {
        let (tag, recipient, value, tx_hash, bytes_str) = prepare_data();

        let message = IngressEvent {
            tag: tag,
            recipient: recipient,
            value: value,
            tx_hash: tx_hash,
        };
        println!("{}", message.to_bytes().to_hex());
        assert_eq!(message.to_bytes().to_hex(), bytes_str);
    }

    #[test]
    fn test_message_from_bytes() {
        let (tag, recipient, value, tx_hash, bytes_str) = prepare_data();

        let message = IngressEvent::from_bytes(bytes_str.from_hex().unwrap().as_slice()).unwrap();
        assert_eq!(message.tag, tag);
        assert_eq!(message.recipient, recipient);
        assert_eq!(message.value, value);
        assert_eq!(message.tx_hash, tx_hash);
    }

    #[test]
    fn test_message_from_log() {
        let (tag, recipient, value, tx_hash, _bytes_str) = prepare_data();
        let ingress_topic = contracts::bridge::events::ingress::filter().topic0;
        let log = Log {
                    address: "0xf1dF5972B7e394201d4fFADD797FAa4A3C8be0ea".into(),
                    topics: ingress_topic.into(),
                    data: Bytes("000000000000000000000000000000000000000000000000000000000000000200000000000000000000000074241db5f3ebaeecf9506e4ae98818609334160400000000000000000000000074241db5f3ebaeecf9506e4ae98818609334160400000000000000000000000000000000000000000000000000000000054c5638".from_hex().unwrap()),
                    transaction_hash: Some(tx_hash),
                    block_hash: None,
                    block_number: None,
                    transaction_index: None,
                    log_index: None,
                    transaction_log_index: None,
                    log_type: None,
                    removed: None,
                };
        let message = IngressEvent::from_log(&log).unwrap();
        assert_eq!(message.tag, tag);
        assert_eq!(message.recipient, recipient);
        assert_eq!(message.value, value);
    }
}
