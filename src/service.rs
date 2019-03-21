//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.

#![warn(unused_extern_crates)]

use std::sync::Arc;
use log::info;
use transaction_pool::{self, txpool::{Pool as TransactionPool}};
use node_runtime::{self, GenesisConfig, opaque::Block, RuntimeApi};
use substrate_service::{
	FactoryFullConfiguration, LightComponents, FullComponents, FullBackend,
	FullClient, LightClient, LightBackend, FullExecutor, LightExecutor,
	TaskExecutor,
};
use basic_authorship::ProposerFactory;
use consensus::{import_queue, start_aura, AuraImportQueue, SlotDuration, NothingExtra};
use primitives::{ed25519::Pair, Pair as _Pair};
use inherents::InherentDataProviders;
use network::construct_simple_protocol;
use substrate_executor::native_executor_instance;
use substrate_service::construct_service_factory;
use vendor::{start_vendor, VendorServiceConfig};
use signer::Keyring;

pub use substrate_executor::NativeExecutor;
// Our native executor instance.
native_executor_instance!(
	pub Executor,
	node_runtime::api::dispatch,
	node_runtime::native_version,
	include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/node_runtime.compact.wasm")
);

#[derive(Default)]
pub struct NodeConfig {
	inherent_data_providers: InherentDataProviders,
}

construct_simple_protocol! {
	/// Demo protocol attachment for substrate.
	pub struct NodeProtocol where Block = Block { }
}

construct_service_factory! {
	struct Factory {
		Block = Block,
		RuntimeApi = RuntimeApi,
		NetworkProtocol = NodeProtocol { |config| Ok(NodeProtocol::new()) },
		RuntimeDispatch = Executor,
		FullTransactionPoolApi = transaction_pool::ChainApi<client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>, Block>
			{ |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
		LightTransactionPoolApi = transaction_pool::ChainApi<client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>, Block>
			{ |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
		Genesis = GenesisConfig,
		Configuration = NodeConfig,
		FullService = FullComponents<Self>
			{ |config: FactoryFullConfiguration<Self>, executor: TaskExecutor| {
                let db_path = config.database_path.clone();
                let keyring = config.keys.first().map_or(Keyring::default(), |key| Keyring::from(key.as_bytes()));
                info!("eth signer key: {}", keyring.to_hex());
                match FullComponents::<Factory>::new(config, executor.clone()) {
                    Ok(service) => {
                        executor.spawn(start_vendor(
                            VendorServiceConfig { kovan_url: "https://kovan.infura.io/v3/5b83a690fa934df09253dd2843983d89".to_string(),
                                                  ropsten_url: "https://ropsten.infura.io/v3/5b83a690fa934df09253dd2843983d89".to_string(),
                                                  kovan_address: "690aB411ca08bB0631C49513e10b29691561bB08".to_string(),
                                                  ropsten_address: "631b6b933Bc56Ebd93e4402aA5583650Fcf74Cc7".to_string(),
                                                  db_path: db_path,
                                                  eth_key: keyring.to_hex(), // sign message
                                                },
                            service.network(),
                            service.client(),
                            service.transaction_pool(),
                            service.keystore(),
                            service.on_exit(),
                        ));
                        return Ok(service)
                    },
                    Err(err) => return Err(err),
                }
                }
			},
		AuthoritySetup = {
			|service: Self::FullService, executor: TaskExecutor, key: Option<Arc<Pair>>| {
				if let Some(key) = key {
					info!("Using authority key {}", key.public());
					let proposer = Arc::new(ProposerFactory {
						client: service.client(),
						transaction_pool: service.transaction_pool(),
					});
					let client = service.client();
					executor.spawn(start_aura(
						SlotDuration::get_or_compute(&*client)?,
						key.clone(),
						client.clone(),
						client,
						proposer,
						service.network(),
						service.on_exit(),
						service.config.custom.inherent_data_providers.clone(),
					)?);
				}

				Ok(service)
			}
		},
		LightService = LightComponents<Self>
			{ |config, executor| <LightComponents<Factory>>::new(config, executor) },
		FullImportQueue = AuraImportQueue<
			Self::Block,
		>
			{ |config: &mut FactoryFullConfiguration<Self> , client: Arc<FullClient<Self>>|
				import_queue(
					SlotDuration::get_or_compute(&*client)?,
					client.clone(),
					None,
					client,
					NothingExtra,
					config.custom.inherent_data_providers.clone(),
				).map_err(Into::into)
			},
		LightImportQueue = AuraImportQueue<
			Self::Block,
		>
			{ |config: &mut FactoryFullConfiguration<Self>, client: Arc<LightClient<Self>>|
				import_queue(
					SlotDuration::get_or_compute(&*client)?,
					client.clone(),
					None,
					client,
					NothingExtra,
					config.custom.inherent_data_providers.clone(),
				).map_err(Into::into)
			},
	}
}

/*
#[cfg(test)]
mod vendor_template {
    use std::sync::Arc;
    use futures::{Future, Stream};
    use network::SyncProvider;
    use client::{BlockchainEvents, BlockBody, blockchain::HeaderBackend, BlockNumberToHash};
    use substrate_keystore::Store as Keystore;
    use transaction_pool::txpool::{self, Pool as TransactionPool, ExtrinsicFor};

    use futures::prelude::*;
    use tokio_timer::{Timer, Interval};

    use node_runtime::{EventRecord, UncheckedExtrinsic, Call, Event, sigcount::*,MatrixCall};
    use runtime_io;
    use primitives::storage::{StorageKey, StorageData, StorageChangeSet};
    use runtime_primitives::generic::{BlockId, Era};
    use runtime_primitives::codec::{Decode, Encode, Compact};
    use node_primitives::{AccountId, Index};
    struct Mock{
        poll_interval: Interval,
    }

    impl Stream for Mock {
        type Item = ();
        type Error = ();
        fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
            loop {
                let _ = match self.poll_interval.poll() {
                    Err(err) => return Err(()),
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
                    Ok(Async::Ready(Some(value))) => value,
                };
                println!("############");
                // do you things
                return Ok(Async::Ready(Some(())));
            }
        }
    }

    pub fn start_vendor<A, B, C, N>(
        network: Arc<N>,
        client: Arc<C>,
        pool: Arc<TransactionPool<A>>,
        keystore: &Keystore,
        on_exit: impl Future<Item=(),Error=()> + Clone,
    ) -> impl Future<Item=(),Error=()> where
        A: txpool::ChainApi<Block = B>,
        B: Block,
        C: BlockchainEvents<B> + BlockBody<B> + HeaderBackend<B> + BlockNumberToHash,
        N: SyncProvider<B>
    {
        let mock = Mock { poll_interval: Interval::new_interval(std::time::Duration::from_secs(2))};
        let mock_stream = mock
                        .for_each(|_| Ok(()));

        let key = keystore.load(&keystore.contents().unwrap()[0], "").unwrap();
        let local_id: AccountId = key.public().0.into();
        println!("ROS account: {:?}", key.public().to_ss58check());
        let timer_stream = Interval::new_interval(std::time::Duration::from_secs(10));
        let fork_client = client.clone();
        let send_stream = timer_stream.for_each(move |_| {
                // get nonce
                // let mut next_index = {
                //     let local_id = self.local_key.public().0;
                //     let cur_index = self.transaction_pool.cull_and_get_pending(&BlockId::hash(self.parent_hash), |pending| pending
                //         .filter(|tx| tx.verified.sender == local_id)
                //         .last()
                //         .map(|tx| Ok(tx.verified.index()))
                //         .unwrap_or_else(|| fork_client.account_nonce(&self.parent_id, local_id))
                //         .map_err(Error::from)
                //     );

                //     match cur_index {
                //         Ok(cur_index) => cur_index + 1,
                //         Err(e) => {
                //             warn!(target: "consensus", "Error computing next transaction index: {:?}", e);
                //             0
                //         }
                //     }
                // };

                let block = fork_client.info().unwrap().best_number;
                let payload = (
                    Compact::<Index>::from(0),  // index/nonce
                    Call::Matrix(MatrixCall::ingress(vec![0, 1, 3, 4, 5, 6, 7],vec![1,0])), //function
                    Compact::<Index>::from(0),  // index/nonce
                    Call::Matrix(MatrixCall::ingress(vec![0, 1, 3, 4, 5, 6, 7], vec![0, 1, 3, 4, 5, 6, 7])), //function
                    Era::immortal(),  
                    fork_client.genesis_hash(),
                );
                let signature = key.sign(&payload.encode());
                let extrinsic = UncheckedExtrinsic::new_signed(
                    payload.0.into(),
                    payload.1,
                    local_id.into(),
                    signature.into(),
                    payload.4
                );
                let xt: ExtrinsicFor<A> = Decode::decode(&mut &extrinsic.encode()[..]).unwrap();
                //println!("check: {:?}", extrinsic.check());
                println!("@@@@@@@@@@@result: {:?}", pool.submit_one(&BlockId::number(block), xt));
                Ok(())
        }).map_err(|_| ());

        // how to fetch real key?
        let events_key = StorageKey(runtime_io::twox_128(b"System Events").to_vec());
        let storage_stream = client.storage_changes_notification_stream(Some(&[events_key])).unwrap()
        .map(|(block, changes)| StorageChangeSet { block, changes: changes.iter().cloned().collect()})
        .for_each(move |change_set| {
            println!("@@@@@@@@@@@@@@@@@@");
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
            println!("@@@@@@@@@@@@changes: {:?}", events);
            events.iter().for_each(|event| {
                if let Event::sigcount(e) = event {
                    match e {
                        //RawEvent::Ingress(hash, msg) => println!("@@@@@@@@ Ingress: hash{:?}, msg{:?}", hash, msg),
                        RawEvent::Txisok(transcation) => println!("XXXXXXXXXXX Txisok: hash{:?}",transcation),
                        RawEvent::TranscationVerified(transcation,vec) => println!("XXXXXXXXXXX Txisok: hash{:?}",transcation) ,
                        // other events.
                        _ => println!("@@@@@@@ other: {:?}", e),
                    }
                }
            });
            Ok(())
        });

        storage_stream
        .join(mock_stream)                      
        .join(send_stream)
        .map(|_| ())
        .select(on_exit)
        .then(|_| {
            println!("##############on exit");
            Ok(())
        })
    }
}
*/