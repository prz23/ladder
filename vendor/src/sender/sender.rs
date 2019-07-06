use super::proxy::SenderProxy;
use crate::error::ResultExt;
use log::{info, warn};
use node_primitives::{AccountId, BlockNumber, Hash};
use node_runtime::{
    order::RawEvent
};
use signer::{KeyPair};
use std::marker::{Send, Sync};
use std::sync::{
    mpsc::{channel, Sender},
};
use tokio_core::reactor::Core;
use web3::{
    types::{Address},
};

const MAX_PARALLEL_REQUESTS: usize = 10;

pub struct SideSender<P> {
    pub name: String,
    pub url: String,
    pub contract_address: Address,
    pub pair: KeyPair,
    pub enable: bool,
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
            let transport = web3::transports::Http::with_event_loop(
                &self.url,
                &event_loop.handle(),
                MAX_PARALLEL_REQUESTS,
            )
            .chain_err(|| format!("Cannot connect to ethereum node at {}", self.url))
            .unwrap();

            // initialize proxy.
            self.proxy.initialize(&mut event_loop, &transport);
            loop {
                let event = receiver.recv().unwrap();

                if !self.enable {
                    continue;
                }

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
        });

        sender
    }
}
