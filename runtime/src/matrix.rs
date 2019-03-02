use rstd::prelude::Vec;
use runtime_primitives::traits::*;
use srml_support::{StorageValue, StorageMap, dispatch::Result};
use {balances, system::{self, ensure_signed}};

pub trait Trait: balances::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;
        
        /*
        // offset  0: 32 bytes :: uint256 - tag    //标记 从 A链到B链。
        // offset 32: 20 bytes :: address - recipient address
        // offset 52: 32 bytes :: uint256 - value
        // offset 84: 32 bytes :: bytes32 - transaction hash
        */
        // 数据转发请求消息
        pub fn ingress(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            let hash = T::Hashing::hash_of(&message);
            Self::deposit_event(RawEvent::Ingress(message.clone(), signature));
            <IngressOf<T>>::insert(hash, message.clone());
            Ok(())
        }

        // 数据转发确认消息
        pub fn egress(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            // TODO
            Ok(())
        }

        // 数据转发超时回退消息
        pub fn rollback(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            Ok(())
        }

        // offset 0: 32 bytes :: uint256 - block number
        // offset 32: 20 bytes :: address - authority0
        // offset 52: 20 bytes :: address - authority1
        // offset 72: 20 bytes :: ..    ....
        // 重新设置验证节点消息
        pub fn reset_authorities(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            Ok(())
        }

        // 银行模块存入消息
        pub fn deposit(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            Ok(())
        }

        // 银行模块取出消息
        pub fn withdraw(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            Ok(())
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Matrix {
        pub IngressOf get(ingress_of): map T::Hash => Vec<u8>;
    }
}

type TransferMessage = Vec<u8>;
type Signatures = Vec<u8>;
decl_event! {
    pub enum Event<T> where 
        <T as system::Trait>::Hash
    {
        Ingress(TransferMessage, Signatures),
        Egress(TransferMessage, Signatures),
        Has(Hash),
    } 
}