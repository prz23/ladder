#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

use sr_primitives::traits::{CheckedAdd, CheckedSub, Hash, Verify, Zero};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, Parameter, StorageMap,
    StorageValue,
};

use system::ensure_signed;

use rstd::marker::PhantomData;
use rstd::prelude::*;

#[cfg(feature = "std")]
pub use std::fmt;

// use Encode, Decode
use parity_codec::{Decode, Encode};
use rstd::ops::Div;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId
    {
        SetMinRequreSignatures(u64),
        TranscationVerified(u64,Vec<(AccountId,u64)>),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Exchange {
        // each coin => amount of exchanged
        TradingVolumeTotal get(trading_volume_total): map u64 => u128;

        // (who,coin) => amount of exchanged
        TradingVolumePerson get(trading_volume_person) : map (T::AccountId,u64) => u128;

        //
        TransactionsQuantityTotal get(transactions_quantity_total) : map u64 => u64;

        //
        TransactionsQuantityPerson get(transactions_quantity_person) : map (T::AccountId,u64) => u64;

        //
        PeriodicHoldings get(periodic_holdings) : map (T::AccountId,u64) => u128;

        // time period (block)
        TimePeriod get(time_period) : u64 = 1000;


    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        ///
        pub  fn set_min_num(origin,new_num: u64) -> Result{

            Ok(())
        }


    }
}

impl<T: Trait> Module<T> {
    fn verify(_tx: T::Hash) -> Result {

        Ok(())
    }

    //Basic storage operations
    ///increase amount of coin_type to the total volume record
    pub fn increase_trading_volume_total(coin_type:u64, amount:u64) {
        let new_volume = amount as u128 + Self::trading_volume_total(coin_type);
        <TradingVolumeTotal<T>>::insert(coin_type,new_volume);
    }

    pub fn increase_trading_volume_person(who:&T::AccountId,coin_type:u64,amount:u64){
        let new_volume = amount as u128 + Self::trading_volume_person((who.clone(),coin_type));
        <TradingVolumePerson<T>>::insert((who.clone(),coin_type),new_volume);
    }

    pub fn number_of_trading_total(coin_type:u64){
        let new_volume = 1u128 + Self::transactions_quantity_total(coin_type) as u128;
        <TradingVolumeTotal<T>>::insert(coin_type,new_volume as u128);
    }

    pub fn number_of_trading_person(who:&T::AccountId,coin_type:u64){
        let new_times = 1 + Self::transactions_quantity_person((who.clone(),coin_type));
        <TransactionsQuantityPerson<T>>::insert((who.clone(),coin_type),new_times);
    }

    /// in OTC module , when order settle , record data , in bank each period of session clean the data
    pub fn volum_and_number_rocord(seller:&T::AccountId, share:u64, buyer:&T::AccountId, money:u64, amount_s:u64, amount_b:u64){
        Self::increase_trading_volume_total(share,amount_s);
        Self::increase_trading_volume_total(money,amount_b);

        Self::increase_trading_volume_person(seller,share,amount_s);
        Self::increase_trading_volume_person(buyer,money,amount_b);

        Self::number_of_trading_total(share);
        Self::number_of_trading_total(money);

        Self::number_of_trading_person(seller,share);
        Self::number_of_trading_person(buyer,money);
    }

    pub fn calculate_a_b(who:&T::AccountId,coin_type:u64) -> (f64,f64){
        let a = (Self::trading_volume_person((who.clone(),coin_type)) as f64) / (Self::trading_volume_total(coin_type) as f64);
        let b = (Self::transactions_quantity_person((who.clone(),coin_type)) as f64) / (Self::transactions_quantity_total(coin_type) as f64);
        (a,b)
    }

    // in bank iterator_all_token() and calculate the amout
    pub fn calculate_reward_for_each(who:&T::AccountId,coin_type:u64){
        let (a,b) = Self::calculate_a_b(who,coin_type);
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use support::{impl_outer_origin, assert_ok,assert_err};
    use runtime_io::with_externalities;
    use primitives::{H256, Blake2Hasher};
    use sr_primitives::BuildStorage;
    use sr_primitives::traits::{BlakeTwo256, IdentityLookup};
    use sr_primitives::testing::{Digest, DigestItem, Header, UintAuthorityId, ConvertUintAuthorityId};
    use support::{StorageMap,StorageValue};

    impl_outer_origin!{
		pub enum Origin for Test {}
	}

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<u64>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }


    impl Trait for Test {
        type Event = ();
    }

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }

    type statistics = Module<Test>;

    #[test]
    fn resolving_data_test() {
        with_externalities(&mut new_test_ext(), || {
            //
        });
    }
}