#[macro_use]
extern crate error_chain;
extern crate toml;
extern crate tokio_timer;
#[macro_use]
extern crate futures;
extern crate web3;
extern crate tokio;
extern crate tokio_core;
#[macro_use]
extern crate log;

#[cfg_attr(test, macro_use)]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
extern crate contracts;
extern crate signer;
extern crate ethabi;
extern crate rustc_hex;

extern crate node_runtime;
extern crate sr_primitives as runtime_primitives;
extern crate substrate_client as client;
extern crate substrate_network as network;
extern crate substrate_keystore as keystore;
extern crate substrate_primitives as primitives;
extern crate substrate_transaction_pool as transaction_pool;


#[cfg(test)]
#[macro_use]
mod test;
#[cfg(test)]
extern crate jsonrpc_core;
#[cfg(test)]
pub use test::{MockTransport, MockClient};


#[macro_use]
mod macros;
pub mod error;
//mod fixed_number;
pub mod log_stream;
pub mod block_number_stream;
pub mod vendor;
pub mod events;
pub mod message;
mod state;
mod utils;

use std::str::FromStr;
use message::{RelayMessage,RelayType};
use tokio_core::reactor::Core;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::path::{Path, PathBuf};
use error::{ResultExt};
use vendor::Vendor;
use signer::{SecretKey, RawTransaction, KeyPair, PrivKey};
use state::{StateStorage};
use network::SyncProvider;
use futures::{Future, Stream};
use keystore::Store as Keystore;
use runtime_primitives::codec::{Decode, Encode, Compact};
use runtime_primitives::generic::{BlockId, Era};
use runtime_primitives::traits::{Block, BlockNumberToHash, ProvideRuntimeApi};
use client::{runtime_api::Core as CoreApi, BlockchainEvents, blockchain::HeaderBackend};
use primitives::storage::{StorageKey, StorageData, StorageChangeSet};
use primitives::{Pair as TraitPait, ed25519::Pair};
use transaction_pool::txpool::{self, Pool as TransactionPool, ExtrinsicFor};
use node_runtime::{
    Call, UncheckedExtrinsic, EventRecord, Event,MatrixCall, BankCall, matrix::*, VendorApi
};
use node_runtime::{Balance, Hash, AccountId, Index, BlockNumber};
use web3::{
    api::Namespace, 
    types::{Address, Bytes, H256},
};
use std::marker::{Send, Sync};

const MAX_PARALLEL_REQUESTS: usize = 10;

pub trait SuperviseClient{
    fn submit(&self, message: RelayMessage);
}

pub struct PacketNonce<B> where B: Block{
    pub nonce: Index, // to control nonce.
    pub last_block: BlockId<B>,
}

pub struct Supervisor<A, B, C, N> where
    A: txpool::ChainApi,
    B: Block
{
    pub client: Arc<C>,
    pub pool: Arc<TransactionPool<A>>,
    pub network: Arc<N>,
    pub key: Pair,
    pub eth_key: SecretKey,
    pub phantom: std::marker::PhantomData<B>,
    // pub queue: Vec<(RelayMessage, u8)>,
    pub packet_nonce: Arc<Mutex<PacketNonce<B>>>,
}

impl<A, B, C, N> Supervisor<A, B, C, N> where
    A: txpool::ChainApi<Block = B>,
    B: Block,
    C: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi,
    N: SyncProvider<B>,
    C::Api: VendorApi<B>
{
    /// get nonce with atomic
    fn get_nonce(&self) -> Index {
        let mut p_nonce = self.packet_nonce.lock().unwrap();
        let info = self.client.info().unwrap();
        let at = BlockId::Hash(info.best_hash);
        if p_nonce.last_block == at {
            p_nonce.nonce = p_nonce.nonce + 1;
        } else {
            p_nonce.nonce = self.client.runtime_api().account_nonce(&at, self.key.public()).unwrap();
            p_nonce.last_block = at;
        }

        p_nonce.nonce
    }
}

impl<A, B, C, N> SuperviseClient for Supervisor<A, B, C, N> where
    A: txpool::ChainApi<Block = B>,
    B: Block,
    C: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi,
    N: SyncProvider<B>,
    C::Api: VendorApi<B> + CoreApi<B>
{
    fn submit(&self, message: RelayMessage) {
        let local_id: AccountId = self.key.public();
        let info = self.client.info().unwrap();
        let at = BlockId::Hash(info.best_hash);
        // let auths = self.client.runtime_api().authorities(&at).unwrap();
        // if auths.contains(&AuthorityId::from(self.key.public().0)) {
        if self.client.runtime_api().is_authority(&at, &self.key.public()).unwrap() {
            let nonce = self.get_nonce();
            let signature = signer::sign_message(&self.eth_key, &message.raw).into();

            let function =  match message.ty {
                    RelayType::Ingress => Call::Matrix(MatrixCall::ingress(message.raw, signature)),
                    RelayType::Egress => Call::Matrix(MatrixCall::egress(message.raw, signature)),
                    RelayType::Deposit => Call::Bank(BankCall::deposit(message.raw, signature)),
                    RelayType::Withdraw => Call::Bank(BankCall::withdraw(message.raw, signature)),
                    RelayType::SetAuthorities => Call::Matrix(MatrixCall::reset_authorities(message.raw, signature)),
                };

            let payload = (
                Compact::<Index>::from(nonce),  // index/nonce
                function, //function
                Era::immortal(),  
                self.client.genesis_hash(),
            );
            
            let signature = self.key.sign(&payload.encode());
            let extrinsic = UncheckedExtrinsic::new_signed(
                payload.0.into(),
                payload.1,
                local_id.into(),
                signature.into(),
                payload.2
            );

            let xt: ExtrinsicFor<A> = Decode::decode(&mut &extrinsic.encode()[..]).unwrap();
            println!("extrinsic {:?}", xt);
            println!("@submit transaction {:?}",self.pool.submit_one(&at, xt));
        }
    }
}

#[derive(Clone)]
pub struct VendorServiceConfig {
    pub kovan_url: String,
    pub ropsten_url: String,
    pub kovan_address: String,
    pub ropsten_address: String,
    pub db_path: String,
    pub eth_key: String,
}

pub struct SideListener<V> {
    pub url: String,
    pub contract_address: Address,
    pub db_file: PathBuf,
    pub spv: Arc<V>,
}

fn print_err(err: error::Error) {
    let message = err
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n\nCaused by:\n  ");
    error!("{}", message);
}

fn is_err_time_out(err: &error::Error) -> bool {
    err.iter().any(|e| e.to_string() == "Request timed out")
}

impl<V> SideListener<V> where 
    V: SuperviseClient + Send + Sync + 'static
{
    fn start(self) {
        // TODO hook the event of http disconnect to keep run.
        std::thread::spawn(move ||{
            let mut event_loop = Core::new().unwrap();
            loop {
                let transport = web3::transports::Http::with_event_loop(
                        &self.url,
                        &event_loop.handle(),
                        MAX_PARALLEL_REQUESTS,
                    )
                    .chain_err(|| {format!("Cannot connect to ethereum node at {}", self.url)}).unwrap();

                if !self.db_file.exists() {
                    std::fs::File::create(&self.db_file).expect("failed to create the storage file of state.");
                }
                let mut storage = StateStorage::load(self.db_file.as_path()).unwrap();
                let vendor = Vendor::new(&transport, self.spv.clone(), storage.state.clone(), self.contract_address)
                                    .and_then(|state| {
                                        storage.save(&state)?;
                                        Ok(())
                                    })
                                    .for_each(|_| Ok(()));
                match event_loop.run(vendor) {
                    Ok(s) => {
                        info!("{:?}", s);
                        break;
                    }
                    Err(err) => {
                        if is_err_time_out(&err) {
                            error!("\nreqeust time out sleep 5s and try again.\n");
                            std::thread::sleep_ms(5000);
                        } else {
                            print_err(err);
                        }
                    }
                }
            }
        });
    }
}

struct SideSender {
    name: String,
    url: String,
    contract_address: Address,
    pair: KeyPair,
}

impl SideSender {
    fn start(self) -> Sender<RawEvent<Balance, AccountId, Hash, BlockNumber>> {
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            let mut event_loop = Core::new().unwrap();
            let transport = web3::transports::Http::with_event_loop(
                    &self.url,
                    &event_loop.handle(),
                    MAX_PARALLEL_REQUESTS,
                )
                .chain_err(|| {format!("Cannot connect to ethereum node at {}", self.url)}).unwrap();

            // get nonce
            let authority_address: Address = self.pair.address();
            let nonce_future  = web3::api::Eth::new(&transport).transaction_count(authority_address, None);
            let mut nonce = event_loop.run(nonce_future).unwrap();
            info!("eth nonce: {}", nonce);
            loop {
                let event = receiver.recv().unwrap();
                let data = match event {
                    RawEvent::Ingress(message, signatures) => {
                        info!("ingress message: {:?}, signatures: {:?}", message, signatures);
                        let payload = contracts::bridge::functions::release::encode_input(message, signatures);
                        Some(payload)
                    },
                    _ => {
                        None
                    }
                };
                if let Some(payload) = data {
                    let transaction = RawTransaction {
                                    nonce: nonce.into(),
                                    to: Some(self.contract_address),
                                    value: 0.into(),
                                    data: payload,
                                    gas_price: 2000000000.into(),
                                    gas: 41000.into(),
                                };
                    let sec: &SecretKey = unsafe { std::mem::transmute(self.pair.privkey()) };
                    let data = signer::sign_transaction(&sec, &transaction);
                    let future = web3::api::Eth::new(&transport).send_raw_transaction(Bytes::from(data));
                    let hash = event_loop.run(future).unwrap();
                    info!("send to eth transaction hash: {:?}", hash);
                    nonce += 1.into();
                }
            }
        });

        sender
    }
}

/// Start the supply worker. The returned future should be run in a tokio runtime.
pub fn start_vendor<A, B, C, N>(
    config: VendorServiceConfig,
    network: Arc<N>,
    client: Arc<C>,
    pool: Arc<TransactionPool<A>>,
    keystore: &Keystore,
    on_exit: impl Future<Item=(),Error=()>,
) -> impl Future<Item=(),Error=()> where
    A: txpool::ChainApi<Block = B> + 'static,
    B: Block + 'static,
    C: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi + 'static,
    N: SyncProvider<B> + 'static,
    C::Api: VendorApi<B>
{
    let key = keystore.load(&keystore.contents().unwrap()[0], "").unwrap();
    let kovan_address = Address::from_str(&config.kovan_address).unwrap();
    let ropsten_address = Address::from_str(&config.ropsten_address).unwrap();
    let eth_key = SecretKey::from_str(&config.eth_key).unwrap();
    let eth_pair = KeyPair::from_privkey(PrivKey::from_slice(&eth_key[..]));
    info!("ss58 account: {:?}, eth account: {}", key.public().to_ss58check(), eth_pair);

    let info = client.info().unwrap();
    let at = BlockId::Hash(info.best_hash);
    let packet_nonce = PacketNonce {
            nonce: client.runtime_api().account_nonce(&at, key.public()).unwrap(),
            last_block: at,
        };

    let spv = Arc::new(Supervisor {
        client: client.clone(),
        pool: pool.clone(),
        network: network.clone(),
        key: key,
        eth_key: eth_key.clone(),
        packet_nonce: Arc::new(Mutex::new(packet_nonce)),
        phantom: std::marker::PhantomData,
    });
    
    //new a thread to listen kovan network
    SideListener {
        url: config.kovan_url.clone(), 
        db_file: Path::new(&config.db_path).join("kovan_storage.json"),
        contract_address: kovan_address,
        spv: spv.clone(),
    }.start();

    //new a thread to listen ropsten network
    SideListener {
        url: config.ropsten_url.clone(),
        db_file:  Path::new(&config.db_path).join("ropsten_storage.json"),
        contract_address: ropsten_address,
        spv: spv.clone(),
    }.start();

    // A thread that send transaction to ETH
    let kovan_sender = SideSender {
        name: "ETH_Kovan".to_string(),
        url: config.kovan_url.clone(),
        contract_address: kovan_address,
        pair: eth_pair.clone(),
    }.start();
    
    let ropsten_sender = SideSender {
        name: "ETH_Ropsten".to_string(),
        url: config.ropsten_url.clone(),
        contract_address: ropsten_address,
        pair: eth_pair.clone(),
    }.start();

    let eth_kovan_tag = H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let eth_ropsten_tag = H256::from_str("0000000000000000000000000000000000000000000000000000000000000002").unwrap();
    // how to fetch real key?
    let events_key = StorageKey(primitives::twox_128(b"System Events").to_vec());
    let storage_stream = client.storage_changes_notification_stream(Some(&[events_key])).unwrap()
    .map(|(block, changes)| StorageChangeSet { block, changes: changes.iter().cloned().collect()})
    .for_each(move |change_set| {
        let records: Vec<Vec<EventRecord<Event>>> = change_set.changes
            .iter()
            .filter_map(|(_, mbdata)| if let Some(StorageData(data)) = mbdata {
                Decode::decode(&mut &data[..])
            } else { None })
            .collect();
        let events: Vec<Event> = records
            .concat()
            .iter()
            .cloned()
            .map(|r| r.event)
            .collect();
        events.iter().for_each(|event| {
            if let Event::matrix(e) = event {
                match e {
                    RawEvent::Ingress(message, signatures) => {
                        println!("raw event ingress: {:?}, {:?}", message, signatures);
                        events::IngressEvent::from_bytes(message).map(|ie| {
                            if ie.tag == eth_kovan_tag {
                                kovan_sender.send(e.clone()).unwrap();
                            } else if ie.tag == eth_ropsten_tag {
                                ropsten_sender.send(e.clone()).unwrap();
                            } else {
                                warn!("unknown event tag of ingress: {:?}", ie.tag);
                            }
                        }).map_err(|err| {
                            warn!("unexpected format of ingress, message {:?}", message);
                        });
                    },
                    _ => {}
                };
            }
        });
        Ok(())
    });

    storage_stream
            .map(|_|())
            .select(on_exit)
            .then(|_| {Ok(())})
}
