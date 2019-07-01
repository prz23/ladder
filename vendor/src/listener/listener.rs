use crate::error::{self, ResultExt};
use crate::label::ChainAlias;
use crate::state::StateStorage;
use crate::supervisor::SuperviseClient;
use crate::vendor::Vendor;
use crate::mapper::Mapper;
use futures::{Stream};
use log::{error, info};
use std::marker::{Send, Sync};
use std::path::{PathBuf};
use std::sync::{Arc};
use tokio_core::reactor::Core;
use web3::{
    types::{Address},
};

const MAX_PARALLEL_REQUESTS: usize = 10;

pub enum ListenerStreamStyle {
    Vendor,
    Mapper,
}

pub struct SideListener<V> {
    pub url: String,
    pub contract_address: Address,
    pub db_file: PathBuf,
    pub spv: Arc<V>,
    pub enable: bool,
    pub chain: ChainAlias,
    pub style: ListenerStreamStyle,
}

fn print_err(err: error::Error) {
    let message = err
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n\nCaused by:\n  ");
    error!("listener: {}", message);
}

fn is_err_time_out(err: &error::Error) -> bool {
    err.iter().any(|e| e.to_string() == "Request timed out")
}

impl<V> SideListener<V>
where
    V: SuperviseClient + Send + Sync + 'static,
{
    pub fn start(self) {
        // return directly.
        if !self.enable {
            return;
        }
        // TODO hook the event of http disconnect to keep run.
        std::thread::spawn(move || {
            let mut event_loop = Core::new().unwrap();
            loop {
                info!("Intend to connect node.");
                let transport = web3::transports::Http::with_event_loop(
                    &self.url,
                    &event_loop.handle(),
                    MAX_PARALLEL_REQUESTS,
                )
                .chain_err(|| format!("Cannot connect to node at {}", self.url))
                .unwrap();

                if !self.db_file.exists() {
                    std::fs::File::create(&self.db_file)
                        .expect("failed to create the storage file of state.");
                }
                let mut storage = StateStorage::load(self.db_file.as_path()).unwrap();
                
                match self.style {
                    ListenerStreamStyle::Vendor => {
                        let stream = Vendor::new(
                            &transport,
                            self.spv.clone(),
                            storage.state.clone(),
                            self.contract_address,
                            self.chain,
                        )
                        .and_then(|state| {
                            storage.save(&state)?;
                            Ok(())
                        })
                        .for_each(|_| Ok(()));

                        match event_loop.run(stream) {
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
                    },
                    ListenerStreamStyle::Mapper => {
                        let stream = Mapper::new(
                            &transport,
                            self.spv.clone(),
                            storage.state.clone(),
                            self.contract_address,
                            self.chain,
                        )
                        .and_then(|state| {
                            storage.save(&state)?;
                            Ok(())
                        })
                        .for_each(|_| Ok(()));

                        match event_loop.run(stream) {
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
                    },
                }
                
            }
        });
    }
}
