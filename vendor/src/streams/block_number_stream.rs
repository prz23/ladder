use crate::error;
use crate::error::ResultExt;
use crate::label::ChainAlias;
use futures::future::FromErr;
use futures::{try_ready, Async, Future, Poll, Stream};
use log::debug;
use std::time::Duration;
use tokio_timer::{Interval, Timeout, Timer};
use web3;
use web3::api::Namespace;
use web3::helpers::CallFuture;
use web3::types::U256;
use web3::Transport;

/// Block Number Stream state.
enum State<T: Transport> {
    AwaitInterval,
    AwaitBlockNumber(Timeout<FromErr<CallFuture<U256, T::Out>, error::Error>>),
}

pub struct BlockNumberStreamOptions<T> {
    pub request_timeout: Duration,
    pub poll_interval: Duration,
    pub confirmations: u32,
    pub transport: T,
    pub last_block_number: u64,
    pub chain: ChainAlias,
}

/// `Stream` that repeatedly polls `eth_blockNumber` and yields new block numbers.
pub struct BlockNumberStream<T: Transport> {
    request_timeout: Duration,
    confirmations: u32,
    transport: T,
    last_checked_block: u64,
    timer: Timer,
    poll_interval: Interval,
    state: State<T>,
    chain: ChainAlias,
}

impl<T: Transport> BlockNumberStream<T> {
    pub fn new(options: BlockNumberStreamOptions<T>) -> Self {
        let timer = Timer::default();

        BlockNumberStream {
            request_timeout: options.request_timeout,
            confirmations: options.confirmations,
            poll_interval: timer.interval(options.poll_interval),
            transport: options.transport,
            last_checked_block: options.last_block_number,
            timer,
            state: State::AwaitInterval,
            chain: options.chain,
        }
    }
}

impl<T: Transport> Stream for BlockNumberStream<T> {
    type Item = u64;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            let (next_state, value_to_yield) = match self.state {
                State::AwaitInterval => {
                    // wait until `interval` has passed
                    let _ = try_stream!(self
                        .poll_interval
                        .poll()
                        .chain_err(|| { format!("BlockNumberStream polling interval failed",) }));
                    // let future = web3::api::Eth::new(&self.transport).block_number();
                    let future = match self.chain {
                        ChainAlias::ETH => web3::api::Eth::new(&self.transport).block_number(),
                        ChainAlias::ABOS => web3::api::Abos::new(&self.transport).block_number(),
                    };
                    let next_state = State::AwaitBlockNumber(
                        self.timer.timeout(future.from_err(), self.request_timeout),
                    );
                    (next_state, None)
                }
                State::AwaitBlockNumber(ref mut future) => {
                    let last_block = try_ready!(future
                        .poll()
                        .chain_err(|| "BlockNumberStream: fetching of last block number failed",))
                    .as_u64();
                    debug!(
                        "BlockNumberStream: fetched last block number {}",
                        last_block
                    );
                    // subtraction that saturates at zero
                    let last_confirmed_block = last_block.saturating_sub(self.confirmations as u64);

                    if self.last_checked_block < last_confirmed_block {
                        self.last_checked_block = last_confirmed_block;
                        (State::AwaitInterval, Some(last_confirmed_block))
                    } else {
                        debug!(
                            "BlockNumberStream: no blocks confirmed since we last checked. waiting some more"
                        );
                        (State::AwaitInterval, None)
                    }
                }
            };

            self.state = next_state;

            if value_to_yield.is_some() {
                return Ok(Async::Ready(value_to_yield));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_transport;
    use serde_json::json;
    use tokio_core::reactor::Core;

    #[test]
    fn test_block_number_stream() {
        let transport = mock_transport!(
            "eth_blockNumber" =>
                req => json!([]),
                res => json!("0x1011");
            "eth_blockNumber" =>
                req => json!([]),
                res => json!("0x1011");
            "eth_blockNumber" =>
                req => json!([]),
                res => json!("0x1012");
            "eth_blockNumber" =>
                req => json!([]),
                res => json!("0x1015");
        );

        let block_number_stream = BlockNumberStream::new(BlockNumberStreamOptions {
            request_timeout: Duration::from_secs(1),
            poll_interval: Duration::from_secs(0),
            confirmations: 12,
            transport: transport.clone(),
            last_block_number: 3,
            chain: ChainAlias::ETH,
        });

        let mut event_loop = Core::new().unwrap();
        let block_numbers = event_loop
            .run(block_number_stream.take(3).collect())
            .unwrap();

        assert_eq!(block_numbers, vec![0x1011 - 12, 0x1012 - 12, 0x1015 - 12]);
        assert_eq!(transport.actual_requests(), transport.expected_requests());
    }
}
