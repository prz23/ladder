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
extern crate ethabi;
extern crate rustc_hex;

extern crate node_runtime;
extern crate node_primitives;
extern crate sr_io as runtime_io;
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
mod utils;

use message::{RelayMessage,RelayType};
use tokio_core::reactor::Core;
use std::sync::{Arc, atomic::AtomicUsize};
use error::{ResultExt};
use vendor::Vendor;
use utils::StreamExt;
use network::SyncProvider;
use futures::{Future, Stream};
use keystore::Store as Keystore;
use runtime_primitives::codec::{Decode, Encode, Compact};
use runtime_primitives::generic::{BlockId, Era};
use runtime_primitives::traits::{As, Block, Header, BlockNumberToHash, ProvideRuntimeApi};
use client::{BlockchainEvents, BlockBody, blockchain::HeaderBackend};
use primitives::storage::{StorageKey, StorageData, StorageChangeSet};
use primitives::ed25519::Pair;
use transaction_pool::txpool::{self, Pool as TransactionPool, ExtrinsicFor};
use node_runtime::{
    Call, UncheckedExtrinsic, EventRecord, Event,MatrixCall, matrix::*, VendorApi
};
use futures::prelude::*;
use node_primitives::{AccountId, Index};

const MAX_PARALLEL_REQUESTS: usize = 10;

pub trait SuperviseClient{
    fn submit(&self, message: RelayMessage);
}

pub struct Supervisor<A, B, C, N> where
    A: txpool::ChainApi
{
    pub client: Arc<C>,
    pub pool: Arc<TransactionPool<A>>,
    pub network: Arc<N>,
    pub key: Pair,
    pub phantom: std::marker::PhantomData<B>,
    // pub queue: Vec<(RelayMessage, u8)>,
    pub nonce: Arc<AtomicUsize>, // to control nonce.
}

impl<A, B, C, N> SuperviseClient for Supervisor<A, B, C, N> where
    A: txpool::ChainApi<Block = B>,
    B: Block,
    C: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi,
    N: SyncProvider<B>,
    C::Api: VendorApi<B>
{
    fn submit(&self, message: RelayMessage) {
        let local_id: AccountId = self.key.public().0.into();
        let info = self.client.info().unwrap();
        let at = BlockId::Hash(info.best_hash);

        let nonce = self.client.runtime_api().account_nonce(&at, local_id.into()).unwrap();
        let function =  match message.ty {
                RelayType::Ingress => Call::Matrix(MatrixCall::ingress(message.raw, vec![1])),
                RelayType::Egress => Call::Matrix(MatrixCall::egress(message.raw, vec![1])),
                RelayType::Deposit => Call::Matrix(MatrixCall::deposit(message.raw, vec![1])),
                RelayType::Withdraw => Call::Matrix(MatrixCall::withdraw(message.raw, vec![1])),
                RelayType::SetAuthorities => Call::Matrix(MatrixCall::reset_authorities(message.raw, vec![1])),
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
        println!("@submit transaction {:?}",self.pool.submit_one(&at, xt));
    }
}

pub struct VendorServiceConfig {
    pub url: String,
}

// /// Start the supply worker. The returned future should be run in a tokio runtime.
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
    info!("ss58 account: {:?}, ", key.public().to_ss58check());

    let spv = Arc::new(Supervisor {
        client: client.clone(),
        pool: pool.clone(),
        network: network.clone(),
        key: key,
        nonce: Arc::new(AtomicUsize::new(0)),
        phantom: std::marker::PhantomData,
    });
    //new a thread to listen 
    std::thread::spawn(move ||{
        let mut event_loop = Core::new().unwrap();
        let transport = web3::transports::Http::with_event_loop(
            &config.url,
            &event_loop.handle(),
            MAX_PARALLEL_REQUESTS,
        )
        .chain_err(|| {format!("Cannot connect to ethereum node at {}",config.url)}).unwrap();
        let vendor = Vendor::new(&transport, spv).for_each(|_| Ok(()));
        event_loop.run(vendor).unwrap();
    });

    // // how to fetch real key?
    let events_key = StorageKey(runtime_io::twox_128(b"System Events").to_vec());
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
            if let Event::sigcount(e) = event {
                match e {
                    //RawEvent::Ingress(hash, msg) => println!("@@@@@@@@ Ingress: hash{:?}, msg{:?}", hash, msg),
                    // RawEvent::Txisok(transcation) => println!("XXXXXXXXXXX Txisok: hash{:?}",transcation),
                    // RawEvent::TranscationVerified(transcation,vec) => println!("XXXXXXXXXXX Txisok: hash{:?}",transcation) ,
                    // other events.
                    _ => println!("@@@@@@@ other: {:?}", e),
                }
            }
        });
        Ok(())
    });

    storage_stream
            .map(|_|())
            .select(on_exit)
            .then(|_| {
                println!("@@@@@@@@@exit");
            Ok(())})
}

/*
/// main logical of vendor.
fn origin() {
    let web3 = Web3::new();
    /// save last 100 logs
    let logs_dump = LogsDump::new();
    let number = FixedNumber::load(path)?;
    loop {
        let _ await interval?;
        let block_number = await get_block_number(&web3)?;
        number.wal(block_number);
        if (number.changed) {
            let logs = await get_logs(&web3, abi)?;
            let logs = check(logs);
            let logs = logs_dump.dump(logs);
            let signed_logs = logs.map(switch).map(sign);
            if client.submit(signed_logs)? {
                number.flash()?;
            }
        }
    }
}
*/