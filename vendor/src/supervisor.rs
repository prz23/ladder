use crate::events;
use crate::message::{RelayMessage, RelayType};
use client::{blockchain::HeaderBackend, runtime_api::Core as CoreApi, BlockchainEvents};
use log::{info, trace};
use network::SyncProvider;
use node_primitives::{AccountId, Nonce as Index};
use node_runtime::{
    BankCall, Call, ExchangeCall, MatrixCall, ErcCall, OrderCall, UncheckedExtrinsic,
    VendorApi, /*,exchangerate */
};
use primitives::{crypto::*, ed25519::Pair, Pair as TraitPair, blake2_256};
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
            let nonce = self.get_nonce();
            let signature: Vec<u8> = signer::Eth::sign_message(&self.eth_key, &message.raw).into();
            
            trace!(
                    "listener {:?} message: 0x{}, signature: 0x{}",
                    message.ty,
                    message.raw.to_hex(),
                    signature.to_hex()
                    );
            let function = match message.ty {
                RelayType::Match => Call::Order(OrderCall::match_order_verification(message.raw, signature)),
                RelayType::Request => Call::Bank(BankCall::request(message.raw, signature)),
                RelayType::Deposit => Call::Bank(BankCall::deposit(message.raw, signature)),
                RelayType::Withdraw => Call::Bank(BankCall::withdraw(message.raw, signature)),
                RelayType::SetAuthorities => Call::Matrix(MatrixCall::reset_authorities(message.raw, signature)),
                RelayType::ExchangeRate => Call::Exchange(ExchangeCall::check_exchange(message.raw, signature)),
                RelayType::LockToken => Call::Erc(ErcCall::lock_erc(message.raw, signature)),
                RelayType::UnlockToken => Call::Erc(ErcCall::unlock_erc(message.raw, signature)),
            };
            info!("ladder operator nonce: {}, function: {:?}", nonce, message.ty);
            let raw_payload = (
                Compact::<Index>::from(nonce), // index/nonce
                function,                      //function
                Era::immortal(),
                self.client.genesis_hash(),
            );
            let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
                self.key.sign(&blake2_256(payload)[..])
            } else {
                self.key.sign(payload)
            });
            let extrinsic = UncheckedExtrinsic::new_signed(
                raw_payload.0.into(),
                raw_payload.1,
                local_id.into(),
                signature.into(),
                raw_payload.2,
            );

            let xt: ExtrinsicFor<A> = Decode::decode(&mut &extrinsic.encode()[..]).unwrap();
            trace!("extrinsic {:?}", xt);
            info!("@submit transaction {:?}", self.pool.submit_one(&at, xt));
        }
    }
}
