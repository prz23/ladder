#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

use sr_primitives::traits::{As,CheckedAdd, CheckedSub, Hash, Verify, Zero};
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
use session::*;

pub trait Trait: system::Trait + session::Trait + otc::Trait + bank::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId
    {
        TestEvent(u64,Vec<(AccountId,u64)>),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Ladsession {

       NewSessionCount get(new_session_count) : u64 = 0;

       NewSession get(new_session) : u64 = 100;

       pub TotalRewardPerson get(total_reward_person) : map T::AccountId => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;
    }
}

impl<T: Trait> Module<T> {

    pub fn new_session_start(elapsed: T::Moment, should_reward: bool){
        Self::transaction_information_processing();
    }

    /// use Participant and AllConinType Vector to calculate the ratio of reward
    pub fn count_rewards_and_grant() {
        //
        let reward_amount_volume_real = 1000000u64  ; //TODO::replace with real amount of reward
        let reward_amount_number_real = 1000000u64; //TODO::replace with real amount of reward
        let reward_amount_deposit_real = 1000000u64; //TODO::replace with real amount of reward


        // trade reward
        let participant_vec = <otc::Module<T>>::participant();
        participant_vec.iter().enumerate().for_each(|(i,(who,coin))|{

            //TODO::replace with real amount of reward
            let reward_amount_volume = (reward_amount_volume_real * <otc::Module<T>>::exchange_to_lad(coin) )as f64 / 10000f64 ;
            let reward_amount_number = (reward_amount_number_real * <otc::Module<T>>::exchange_to_lad(coin) )as f64 / 10000f64 ;


            let volume_ratio = <otc::Module<T>>::trading_volume_person((who.clone(),*coin)) as f64 / <otc::Module<T>>::trading_volume_total(coin) as f64;
            let number_ratio = <otc::Module<T>>::transactions_quantity_person((who.clone(),*coin)) as f64 / <otc::Module<T>>::transactions_quantity_total(coin) as f64;

            let reward = ( reward_amount_volume as f64 * volume_ratio + reward_amount_number * number_ratio )as u64;
            <TotalRewardPerson<T>>::mutate(who,|balance| *balance += reward );
            <bank::Module<T>>::deposit_reward(who,reward);
        });
        // deposit reward
        <bank::Module<T>>::iterator_all_token(|accountid,coin_type,sender|{
            //TODO:: coin exchangerate
            let reward_amount_deposit = (reward_amount_deposit_real * <otc::Module<T>>::exchange_to_lad(coin_type) )as f64 / 10000f64 ;

            let total_token_for_this_coin = <bank::Module<T>>::total_token_for_specific_coin(&accountid,&sender,coin_type);
            let reward = ((<bank::Module<T>>::total_token_for_specific_coin(accountid,&sender,coin_type) as f64
                               / T::Balance::as_(<bank::Module<T>>::coin_deposit(coin_type)) as f64) * reward_amount_deposit ) as u64;
            <bank::Module<T>>::deposit_reward(accountid,reward);
            Ok(())
        });

    }

    pub fn is_new_session() -> bool {
        if Self::new_session_count() >= Self::new_session(){
            <NewSessionCount<T>>::put(0);
            return true;
        }
        <NewSessionCount<T>>::put(Self::new_session_count()+1);
        true
    }

    pub fn transaction_information_processing(){
        //a new session start , clear the supprot vector contains all active buyers and sellers in last session
        if Self::is_new_session() {
            <otc::Module<T>>::periodical_clean();
        }
        // then
        Self::count_rewards_and_grant();
    }

}

impl<T: Trait> OnSessionChange<T::Moment> for Module<T> {
    fn on_session_change(elapsed: T::Moment, should_reward: bool) {
        runtime_io::print("LadderSession");
        Self::new_session_start(elapsed, should_reward);
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