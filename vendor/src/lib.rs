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
mod config;

pub use crate::config::ServiceConfig;

use crate::label::ChainAlias;
use crate::listener::{SideListener};
use crate::config::{ServiceConfig as VendorServiceConfig, ListenerStreamStyle, EngineKind};
use crate::sender::{AbosProxy, EthProxy, SideSender, SignContext, SenderProxy};
use crate::supervisor::{PacketNonce, Supervisor, SuperviseClient};
use crate::message::RelayMessage;
use client::{blockchain::HeaderBackend, BlockchainEvents};
use exchange::Exchange;
use futures::{Future, Stream};
use keystore::Store as Keystore;
use log::{error, info, warn};
use network::SyncProvider;
use node_runtime::{
    Event, EventRecord, order::RawEvent,
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
    Arc, Mutex, mpsc::Sender,
};
use node_primitives::AccountId;
use std::collections::HashMap;
use transaction_pool::txpool::{self, Pool as TransactionPool};
use web3::{
    types::{Address, H256, U256},
};

#[cfg_attr(test, macro_use)]
mod test;
#[cfg(test)]
pub use crate::test::{MockClient, MockTransport};


impl From<EngineKind> for ChainAlias {
    fn from(s: EngineKind) -> Self {
        match s {
            EngineKind::Eth => ChainAlias::ETH,
            EngineKind::Abos => ChainAlias::ABOS,
        }
    }
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
    let sign_key = PrivKey::from_str(&config.sign_key).unwrap();
    let eth_pair = KeyPair::from_privkey(sign_key);
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

    let spv = Arc::new(Supervisor {
        client: client.clone(),
        pool: pool.clone(),
        network: network.clone(),
        key: key,
        eth_key: sign_key.clone(),
        packet_nonce: Arc::new(Mutex::new(packet_nonce)),
        phantom: std::marker::PhantomData,
    });

    // init listeners
    let side_listeners = config.listeners.iter().map( |c|{
        let contract_address = Address::from_str(&c.address).unwrap();
        SideListener {
            url: c.url.clone(),
            contract_address: contract_address,
            db_file: Path::new(&config.db_path).join(&c.name).into(),
            spv: spv.clone(),
            chain: c.kind.clone().into(),
            style: c.stream_style.clone(),
        }
    });

    // start listeners
    side_listeners.for_each(|listener| {
        listener.start();
    });

    let mut dic: HashMap<String, Sender<RawEvent<AccountId>>> = HashMap::new();
    config.senders.iter().for_each( |c| {
        let contract_address = Address::from_str(&c.address).unwrap();
        let pair = KeyPair::from_privkey(PrivKey::from_str(&c.key.clone().unwrap()).unwrap());

        match c.kind {
            EngineKind::Eth => {
                let tx = SideSender {
                    name: c.name.clone(),
                    url: c.url.clone(),
                    contract_address: contract_address,
                    pair: pair.clone(),
                    proxy: EthProxy {
                        pair: eth_pair.clone(),
                        context: SignContext {
                            height: 0,
                            nonce: U256::from(0),
                            contract_address: contract_address,
                        },
                    },
                }.start();
                dic.insert(c.name.clone(), tx);
            },

            EngineKind::Abos => {
                let tx = SideSender {
                    name: c.name.clone(),
                    url: c.url.clone(),
                    contract_address: contract_address,
                    pair: pair.clone(),
                    proxy: AbosProxy {
                        pair: pair.clone(),
                        chain_id: U256::from_str("00000000000000000000000000000000000000000000ca812def6446350c7e8d").unwrap(),
                        context: SignContext {
                            height: 0,
                            nonce: U256::from(0),
                            contract_address: contract_address,
                        },
                    },
                }.start();
                dic.insert(c.name.clone(), tx);
            }
        }

    });

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
                if let Event::order(e) = event {
                    match e {
                        RawEvent::MatchOrder(bill, price, seller, sell_sender, sell_receiver, sell_amount, sell_reserved, sell_tag,
                                buyer, buy_sender, buy_receiver, buy_amount, buy_reserved, buy_tag) => {
                                
                                let sell_value: U256 = U256::from(*sell_amount) * U256::from(1_000_000_000);
                                let buy_value: U256 = U256::from(*buy_amount) * U256::from(1_000_000_000);

                                let sell_event = events::MatchEvent {
                                    tag: *sell_tag,
                                    bill: *bill as u64,
                                    from: Address::from_slice(sell_sender.as_ref()),
                                    from_bond: H256::from_slice(seller.as_ref()),
                                    to: Address::from_slice(buy_receiver.as_ref()),
                                    to_bond: H256::from_slice(buyer.as_ref()),
                                    value: sell_value,
                                    reserved: *sell_reserved as u8,
                                };
                                info!("{:?}", sell_event);

                                let buy_event = events::MatchEvent {
                                    tag: *buy_tag,
                                    bill: *bill as u64,
                                    from: Address::from_slice(buy_sender.as_ref()),
                                    from_bond: H256::from_slice(buyer.as_ref()),
                                    to: Address::from_slice(sell_receiver.as_ref()),
                                    to_bond: H256::from_slice(seller.as_ref()),
                                    value: buy_value,
                                    reserved: *buy_reserved as u8,
                                };
                                info!("{:?}", buy_event);
                                spv.submit(RelayMessage::from(sell_event));
                                spv.submit(RelayMessage::from(buy_event));
                        }
                        RawEvent::Settlement(message, signatures) => {
                            info!(
                                "raw match event: message: {}, signatures: {}",
                                message.to_hex(),
                                signatures.to_hex()
                            );
                            events::MatchEvent::from_bytes(message)
                                .map(|ie| {
                                    if ie.tag == 1 {
                                        if let Some(kovan_sender) = dic.get("kovan") {
                                            kovan_sender.send(e.clone()).unwrap();
                                        } else {
                                            warn!("Uninstantiated sender: {:?}", ie.tag);
                                        }
                                    } else if ie.tag == 2 {
                                        if let Some(abos_sender) = dic.get("abos") {
                                            abos_sender.send(e.clone()).unwrap();
                                        } else {
                                            warn!("Uninstantiated sender: {:?}", ie.tag);
                                        }
                                    } else {
                                        warn!("unknown event tag of ingress: {:?}", ie.tag);
                                    }
                                })
                                .map_err(|_err| {
                                    warn!(
                                        "unexpected format of match, message {:?}",
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
