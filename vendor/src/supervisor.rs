use crate::events;
use crate::message::{RelayMessage, RelayType};
use client::{blockchain::HeaderBackend, runtime_api::Core as CoreApi, BlockchainEvents};
use log::{debug, error, info, trace, warn};
use network::SyncProvider;
use node_primitives::{AccountId, Nonce as Index};
use node_runtime::{
    BankCall, Call, ExchangeCall, MatrixCall, UncheckedExtrinsic,
    VendorApi, /*,exchangerate */
};
use primitives::{crypto::*, ed25519::Pair, Pair as TraitPair};
use runtime_primitives::{
    codec::{Compact, Decode, Encode},
    generic::{BlockId, Era},
    traits::{Block, BlockNumberToHash, ProvideRuntimeApi},
};
use rustc_hex::ToHex;
use signer::{PrivKey};
use std::sync::{
    Arc, Mutex,
};
use transaction_pool::txpool::{self, ExtrinsicFor, Pool as TransactionPool};
use web3::{
    types::{H256},
};

pub trait SuperviseClient {
    fn submit(&self, message: RelayMessage);
}

pub struct PacketNonce<B>
where
    B: Block,
{
    pub nonce: Index, // to control nonce.
    pub last_block: BlockId<B>,
}

pub struct Supervisor<A, B, C, N>
where
    A: txpool::ChainApi,
    B: Block,
{
    pub client: Arc<C>,
    pub pool: Arc<TransactionPool<A>>,
    pub network: Arc<N>,
    pub key: Pair,
    pub eth_key: PrivKey,
    pub phantom: std::marker::PhantomData<B>,
    // pub queue: Vec<(RelayMessage, u8)>,
    pub packet_nonce: Arc<Mutex<PacketNonce<B>>>,
}

impl<A, B, C, N> Supervisor<A, B, C, N>
where
    A: txpool::ChainApi<Block = B>,
    B: Block,
    C: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi,
    N: SyncProvider<B>,
    C::Api: VendorApi<B>,
{
    /// get nonce with atomic
    fn get_nonce(&self) -> Index {
        let mut p_nonce = self.packet_nonce.lock().unwrap();
        let info = self.client.info().unwrap();
        let at = BlockId::Hash(info.best_hash);
        if p_nonce.last_block == at {
            p_nonce.nonce = p_nonce.nonce + 1;
        } else {
            p_nonce.nonce = self
                .client
                .runtime_api()
                .account_nonce(&at, &self.key.public().0.unchecked_into())
                .unwrap();
            p_nonce.last_block = at;
        }

        p_nonce.nonce
    }
}

fn get_tag(tag: &H256, index: usize) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&tag.0[index..(index + 8)]);
    events::array_to_u64(arr)
}

impl<A, B, C, N> SuperviseClient for Supervisor<A, B, C, N>
where
    A: txpool::ChainApi<Block = B>,
    B: Block,
    C: BlockchainEvents<B> + HeaderBackend<B> + BlockNumberToHash + ProvideRuntimeApi,
    N: SyncProvider<B>,
    C::Api: VendorApi<B> + CoreApi<B>,
{
    fn submit(&self, relay_message: RelayMessage) {
        let local_id: AccountId = self.key.public().0.unchecked_into();
        let info = self.client.info().unwrap();
        let at = BlockId::Hash(info.best_hash);
        if self
            .client
            .runtime_api()
            .is_authority(&at, &self.key.public().0.unchecked_into())
            .unwrap()
        {
            let mut message = relay_message.clone();
            //TODO refactor ,now just amend value.
            if message.ty == RelayType::Ingress {
                let mut ingress = events::IngressEvent::from_bytes(&message.raw).unwrap();
                let tag = ingress.tag;
                // H256 1 = [ 0x0, 0x0, ......., 0x1 ]; big endian
                let from_tag = u64::from_be(get_tag(&tag, 8));
                let to_tag = u64::from_be(get_tag(&tag, 24));
                info!("@@@@@@@@@@@@@@from tag: {}, to tag: {}", from_tag, to_tag);
                let mut from_price = self.client.runtime_api().price_of(&at, from_tag).unwrap();
                let mut to_price = self.client.runtime_api().price_of(&at, to_tag).unwrap();
                info!(
                    "@@@@@@@@@@@@@@from price: {}. to price:{}",
                    from_price, to_price
                );
                // TODO when can't get price, then set 1:1
                if to_price == 0 || from_price == 0 {
                    to_price = 1;
                    from_price = 1;
                }
                let to_value = (ingress.value * to_price) / from_price;
                ingress.value = to_value;
                message.raw = ingress.to_bytes();
            }
            let nonce = self.get_nonce();
            let signature: Vec<u8> = signer::Eth::sign_message(&self.eth_key, &message.raw).into();

            let function = match message.ty {
                RelayType::Ingress => {
                    info!(
                        "listener Ingress message: {}, signature: {}",
                        message.raw.to_hex(),
                        signature.to_hex()
                    );
                    Call::Matrix(MatrixCall::ingress(message.raw, signature))
                },
                RelayType::Egress => Call::Matrix(MatrixCall::egress(message.raw, signature)),
                RelayType::Deposit => Call::Bank(BankCall::deposit(message.raw, signature)),
                RelayType::Withdraw => Call::Bank(BankCall::withdraw(message.raw, signature)),
                RelayType::SetAuthorities => {
                    Call::Matrix(MatrixCall::reset_authorities(message.raw, signature))
                },
                RelayType::ExchangeRate => {
                    Call::Exchange(ExchangeCall::check_exchange(message.raw, signature))
                },
                RelayType::LockToken => {
                    // TODO
                    Call::Matrix(MatrixCall::ingress(message.raw, signature))
                },
                RelayType::UnlockToken => {
                    // TODO
                    Call::Matrix(MatrixCall::ingress(message.raw, signature))
                }
            };

            let payload = (
                Compact::<Index>::from(nonce), // index/nonce
                function,                      //function
                Era::immortal(),
                self.client.genesis_hash(),
            );

            let signature = self.key.sign(&payload.encode());
            let extrinsic = UncheckedExtrinsic::new_signed(
                payload.0.into(),
                payload.1,
                local_id.into(),
                signature.into(),
                payload.2,
            );

            let xt: ExtrinsicFor<A> = Decode::decode(&mut &extrinsic.encode()[..]).unwrap();
            trace!("extrinsic {:?}", xt);
            info!("@submit transaction {:?}", self.pool.submit_one(&at, xt));
        }
    }
}
