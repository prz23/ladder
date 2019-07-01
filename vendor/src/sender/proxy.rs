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
    fn initialize(&mut self, event_loop: &mut Core, transport: &impl Transport);
    fn send(&mut self, event_loop: &mut Core, transport: &impl Transport, payload: Vec<u8>);
}

pub struct EthProxy {
    pub pair: KeyPair,
    pub context: SignContext,
}

impl SenderProxy for EthProxy {
    fn initialize(&mut self, event_loop: &mut Core, transport: &impl Transport) {
        let authority_address: Address = self.pair.address();
        let nonce = event_loop
            .run(web3::api::Eth::new(&transport).transaction_count(authority_address, None))
            .unwrap();
        info!("eth nonce: {}", nonce);
        self.context.nonce = nonce;
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
}

impl SenderProxy for AbosProxy {
    fn initialize(&mut self, event_loop: &mut Core, transport: &impl Transport) {
        let authority_address: Address = self.pair.address();
        let nonce = event_loop
            .run(web3::api::Abos::new(&transport).transaction_count(authority_address, None))
            .unwrap();
        info!("abos nonce: {}", nonce);
        self.context.nonce = nonce;
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
            chain_id: U256::from(1),
            version: 1,
        };
        let data = signer::Abos::sign_transaction(self.pair.privkey(), &transaction);

        let future = web3::api::Abos::new(&transport).send_raw_transaction(Bytes::from(data));
        let hash = event_loop.run(future).unwrap();
        info!("send to Abos transaction hash: {:?}", hash);
        self.context.nonce += 1.into();
    }
}
