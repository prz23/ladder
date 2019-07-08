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

use runtime_io::*;

use order::{OrderPair,OrderT,Symbol};

pub trait Trait: system::Trait + bank::Trait + order::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId
    {
        SellOrder(AccountId,OrderPair,u64,Symbol,Symbol,u128), // seller pair index amount price uniqueindex
        Buy(AccountId,AccountId,OrderPair,u64,Symbol,u128,bool), //buyer seller pair index amount uniqueindex
        CancelsellOrder(AccountId,OrderPair,u64,u128),
        MatchOrder(u128, Symbol, AccountId, u64, bool, AccountId, u64, bool),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Otc {

     }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// add an exchange pair
        pub fn new_pair(origin,pair: OrderPair) -> Result {
            <order::Module<T>>::add_pair(pair)?;
            Ok(())
        }

        ///  put up a sell order to sell the amount of pair.share for per_price of pair.money
        pub fn put_order(origin, pair_type:OrderPair,amount:u64, per_price:u64,acc:Vec<u8>,reserved:bool) -> Result{
            let sender = ensure_signed(origin)?;
            // make sure the sell order is valid
            Self::check_valid_order(sender.clone(),pair_type.clone(),amount,per_price)?;
            // generate a new sell order , lock the money in bank,deposit_event and put up the order
            Self::generate_new_sell_order_and_put_into_list(sender.clone(),pair_type.clone(),amount,per_price,reserved,acc);

            Ok(())
        }

        /// chose an sell order to buy the amount of pair.share
        pub fn buy(origin, seller:T::AccountId, pair:OrderPair ,index:u64, amount:u64,acc:Vec<u8>,reserved:bool) -> Result{
            let buyer = ensure_signed(origin)?;

            // find the sell order
            if let Some(mut sellorder) = <order::Module<T>>::sell_order_of((seller,pair,index)){
                // make sure the buy operate is valid
                Self::check_valid_buy(buyer.clone(),amount,sellorder.clone())?;
                // do the buy operate and modify the order's status
                Self::buy_operate(buyer.clone(),sellorder.clone(),amount,reserved,acc)?;
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }

        pub fn cancel_order(origin,pair_type:OrderPair,index:u64) -> Result {
            let sender = ensure_signed(origin)?;

            // find  the order
            if let Some(mut sellorder) = <order::Module<T>>::sell_order_of((sender.clone(),pair_type,index)){
                // cancel the sell order and unlock the share not deal yet
                Self::cancel_order_operate(sender.clone(),sellorder)?;
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }

        pub fn match_order_verification(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {

    /// Query the Bank Module，make sure the seller have enough money to sell
    /// and check the parmeters is not zero
    fn check_valid_order(who: T::AccountId, pair:OrderPair, amount:u64, price:u64) -> Result {

        <order::Module<T>>::check_valid_order(who.clone(),&pair,amount,price)?;

        //bank---->pair.share  money enough?
        let deposit_data = <bank::Module<T>>::despositing_banance(&who);
        if  deposit_data == [].to_vec() { return Err("no data"); }
        let mut  not_enough_money_error = false;
        deposit_data.iter().enumerate().for_each(|(i,&(balance,coin))|{
            if coin == pair.share {
                if balance < T::Balance::sa(amount) {
                    not_enough_money_error = true;
                }
            }
        });
        if not_enough_money_error == true { return Err("not_enough_money_error "); }
        Ok(())
    }

    fn check_valid_buy(who:T::AccountId,amount:u64,sellorder:OrderT<T>) -> Result {

        let mut sell_order = sellorder.clone();
        <order::Module<T>>::check_valid_buy(who.clone(),amount,&mut sell_order)?;

        //bank---->pair.share  money enough?
        let deposit_data = <bank::Module<T>>::despositing_banance(&who);
        if  deposit_data == [].to_vec() { return Err("no data"); }
        let mut  not_enough_money_error = false;
        deposit_data.iter().enumerate().for_each(|(i,&(balance,coin))|{
            if coin == sell_order.pair.money {
                if balance < T::Balance::sa(amount*sell_order.price) {
                    not_enough_money_error = true;
                }
            }
        });
        if not_enough_money_error == true { return Err("not_enough_money_error "); }
        Ok(())
    }

    fn generate_new_sell_order_and_put_into_list(who:T::AccountId,pair_type:OrderPair,amount:u64,per_price:u64,reserved:bool,acc:Vec<u8>){
        // put sell order in order Module
        <order::Module<T>>::generate_new_sell_order_and_put_into_list(who.clone(),pair_type.clone(),amount,per_price,
                                                                      reserved,acc);
        // lock the specific kingd of money of amount in bank Module
        <bank::Module<T>>::lock(who.clone(),pair_type.share,amount,bank::LockType::OTCLock);
    }

    // buy operate , lock the money and change the status
    fn buy_operate(buyer:T::AccountId,mut sellorder: OrderT<T>, amount:u64, reserved:bool,acc:Vec<u8>) -> Result {

        let mut sell_order = sellorder.clone();
        // Judge and process this buy operation and update the sell order
        <order::Module<T>>::buy_operate(buyer.clone(),&mut sell_order,amount,reserved,acc)?;

        // exchange/unlock/lock the token in bank for buyer & seller
        <bank::Module<T>>::buy_operate(buyer.clone(),sell_order.who.clone(),sell_order.pair.share.clone(),
                                       sell_order.pair.money.clone(),sell_order.price.clone(),
                                       amount,sell_order.reserved(),reserved);

        Ok(())
    }

    /// cancel the sell order
    pub fn cancel_order_operate(who:T::AccountId,mut sell_order:OrderT<T>) -> Result{

        match <order::Module<T>>::cancel_order_operate(who.clone(),&mut sell_order){
            Err("Has been cancelled")  => return Ok(()),
            Ok(()) => {
                // unlock the left share of seller
                let left_shares = sell_order.amount - sell_order.already_deal;
                <bank::Module<T>>::unlock(who.clone(),sell_order.pair.share,left_shares,bank::LockType::OTCLock);
            },
            _ => return Err("unknown err"),
        }
        Ok(())
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
    #[cfg(feature = "std")]
    use rustc_hex::*;


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

    impl Trait for Test {
        type Event = ();
    }

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }
    type Bank = bank::Module<Test>;
    type Order = order::Module<Test>;
    type OTC = Module<Test>;

    #[test]
    fn order_pair_test() {
        with_externalities(&mut new_test_ext(), || {
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(Order::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(Order::is_valid_pair(&pair));
        });
    }

    #[test]
    fn put_order_buy_test() {
        with_externalities(&mut new_test_ext(), || {
            let acc : Vec<u8> = [2,3,4,5].to_vec();
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(Order::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(Order::is_valid_pair(&pair));

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 0 , 10 ,acc.clone(),true) ,"price cann't be 0");
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 0 ,acc.clone(),true) ,"price cann't be 0");

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 10 ,acc.clone(),true) ,"no data");
            assert_err!(OTC::put_order(Some(2).into() , pair.clone(), 10 , 10 ,acc.clone(),true) ,"no data");

            Bank::depositing_withdraw_record(1,50,1,true);
            Bank::depositing_withdraw_record(1,50,2,true);
            Bank::depositing_withdraw_record(1,50,3,true);
            assert_eq!(Bank::despositing_account(),vec![1]);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (50, 1), (50, 2), (50, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ,acc.clone(),true));
            let aa = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (40, 1), (50, 2), (50, 3), (0, 4)].to_vec());
            assert_eq!(Bank::despositing_banance_reserved(1),[(0, 0), (10, 1), (0, 2), (0, 3), (0, 4)].to_vec());

            Bank::depositing_withdraw_record(2,50,1,true);
            Bank::depositing_withdraw_record(2,50,2,true);
            assert_eq!(Bank::despositing_banance(2),[(0, 0), (50, 1), (50, 2), (0, 3), (0, 4)].to_vec());
            assert_ok!(OTC::buy(Some(2).into(),1, pair.clone(),1,5 ,acc.clone(),true));
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (40, 1), (100, 2), (50, 3), (0, 4)].to_vec());
            assert_eq!(Bank::despositing_banance_reserved(1),[(0, 0), (5, 1), (0, 2), (0, 3), (0, 4)].to_vec());

            assert_eq!(Bank::despositing_banance(2),[(0, 0), (55, 1), (0, 2), (0, 3), (0, 4)].to_vec());
            let bb = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(bb.already_deal,5);

            assert_err!(OTC::buy(Some(2).into(),1, pair.clone(),1,1,acc.clone(),true),"not_enough_money_error ");

        });
    }

    #[test]
    fn lock_test() {
        with_externalities(&mut new_test_ext(), || {
            let acc : Vec<u8> = [2,3,4,5].to_vec();
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(Order::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(Order::is_valid_pair(&pair));


            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 0 , 10, acc.clone(),true ) ,"price cann't be 0");
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 0 ,acc.clone(),true) ,"price cann't be 0");

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 10 ,acc.clone(),true) ,"no data");
            assert_err!(OTC::put_order(Some(2).into() , pair.clone(), 10 , 10 ,acc.clone(),true) ,"no data");

            Bank::depositing_withdraw_record(1,50,1,true);
            Bank::depositing_withdraw_record(1,50,2,true);
            Bank::depositing_withdraw_record(1,50,3,true);
            assert_eq!(Bank::despositing_account(),vec![1]);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (50, 1), (50, 2), (50, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ,acc.clone(),false));
            let aa = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();

            Bank::depositing_withdraw_record(2,50,1,true);
            Bank::depositing_withdraw_record(2,50,2,true);
            assert_eq!(Bank::despositing_banance(2),[(0, 0), (50, 1), (50, 2), (0, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ,acc.clone(),true));
            let aa = Order::sell_order_of( (1, pair.clone(), 2) ).unwrap();
            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 30, 10 ,acc.clone(),true));

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ,acc.clone(),true),"not_enough_money_error ");
        });
    }

    #[test]
    fn cancel_order_test() {
        with_externalities(&mut new_test_ext(), || {
            let acc : Vec<u8> = [2,3,4,5].to_vec();
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(Order::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(Order::is_valid_pair(&pair));

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 0 , 10 ,acc.clone(),true) ,"price cann't be 0");
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 0 ,acc.clone(),true) ,"price cann't be 0");

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 10 ,acc.clone(),true) ,"no data");
            assert_err!(OTC::put_order(Some(2).into() , pair.clone(), 10 , 10 ,acc.clone(),true) ,"no data");

            Bank::depositing_withdraw_record(1,50,1,true);
            Bank::depositing_withdraw_record(1,50,2,true);
            Bank::depositing_withdraw_record(1,50,3,true);
            assert_eq!(Bank::despositing_account(),vec![1]);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (50, 1), (50, 2), (50, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ,acc.clone(),true));
            // a new order's status is new
            let aa = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(aa.status,order::OtcStatus::New);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (40, 1), (50, 2), (50, 3), (0, 4)].to_vec());
            assert_eq!(Bank::despositing_banance_reserved(1),[(0, 0), (10, 1), (0, 2), (0, 3), (0, 4)].to_vec());

            // an order was put up , cancel it, and see the status changes into done
            assert_ok!(OTC::cancel_order(Some(1).into() , pair.clone(),1));
            let bb = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(bb.status,order::OtcStatus::Done);
        });
    }

    #[test]
    fn cancel_all_order_test() {
        with_externalities(&mut new_test_ext(), || {
            let acc : Vec<u8> = [2,3,4,5].to_vec();
            let pair:OrderPair = OrderPair{ share:3 ,money:2};
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));

            let pair2:OrderPair = OrderPair{ share:3 ,money:1};
            assert_ok!(OTC::new_pair(Some(1).into() , pair2.clone()));


            Bank::depositing_withdraw_record(1,55,3,true);
            assert_eq!(Bank::despositing_account(),vec![1]);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (0, 1), (0, 2), (55, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 15, 10 ,acc.clone(),true));
            // a new order's status is new
            let aa = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(aa.status,order::OtcStatus::New);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (0, 1), (0, 2), (40, 3), (0, 4)].to_vec());
            assert_eq!(Bank::despositing_banance_reserved(1),[(0, 0), (0, 1), (0, 2), (15, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair2.clone(), 15, 10 ,acc.clone(),true));
            // an account 1 has type3 token 40 free and 15 locked by put up sell order
            // now cancel all, the status turn into done
            Order::cancel_order_for_bank_withdraw(1,pair.share);

            // all status into done
            let cc = Order::sell_order_of((1, pair.clone(), 1) ).unwrap();
            assert_eq!(cc.status,order::OtcStatus::Done);
            let dd = Order::sell_order_of((1, pair2.clone(), 1) ).unwrap();
            assert_eq!(dd.status,order::OtcStatus::Done);
        });
    }

    #[test]
    fn settlement_test() {
        with_externalities(&mut new_test_ext(), || {
        let mut message : Vec<u8>= "00000000000000010000000000000002f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adadf499ed0e7a5c28bcf610ff4c866c8e5ea421f63c21a5c6004bc50b4dc4810ff72c81b6161f6bf7f76e635b161a2535f3b221bec1000000000000000000000000000000000000000000000000000000000000000101".from_hex().unwrap();
        let sign : Vec<u8>= "4625ad0747cc75ab29c97a69ef561c2a7d154e7ec90b180d37df2b7a85ec6fb35588f5b38ca1af1e8e8a469114edecd05689c143e0cdb5ae032c349d0c22ae061b".from_hex().unwrap();

        assert_ok!(Order::match_order_verification(Some(1).into(),message,sign));
        });
    }

    #[test]
    fn basic_withdraw_request_test() {
        with_externalities(&mut new_test_ext(), || {
            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::despositing_account(),[11744161374129632607].to_vec());
            assert_eq!(Bank::despositing_banance(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (0, 3), (0, 4)].to_vec());


            //assert_eq!(Bank::coin_deposit(0),<tests::Test as Trait>::Balance::sa(0));
            let mut data2 : Vec<u8>= "00000000000000010000000000000000000000000000000000000000000000000000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea4".from_hex().unwrap();
            let sign2 : Vec<u8>= "b36bba3f9e7138e45b9ff9918a0759623ca146b3956174efaadb37635c2adb440f9fd75e7773803337d4802d94f5c78788121dccd4b698080b047171966483711b".from_hex().unwrap();
            assert_ok!(Bank::withdrawrequest(Origin::signed(5),data2,sign2));
            assert_eq!(Bank::despositing_banance(11744161374129632607),[].to_vec());
            assert_eq!(Bank::despositing_banance_withdraw(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (0, 3), (0, 4)]);
        });
    }

    #[test]
    fn withdraw_request_withdraw_sell_order_test() {
        with_externalities(&mut new_test_ext(), || {
            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::despositing_account(),[11744161374129632607].to_vec());
            assert_eq!(Bank::despositing_banance(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (0, 3), (0, 4)].to_vec());

            // put order
            let acc : Vec<u8> = [2,3,4,5].to_vec();
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(OTC::put_order(Some(11744161374129632607).into() , pair.clone(), 10, 10 ,acc.clone(),true));
            assert_eq!(Bank::despositing_banance(11744161374129632607),[(0, 0), (9999990, 1), (0, 2), (0, 3), (0, 4)].to_vec());
            assert_eq!(Bank::despositing_banance_reserved(11744161374129632607),[(0, 0), (10, 1), (0, 2), (0, 3), (0, 4)].to_vec());


            //assert_eq!(Bank::coin_deposit(0),<tests::Test as Trait>::Balance::sa(0));
            let mut data2 : Vec<u8>= "00000000000000010000000000000000000000000000000000000000000000000000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea4".from_hex().unwrap();
            let sign2 : Vec<u8>= "b36bba3f9e7138e45b9ff9918a0759623ca146b3956174efaadb37635c2adb440f9fd75e7773803337d4802d94f5c78788121dccd4b698080b047171966483711b".from_hex().unwrap();
            assert_ok!(Bank::withdrawrequest(Origin::signed(5),data2,sign2));
            assert_eq!(Bank::despositing_banance(11744161374129632607),[].to_vec());
            assert_eq!(Bank::despositing_banance_withdraw(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (0, 3), (0, 4)]);


        });
    }
}