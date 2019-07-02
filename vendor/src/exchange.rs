use crate::events;
use crate::message::RelayMessage;
use crate::supervisor::{PacketNonce, SuperviseClient};
use chrono::prelude::*;
use client::{blockchain::HeaderBackend, BlockchainEvents};
use curl::easy::{Easy2, Handler, WriteError};
use node_runtime::VendorApi;
use primitives::ed25519::Public;
use runtime_primitives::{
    generic::BlockId,
    traits::{Block, BlockNumberToHash, ProvideRuntimeApi},
};
use rustc_serialize::json;
use std::marker::{Send, Sync};
use std::str::FromStr;
use std::sync::{
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
use transaction_pool::txpool::{self, Pool as TransactionPool};
use web3::{types::H256};

#[derive(RustcDecodable, RustcEncodable)]
pub struct exchange_rate {
    ticker: String,
    exchangeName: String,
    base: String,
    currency: String,
    symbol: String,
    high: f64,
    open: f64,
    close: f64,
    low: f64,
    vol: f64,
    degree: f64,
    value: f64,
    changeValue: f64,
    commissionRatio: f64,
    quantityRatio: f64,
    turnoverRate: f64,
    dateTime: u64,
}

fn parse_exchange_rate(content: String) -> (f64, u64) {
    // Deserialize using `json::decode`
    let decoded: exchange_rate = json::decode(&content).unwrap();
    println!("exchange_rate {:?}", decoded.close);
    println!("exchange_rate time {:?}", decoded.dateTime);
    (decoded.close, decoded.dateTime)
}
// let local_id: AccountId = self.key.public().0.into();
pub trait ExchangeTrait {
    fn check_validators(&self) -> bool;
}

impl<A, B, Q, V> ExchangeTrait for Exchange<A, B, Q, V>
where
    A: txpool::ChainApi<Block = B> + 'static,
    B: Block,
    Q: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi + 'static,
    Q::Api: VendorApi<B>,
    V: SuperviseClient + Send + Sync + 'static,
{
    fn check_validators(&self) -> bool {
        let info = self.client.info().unwrap();
        let at = BlockId::Hash(info.best_hash);
        let accountid = &self.accountid;
        self.client
            .runtime_api()
            .is_authority(&at, accountid)
            .unwrap()
    }
}

struct Collector(Vec<u8>);
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

pub struct Exchange<A, B, C, V>
where
    A: txpool::ChainApi,
    B: Block,
    V: SuperviseClient + Send + Sync + 'static,
{
    pub client: Arc<C>,
    pub pool: Arc<TransactionPool<A>>,
    pub accountid: Public,
    //pub phantom: std::marker::PhantomData<B>,
    pub packet_nonce: Arc<Mutex<PacketNonce<B>>>,
    pub spv: Arc<V>,
}

/// oracle
impl<A, B, Q, V> Exchange<A, B, Q, V>
where
    A: txpool::ChainApi<Block = B> + 'static,
    B: Block,
    Q: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi + 'static,
    Q::Api: VendorApi<B>,
    V: SuperviseClient + Send + Sync + 'static,
{
    pub fn start(self) {
        std::thread::spawn(move || {
            loop {
                let mut timestamp = Utc::now().timestamp() as u64;
                if timestamp % 120 == 0 {
                    self.get_exchange_rate(
                        "http://api.coindog.com/api/v1/tick/BITFINEX:ETHUSD?unit=cny",
                        1,
                        timestamp,
                    );
                    self.get_exchange_rate(
                        "http://api.coindog.com/api/v1/tick/BITFINEX:XRPUSD?unit=cny",
                        2,
                        timestamp,
                    );
                }
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    fn get_exchange_rate(&self, url: &str, extype: u64, timestamp: u64) {
        let mut easy = Easy2::new(Collector(Vec::new()));
        easy.get(true).unwrap();
        easy.url(url).unwrap();

        if self.check_validators() {
            // perform the http get method and fetch the responsed
            if easy.perform().is_err() {
                println!("err");
            }

            if easy.response_code().unwrap() == 200 {
                let contents = easy.get_ref();
                let contents_string = String::from_utf8_lossy(&contents.0).to_string();
                let (exchange_rate, _time) = parse_exchange_rate(contents_string);
                let hash = H256::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000001",
                )
                .unwrap();
                let message = events::ExchangeRateEvent {
                    pair: extype,
                    time: timestamp,
                    rate: (exchange_rate * 100000.0f64) as u64,
                    tx_hash: hash,
                };
                self.spv.submit(RelayMessage::from(message));
            } else {
                println!("failed get info");
            }
        };
    }
}
