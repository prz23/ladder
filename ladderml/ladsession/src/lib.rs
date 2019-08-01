#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

use sr_primitives::traits::As;
use support::{
    decl_event, decl_module, decl_storage, StorageMap,
    StorageValue,
};

use rstd::prelude::*;

#[cfg(feature = "std")]
pub use std::fmt;

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

    pub fn new_session_start(_elapsed: T::Moment, _should_reward: bool){
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
        participant_vec.iter().enumerate().for_each(|(_i,(who,coin))|{

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

            //let total_token_for_this_coin = <bank::Module<T>>::total_token_for_specific_coin(&accountid,&sender,coin_type);
            let reward = ((<bank::Module<T>>::total_token_for_specific_coin(accountid,&sender,coin_type) as f64
                               / T::Balance::as_(<bank::Module<T>>::coin_deposit(coin_type)) as f64) * reward_amount_deposit ) as u64;
            <bank::Module<T>>::deposit_reward(accountid,reward);
            Ok(())
        });

    }

    pub fn is_new_session() -> bool {
        let session_count = Self::new_session_count() + 1;
        if session_count >= Self::new_session(){
            <NewSessionCount<T>>::put(0);
            return true;
        }
        <NewSessionCount<T>>::put(session_count);
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
        runtime_io::print("LadderSessionTest");
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
    use order::OrderPair;

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

    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
    }

    impl consensus::Trait for Test {
        type Log = DigestItem;
        type SessionKey = UintAuthorityId;
        type InherentOfflineReport = ();
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }

    impl session::Trait for Test {
        type ConvertAccountIdToSessionKey = ConvertUintAuthorityId;
        type OnSessionChange = ();
        type Event = ();
    }

    impl signcheck::Trait for Test {
        type Event = ();
    }

    impl bank::Trait for Test {
        type Currency = balances::Module<Self>;
        type Event = ();
    }

    impl order::Trait for Test {
        type Event = ();
    }

    impl otc::Trait for Test {
        type Event = ();
    }


    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }

    type Bank = bank::Module<Test>;
    type OTC = otc::Module<Test>;
    type Order = order::Module<Test>;
    /////type Statistics = statistics::Module<Test>;
    type Ladsession = Module<Test>;

    #[test]
    fn ladsession_test() {
        with_externalities(&mut new_test_ext(), || {
            //
            let seller_acc : Vec<u8> = [2,3,4,95].to_vec();
            let seller_acc2 : Vec<u8> = [3,4,5,6,1].to_vec();

            let buyer_acc : Vec<u8> = [10,15,68].to_vec();
            let buyer_acc2 : Vec<u8> = [55,41,12].to_vec();

            let pair: OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(Order::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(Order::is_valid_pair(&pair));

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 0 , 10 ,seller_acc.clone(),seller_acc2.clone(),true) ,"price cann't be 0");
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 0 ,seller_acc.clone(),seller_acc2.clone(),true) ,"price cann't be 0");

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 10 ,seller_acc.clone(),seller_acc2.clone(),true) ,"not_enough_money_error check_valid_order");
            assert_err!(OTC::put_order(Some(2).into() , pair.clone(), 10 , 10 ,seller_acc.clone(),seller_acc2.clone(),true) ,"not_enough_money_error check_valid_order");

            Bank::modify_token(1,seller_acc.clone(),1,50,bank::TokenType::Free,true);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),1)),50);

            assert_ok!(OTC::put_order(Some(1).into(),pair.clone(),10, 1000000 ,seller_acc.clone(),seller_acc2.clone(),true));
            let aa = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),1)),40);
            assert_eq!(Bank::deposit_otc_token((1,seller_acc.clone(),1)),10);


            //buy 4 coin-1 use coin-2
            Bank::modify_token(2,buyer_acc.clone(),2,50,bank::TokenType::Free,true);
            assert_eq!(Bank::deposit_free_token((2,buyer_acc.clone(),2)),50);

            assert_err!(OTC::buy(Some(2).into(),1, pair.clone(),1, 6,buyer_acc.clone(),buyer_acc2.clone(),true),"not_enough_money_error check_valid_buy");
            assert_err!(OTC::buy(Some(2).into(),1, pair.clone(),1,11 ,buyer_acc.clone(),buyer_acc2.clone(),true),"cant buy too much!");


            assert_ok!(OTC::buy(Some(2).into(), 1, pair.clone(), 1, 4, buyer_acc.clone(),buyer_acc2.clone(),true));

            assert_eq!(Bank::deposit_free_token((2,buyer_acc2.clone(),1)),4);  // 0 + 5 =5
            assert_eq!(Bank::deposit_otc_token((1,seller_acc.clone(),1)),6);  // 10 - 4 = 6
            assert_eq!(Bank::deposit_free_token((2,buyer_acc.clone(),2)),10);  // 50 - 40 = 10
            assert_eq!(Bank::deposit_free_token((1,seller_acc2.clone(),2)),40);  // 0 + 40 = 40

            //check the data
            assert_eq!(OTC::trading_volume_person((1,1)),4);
            assert_eq!(OTC::trading_volume_person((2,2)),40);
            assert_eq!(OTC::all_coin_type(),[1,2].to_vec());
            assert_eq!(OTC::participant(),[(1,1),(2,2)].to_vec());
            assert_eq!(OTC::transactions_quantity_person((1,1)),1);
            assert_eq!(OTC::transactions_quantity_person((1,2)),0);
            assert_eq!(OTC::transactions_quantity_person((2,2)),1);
            assert_eq!(OTC::transactions_quantity_person((2,1)),0);
            assert_eq!(OTC::transactions_quantity_total(2),1);

            //clean the statistics
            OTC::periodical_clean();
            assert_eq!(OTC::trading_volume_person((1,1)),4);
            assert_eq!(OTC::trading_volume_person((2,2)),40);
            assert_eq!(OTC::all_coin_type(),[].to_vec());
            assert_eq!(OTC::participant(),[].to_vec());

            //buy again and check the data
            Bank::modify_token(2,buyer_acc.clone(),2,50,bank::TokenType::Free,true);
            assert_ok!(OTC::buy(Some(2).into(), 1, pair.clone(), 1, 2, buyer_acc.clone(),buyer_acc2.clone(),true));

            assert_eq!(OTC::trading_volume_person((1,1)),2);
            assert_eq!(OTC::trading_volume_person((2,2)),20);
            assert_eq!(OTC::all_coin_type(),[1,2].to_vec());
            assert_eq!(OTC::participant(),[(1,1),(2,2)].to_vec());
        });
    }
}