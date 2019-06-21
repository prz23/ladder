#[macro_use]
mod macros;
pub mod error;
pub mod events;
mod exchange;
pub mod label;
mod listener;
pub mod message;
mod sender;
mod state;
mod streams;
mod supervisor;
mod utils;
mod vendor;
mod mapper;

use crate::label::ChainAlias;
use crate::listener::{SideListener, ListenerStreamStyle};
use crate::sender::{AbosProxy, EthProxy, SideSender, SignContext};
use crate::supervisor::{PacketNonce, Supervisor};
use client::{blockchain::HeaderBackend, BlockchainEvents};
use exchange::Exchange;
use futures::{Future, Stream};
use keystore::Store as Keystore;
use log::{debug, error, info, trace, warn};
use network::SyncProvider;
use node_runtime::{
    matrix::*, Event, EventRecord,
    VendorApi, /*,exchangerate */
};
use primitives::storage::{StorageChangeSet, StorageData, StorageKey};
use primitives::{crypto::Ss58Codec, crypto::*};
use runtime_primitives::{
    codec::{Decode},
    generic::{BlockId},
    traits::{Block, BlockNumberToHash, ProvideRuntimeApi},
};
use rustc_hex::ToHex;
use signer::{KeyPair, PrivKey};
use std::path::{Path};
use std::str::FromStr;
use std::sync::{
    Arc, Mutex,
};
use transaction_pool::txpool::{self, Pool as TransactionPool};
use web3::{
    types::{Address, H256, U256},
};

#[cfg_attr(test, macro_use)]
mod test;
#[cfg(test)]
pub use crate::test::{MockClient, MockTransport};

#[derive(Clone)]
pub struct RunStrategy {
    pub listener: bool,
    pub sender: bool,
    pub enableexchange: bool,
}

#[derive(Clone)]
pub struct VendorServiceConfig {
    pub kovan_url: String,
    pub abos_url: String,
    pub kovan_address: String,
    pub abos_address: String,
    pub eth_url: String,
    pub mapper_address: String,
    pub db_path: String,
    pub eth_key: String,
    pub strategy: RunStrategy,
}

/// Start the supply worker. The returned future should be run in a tokio runtime.
pub fn start_vendor<A, B, C, N>(
    config: VendorServiceConfig,
    network: Arc<N>,
    client: Arc<C>,
    pool: Arc<TransactionPool<A>>,
    keystore: &Keystore,
    on_exit: impl Future<Item = (), Error = ()>,
) -> impl Future<Item = (), Error = ()>
where
    A: txpool::ChainApi<Block = B> + 'static,
    B: Block + 'static,
    C: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi + 'static,
    N: SyncProvider<B> + 'static,
    C::Api: VendorApi<B>,
{
    let key = keystore.load(&keystore.contents().unwrap()[0], "").unwrap();
    let key2 = keystore.load(&keystore.contents().unwrap()[0], "").unwrap();
    let kovan_address = Address::from_str(&config.kovan_address).unwrap();
    let abos_address = Address::from_str(&config.abos_address).unwrap();
    let mapper_address = Address::from_str(&config.mapper_address).unwrap();
    let eth_key = PrivKey::from_str(&config.eth_key).unwrap();
    let eth_pair = KeyPair::from_privkey(eth_key);
    info!(
        "ss58 account: {:?}, eth account: {}",
        key.public().to_ss58check(),
        eth_pair
    );
    let info = client.info().unwrap();
    let at = BlockId::Hash(info.best_hash);
    let packet_nonce = PacketNonce {
        nonce: client
            .runtime_api()
            .account_nonce(&at, &key.public().0.unchecked_into())
            .unwrap(),
        last_block: at,
    };
    let packet_nonce2 = PacketNonce {
        nonce: client
            .runtime_api()
            .account_nonce(&at, &key.public().0.unchecked_into())
            .unwrap(),
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
        enable: config.strategy.listener,
        chain: ChainAlias::ETH,
        style: ListenerStreamStyle::Vendor,
    }
    .start();

    //new a thread to listen abos network
    SideListener {
        url: config.abos_url.clone(),
        db_file: Path::new(&config.db_path).join("abos_storage.json"),
        contract_address: abos_address,
        spv: spv.clone(),
        enable: config.strategy.listener,
        chain: ChainAlias::ABOS,
        style: ListenerStreamStyle::Vendor,
    }
    .start();

    // listen mapper information from eth
    SideListener {
        url: config.eth_url.clone(),
        db_file: Path::new(&config.db_path).join("eth_mapper_storage.json"),
        contract_address: mapper_address,
        spv: spv.clone(),
        enable: config.strategy.listener,
        chain: ChainAlias::ETH,
        style: ListenerStreamStyle::Mapper,
    }
    .start();

    // A thread that send transaction to ETH
    let kovan_sender = SideSender {
        name: "ETH-kovan".to_string(),
        url: config.kovan_url.clone(),
        contract_address: kovan_address,
        pair: eth_pair.clone(),
        enable: config.strategy.sender,
        proxy: EthProxy {
            pair: eth_pair.clone(),
            context: SignContext {
                height: 0,
                nonce: U256::from(0),
                contract_address: kovan_address,
            },
        },
    }
    .start();

    let abos_sender = SideSender {
        name: "ABOS-test".to_string(),
        url: config.abos_url.clone(),
        contract_address: abos_address,
        pair: eth_pair.clone(),
        enable: config.strategy.sender,
        proxy: AbosProxy {
            pair: eth_pair.clone(),
            context: SignContext {
                height: 0,
                nonce: U256::from(0),
                contract_address: abos_address,
            },
        },
    }
    .start();

    let id_need = key2.public().0.unchecked_into();

    if config.strategy.enableexchange {
        // exchange
        let _ext = Exchange {
            client: client.clone(),
            pool: pool.clone(),
            accountid: id_need,
            packet_nonce: Arc::new(Mutex::new(packet_nonce2)),
            spv: spv.clone(),
        }
        .start();
    }

    let eth_kovan_tag = H256::from_str(events::ETH_COIN).unwrap();
    let abos_tag = H256::from_str(events::ABOS_COIN).unwrap();
    // how to fetch real key?
    let events_key = StorageKey(primitives::twox_128(b"System Events").to_vec());
    let storage_stream = client
        .storage_changes_notification_stream(Some(&[events_key]))
        .unwrap()
        .map(|(block, changes)| StorageChangeSet {
            block,
            changes: changes.iter().cloned().collect(),
        })
        .for_each(move |change_set| {
            let records: Vec<Vec<EventRecord<Event>>> = change_set
                .changes
                .iter()
                .filter_map(|(_, mbdata)| {
                    if let Some(StorageData(data)) = mbdata {
                        Decode::decode(&mut &data[..])
                    } else {
                        None
                    }
                })
                .collect();
            let events: Vec<Event> = records.concat().iter().cloned().map(|r| r.event).collect();
            events.iter().for_each(|event| {
                if let Event::matrix(e) = event {
                    match e {
                        RawEvent::Ingress(message, signatures) => {
                            info!(
                                "raw event ingress: message: {}, signatures: {}",
                                message.to_hex(),
                                signatures.to_hex()
                            );
                            events::IngressEvent::from_bytes(message)
                                .map(|ie| {
                                    if ie.tag[24..32] == eth_kovan_tag[24..32] {
                                        kovan_sender.send(e.clone()).unwrap();
                                    } else if ie.tag[24..32] == abos_tag[24..32] {
                                        abos_sender.send(e.clone()).unwrap();
                                    } else {
                                        warn!("unknown event tag of ingress: {:?}", ie.tag);
                                    }
                                })
                                .map_err(|_err| {
                                    warn!(
                                        "unexpected format of ingress, message {:?}",
                                        message.to_hex()
                                    );
                                });
                        }
                        _ => {}
                    };
                }
            });
            Ok(())
        });

    storage_stream.map(|_| ()).select(on_exit).then(|_| Ok(()))
}
