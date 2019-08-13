use crate::error::{self, ResultExt};
use crate::events;
use crate::label::ChainAlias;
use crate::message::RelayMessage;
use crate::state::State;
use crate::streams::log_stream::{LogStream, LogStreamOptions};
use crate::supervisor::SuperviseClient;
use contracts;
use futures::{Async, Poll, Stream};
use std::sync::Arc;
use std::time::Duration;
use web3::{types::Address, Transport};

pub struct Mapper<T: Transport, C: SuperviseClient> {
    client: Arc<C>,
    state: State,
    lock_stream: LogStream<T>,
    unlock_stream: LogStream<T>,
}

impl<T: Transport, C: SuperviseClient> Mapper<T, C> {
    pub fn new(
        transport: &T,
        client: Arc<C>,
        state: State,
        contract_address: Address,
        chain: ChainAlias,
    ) -> Self {
        Self {
            lock_stream: LogStream::new(LogStreamOptions {
                request_timeout: Duration::from_secs(30),
                poll_interval: Duration::from_secs(10),
                confirmations: 3,
                transport: transport.clone(),
                contract_address: contract_address,
                last_block_number: state.lock_token, //10351660,
                filter: contracts::mapper::events::lock_token::filter(),
                chain,
            }),
            unlock_stream: LogStream::new(LogStreamOptions {
                request_timeout: Duration::from_secs(30),
                poll_interval: Duration::from_secs(10),
                confirmations: 3,
                transport: transport.clone(),
                contract_address: contract_address,
                last_block_number: state.unlock_token,
                filter: contracts::mapper::events::unlock_token::filter(),
                chain,
            }),
            client: client,
            state: state,
        }
    }

    pub fn mock(transport: &T, client: Arc<C>) -> Self {
        Self {
            lock_stream: LogStream::new(LogStreamOptions {
                request_timeout: Duration::from_secs(1),
                poll_interval: Duration::from_secs(1),
                confirmations: 12,
                transport: transport.clone(),
                contract_address: "0000000000000000000000000000000000000001".into(),
                last_block_number: 3,
                filter: contracts::mapper::events::lock_token::filter(),
                chain: ChainAlias::ETH,
            }),
            unlock_stream: LogStream::new(LogStreamOptions {
                request_timeout: Duration::from_secs(1),
                poll_interval: Duration::from_secs(1),
                confirmations: 12,
                transport: transport.clone(),
                contract_address: "0000000000000000000000000000000000000002".into(),
                last_block_number: 3,
                filter: contracts::mapper::events::unlock_token::filter(),
                chain: ChainAlias::ETH,
            }),
            client: client,
            state: State::default(),
        }
    }
}

impl<T: Transport, C: SuperviseClient> Stream for Mapper<T, C> {
    type Item = State;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            let mut changed = false;
            // get ingress logs
            let ret = try_maybe_stream!(self
                .lock_stream
                .poll()
                .chain_err(|| "Vendor: Get poll log Failed.",));
            if let Some(ret) = ret {
                for log in &ret.logs {
                    let message =
                        events::LockTokenEvent::from_log(log)?;
                    self.client.submit(RelayMessage::from(message));
                }
                self.state.lock_token = ret.to;
                changed = true;
            }

            let ret = try_maybe_stream!(self
                .unlock_stream
                .poll()
                .chain_err(|| "Vendor: Get poll log Failed.",));
            if let Some(ret) = ret {
                for log in &ret.logs {
                    let message =
                        events::UnlockTokenEvent::from_log(log)?;
                    self.client.submit(RelayMessage::from(message));
                }
                self.state.unlock_token = ret.to;
                changed = true;
            }

            if changed {
                return Ok(Async::Ready(Some(self.state.clone())));
            } else {
                return Ok(Async::NotReady);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_transport;
    use crate::test::MockClient;
    use contracts;
    use serde_json::json;
    use tokio_core::reactor::Core;
    use crate::vendor::Vendor;

    #[test]
    #[ignore]
    fn test_vendor_stream() {
        let lock_topic = contracts::mapper::events::lock_token::filter().topic0;
        let unlock_topic = contracts::mapper::events::unlock_token::filter().topic0;

        let client = Arc::new(MockClient::default());
        let transport = mock_transport!(
            "eth_blockNumber" =>
                req => json!([]),
                res => json!("0x1011");
            "eth_getLogs" =>
                req => json!([{
                    "address": "0x0000000000000000000000000000000000000001",
                    "fromBlock": "0x4",
                    "toBlock": "0x1005",
                    "topics": [lock_topic]
                }]),
                res => json!([{
                    "address": "0x0000000000000000000000000000000000000001",
                    "topics": [lock_topic],
                    "data": "0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000074241db5f3ebaeecf9506e4ae98818609334160400000000000000000000000074241db5f3ebaeecf9506e4ae98818609334160400000000000000000000000000000000000000000000000000000000054c56380000000000000000000000000000000000000000000000000000000000000002",
                    "type": "",
                    "transactionHash": "0x1045bfe274b88120a6b1e5d01b5ec00ab5d01098346e90e7c7a3c9b8f0181c80"
                }]);
            "eth_blockNumber" =>
                req => json!([]),
                res => json!("0x1011");
            "eth_getLogs" =>
                req => json!([{
                    "address": "0x0000000000000000000000000000000000000002",
                    "fromBlock": "0x4",
                    "toBlock": "0x1005",
                    "topics": [unlock_topic]
                }]),
                res => json!([]);
        );
        let vendor = Vendor::mock(&transport, client.clone());
        let mut event_loop = Core::new().unwrap();
        let _log_ranges = event_loop.run(vendor.take(1).collect()).unwrap();
    }
}
