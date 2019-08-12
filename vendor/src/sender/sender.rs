use super::proxy::SenderProxy;
use crate::error::ResultExt;
use log::{info, warn, error};
use node_primitives::{AccountId, BlockNumber, Hash};
use node_runtime::{
    order::RawEvent
};
use signer::{KeyPair};
use std::marker::{Send, Sync};
use std::sync::{
    Arc,
    mpsc::{channel, Sender},
};
use tokio_core::reactor::Core;
use web3::{
    types::{Address},
};
use std::time::Duration;

const MAX_PARALLEL_REQUESTS: usize = 10;

pub struct SideSender<P> {
    pub name: String,
    pub url: String,
    pub contract_address: Address,
    pub pair: KeyPair,
    pub proxy: P,
}

impl<P> SideSender<P>
where
    P: SenderProxy + Send + Sync + 'static,
{
    pub fn start(mut self) -> Sender<RawEvent<AccountId>> {
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            let mut event_loop = Core::new().unwrap();
            loop {
                let event = receiver.recv().unwrap();
                let transport = match web3::transports::Http::with_event_loop(
                    &self.url,
                    &event_loop.handle(),
                    MAX_PARALLEL_REQUESTS,
                )
                .chain_err(|| format!("Cannot connect to ethereum node at {}", self.url)) {
                    Ok(t) => {
                        t
                    },
                    Err(e) => {
                        error!("{}",e);
                        std::thread::sleep(Duration::from_secs(30));
                        continue;
                    }
                };
                // initialize proxy.
                if let Err(e) = self.proxy.initialize(&mut event_loop, &transport) {
                    error!("{}", e);
                    std::thread::sleep(Duration::from_secs(60));
                    continue;
                }
                loop {
                    let event = receiver.recv().unwrap();
                    match event {
                        RawEvent::Settlement(message, signatures) => {
                            info!(
                                "ingress message: {:?}, signatures: {:?}",
                                message, signatures
                            );
                            let payload = contracts::bridge::functions::settle::encode_input(
                                message, signatures,
                            );
                            self.proxy.send(&mut event_loop, &transport, payload);
                        }
                        _ => warn!("SideSender: unknown event!"),
                    }
                }
            }
        });

        sender
    }
}
