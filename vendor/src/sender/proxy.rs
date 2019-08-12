use log::{error, info};
use signer::{AbosTransaction, EthTransaction, KeyPair};
use tokio_core::reactor::Core;
use web3::{
    api::Namespace,
    types::{Address, Bytes, U256},
    Transport,
};

pub struct SignContext {
    pub height: u64,
    pub nonce: U256,
    pub contract_address: Address,
}

pub trait SenderProxy {
    fn initialize(&mut self, event_loop: &mut Core, transport: &impl Transport) -> Result<(), String>;
    fn send(&mut self, event_loop: &mut Core, transport: &impl Transport, payload: Vec<u8>);
}

pub struct EthProxy {
    pub pair: KeyPair,
    pub context: SignContext,
}

impl SenderProxy for EthProxy {
    fn initialize(&mut self, event_loop: &mut Core, transport: &impl Transport) -> Result<(), String>{
        let authority_address: Address = self.pair.address();
        let nonce = event_loop
            .run(web3::api::Eth::new(&transport).transaction_count(authority_address, None));
        match nonce {
            Ok(n) => {
                info!("eth nonce: {}", n);
                self.context.nonce = n;
                Ok(())
            }
            Err(e) => {
                Err(format!("{}", e))
            }
        }
    }

    fn send(&mut self, event_loop: &mut Core, transport: &impl Transport, payload: Vec<u8>) {
        let transaction = EthTransaction {
            nonce: self.context.nonce.into(),
            to: Some(self.context.contract_address),
            value: 0.into(),
            data: payload,
            gas_price: 1000000000.into(),
            gas: 200000.into(),
        };
        let data = signer::Eth::sign_transaction(self.pair.privkey(), &transaction);

        let future = web3::api::Eth::new(&transport).send_raw_transaction(Bytes::from(data));
        match event_loop.run(future) {
            Ok(hash) => {
                info!("send to eth transaction hash: {:?}", hash);
                self.context.nonce += 1.into();
            }
            Err(e) => error!("send to eth error! case:{:?}", e),
        }
    }
}

pub struct AbosProxy {
    pub pair: KeyPair,
    pub context: SignContext,
    pub chain_id: U256,
}

impl SenderProxy for AbosProxy {
    fn initialize(&mut self, event_loop: &mut Core, transport: &impl Transport) -> Result<(), String>{
        let authority_address: Address = self.pair.address();
        let nonce = event_loop
            .run(web3::api::Abos::new(&transport).transaction_count(authority_address, None));
        match nonce {
            Ok(n) => {
                info!("eth nonce: {}", n);
                self.context.nonce = n;
                Ok(())
            }
            Err(e) => {
                Err(format!("{}", e))
            }
        }
    }

    fn send(&mut self, event_loop: &mut Core, transport: &impl Transport, payload: Vec<u8>) {
        let height: u64 = event_loop
            .run(web3::api::Abos::new(&transport).block_number())
            .unwrap()
            .into();
        let transaction = AbosTransaction {
            to: Some(self.context.contract_address),
            nonce: self.context.nonce.to_string(),
            quota: 210000,
            valid_until_block: height + 88,
            data: payload,
            value: U256::from(0),
            chain_id: self.chain_id,
            version: 2,
        };
        let data = signer::Abos::sign_transaction(self.pair.privkey(), &transaction);

        let future = web3::api::Abos::new(&transport).send_raw_transaction(Bytes::from(data));
        match event_loop.run(future) {
            Ok(hash) => {
                info!("send to Abos transaction hash: {:?}", hash);
                self.context.nonce += 1.into();
            }
            Err(e) => error!("send to abos error! case:{:?}", e),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio_core::reactor::Core;
    use std::str::FromStr;
    use web3::types::H256;
    use signer::{KeyPair, PrivKey};

    fn get_key() -> KeyPair {
        let privkey = PrivKey::from(
            H256::from_str("2f2f416c69636508080808080808080808080808080808080808080808080808")
                .unwrap(),
        );
        let pair = KeyPair::from_privkey(privkey);
        pair
    }

    #[test]
    fn eth_proxy_test() {
        let mut proxy = EthProxy {
            pair: get_key(),
            context: SignContext {
                height: 0,
                nonce: U256::from(0),
                contract_address: Address::from_str("E68ccBF91ad45C4249b877f1B9f8224179e1a126").unwrap(),
            }
        };

        let mut event_loop = Core::new().unwrap();
        let transport = web3::transports::Http::with_event_loop(
                &"https://kovan.infura.io/v3/838099c0002948caa607f6cfb29a4816",
                &event_loop.handle(),
                4,
            ).unwrap();

        proxy.initialize(&mut event_loop,& transport);

        proxy.send(&mut event_loop,& transport, vec![]);
    }

    #[test]
    fn abos_proxy_test() {
        let mut proxy = AbosProxy {
            pair: get_key(),
            chain_id: U256::from_str("00000000000000000000000000000000000000000000ca812def6446350c7e8d").unwrap(),
            context: SignContext {
                height: 0,
                nonce: U256::from(0),
                contract_address: Address::from_str("9dCA5E2eDE0a6848bE3d24EFB4D871a53786A1d2").unwrap(),
            }
        };

        let mut event_loop = Core::new().unwrap();
        let transport = web3::transports::Http::with_event_loop(
                &"http://47.92.173.78:1337",
                &event_loop.handle(),
                4,
            ).unwrap();

        proxy.initialize(&mut event_loop,& transport);

        proxy.send(&mut event_loop,& transport, vec![]);
    }
}