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

pub trait Trait: system::Trait + bank::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// 记录不同币种的类型 1 eth   2 btc  等等
pub type Symbol = u64;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OrderPair {
    #[codec(compact)]
    pub share: Symbol,      // 股票
    #[codec(compact)]
    pub money: Symbol,      // 钱
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OrderContent<pair,AccountID,symbol,status> {
    pub pair: pair,          // exchange pair
    pub index: u64,          // sell order index assigned for a specific user
    pub who: AccountID,      //  seller
    pub amount: symbol,      // share
    pub price: symbol,       // price
    pub already_deal:symbol, // already sold
    pub status: status,      // sell order status
}

impl<pair,AccountID,symbol,status> OrderContent<pair,AccountID,symbol,status>{
    pub fn parse_order_data(&self) {

    }
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Status {
    New,
    Half,
    Done,
}

pub type OrderT<T> = OrderContent<
    OrderPair,
    <T as system::Trait>::AccountId,
    Symbol,
    Status,
>;

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
    {
        SetMinRequreSignatures(AccountId,Hash),
        SellOrder(AccountId,OrderPair,u64,Symbol,Symbol), // seller pair index amount price
        Buy(AccountId,OrderPair,u64,Symbol), //seller pair index amount
        CancelsellOrder(AccountId,OrderPair,u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Signature {

        /// all seller order infomation
        pub SellOrdersOf get(sell_order_of):map (T::AccountId, OrderPair, u64) => Option<OrderT<T>>;

        /// exist exchange pair list
        pub OrderPairList get(pair_list):  Vec<OrderPair>;

        /// sell order index assigned for a specific user
        pub LastSellOrderIndexOf get(last_sell_order_index_of): map(T::AccountId,OrderPair)=>Option<u64>;
     }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// add an exchange pair
        pub fn new_pair(origin,pair: OrderPair) -> Result {
            Self::add_pair(pair)?;
            Ok(())
        }

        ///  put up a sell order to sell the amount of pair.share for per_price of pair.money
        pub fn put_order(origin, pair_type:OrderPair,amount:u64, per_price:u64) -> Result{
            let sender = ensure_signed(origin)?;

            // make sure the sell order is valid
            Self::check_valid_order(sender.clone(),pair_type.clone(),amount,per_price)?;

            // generate a new sell order , lock the money in bank,deposit_event and put up the order
            Self::generate_new_sell_order_and_put_into_list(sender.clone(),pair_type.clone(),amount,per_price);

            Ok(())
        }

        /// chose an sell order to buy the amount of pair.share
        pub fn buy(origin, seller:T::AccountId, pair:OrderPair ,index:u64, amount:u64) -> Result{
            let buyer = ensure_signed(origin)?;

            // find the sell order
            if let Some(mut sellorder) = Self::sell_order_of((seller,pair,index)){

                // make sure the buy operate is valid
                Self::check_valid_buy(buyer.clone(),amount,sellorder.clone())?;
                // do the buy operate and modify the order's status
                Self::buy_operate(buyer.clone(),sellorder.clone(),amount);
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }


        pub fn cancel_order(origin,pair_type:OrderPair,index:u64) -> Result {
            let sender = ensure_signed(origin)?;

            // find  the order
            if let Some(mut sellorder) = Self::sell_order_of((sender.clone(),pair_type,index)){
                // cancel the sell order and return the share not exchanged
                Self::cancel_order_operate(sender.clone(),sellorder)?;
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    // add a new exchange pair
    pub fn add_pair(pair: OrderPair) -> Result {
        if let Err(_) = Self::is_valid_pair(&pair) {
            let mut pair_list: Vec<OrderPair> = <OrderPairList<T>>::get();
            pair_list.push(pair);
            <OrderPairList<T>>::put(pair_list);
        }
        Ok(())
    }


    fn is_price_zero(price:u64) -> Result {
        if price == Zero::zero() {
            return Err("price cann't be 0");
        }
        Ok(())
    }

    ///
    fn is_valid_pair(pair: &OrderPair) -> Result {
        let pair_list: Vec<OrderPair> = <OrderPairList<T>>::get();
        if pair_list.contains(pair) {
            Ok(())
        } else {
            Err("an invalid orderpair")
        }
    }

    /// Query the Bank Module，make sure the seller have enough money to sell
    /// and check the parmeters is not zero
    fn check_valid_order(who: T::AccountId, pair:OrderPair, amount:u64, price:u64) -> Result {
        //
        Self::is_valid_pair(&pair)?;

        //
        Self::is_price_zero(amount)?;
        Self::is_price_zero(price)?;

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

    fn check_valid_buy(who:T::AccountId,amount:u64,sell_order:OrderT<T>) -> Result {
        //
        Self::is_price_zero(amount)?;

        // cant buy exceed the sell order
        let left_share = sell_order.amount - sell_order.already_deal;
        if amount > left_share {
            return Err("cant buy too much!");
        }
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

    fn generate_new_sell_order_and_put_into_list(who:T::AccountId, pair_type:OrderPair,amount:u64, per_price:u64 ){
        // assign a new sell order index
        let new_last_index = Self::last_sell_order_index_of((who.clone(), pair_type.clone())).unwrap_or_default() + 1;
        <LastSellOrderIndexOf<T>>::insert((who.clone(), pair_type.clone()), new_last_index);
        // generate a new sell order
        let mut new_sell_order :OrderT<T> = OrderContent{
            pair:pair_type.clone(),
            index:new_last_index,
            who:who.clone(),
            amount: amount,      // pair.share的挂单数量
            price: per_price,    // pair.money的单价
            already_deal:0,      // 已经交易掉的数量
            status: Status::New, //交易当前状态
        };
        <SellOrdersOf<T>>::insert((who.clone(), pair_type.clone(), new_last_index), new_sell_order.clone());
        // lock the money in bank with the amount of the sell order
        <bank::Module<T>>::lock(who.clone(),pair_type.share,amount);

        //deposit_event
        Self::deposit_event(RawEvent::SellOrder(who.clone(),pair_type.clone(),
                                                new_last_index,amount,per_price,));
    }

    // buy operate , lock the money and change the status
    fn buy_operate(buyer:T::AccountId,mut sell_order: OrderT<T>, amount:u64) -> Result {
        // modify the exchanged money
        sell_order.already_deal = sell_order.already_deal+amount;
        // change sell order status
        if sell_order.already_deal < sell_order.amount {
            sell_order.status = Status::Half;
        }else if sell_order.amount == sell_order.already_deal {
            sell_order.status = Status::Done;
        }else { return  Err("wrong!"); }

        // save the modified sell order
        <SellOrdersOf<T>>::insert((sell_order.who.clone(),sell_order.pair.clone(),sell_order.index.clone()),
                              sell_order.clone());
        //exchange the money in bank
        <bank::Module<T>>::buy_operate(buyer.clone(),sell_order.who.clone(),sell_order.pair.share.clone(),
                                       sell_order.pair.money.clone(),sell_order.price.clone(),
                                       amount);
        //deposit_event
        Self::deposit_event(RawEvent::Buy(buyer.clone(),sell_order.pair.clone(),
                                          sell_order.index, amount, ));
        Ok(())
    }

    /// cancel the sell order
    pub fn cancel_order_operate(who:T::AccountId,mut sell_order:OrderT<T>) -> Result{
        // if the order is in Done status ,return directly
        if sell_order.status == Status::Done { return Ok(()) ;}

        // calculate the left money in sell order
        let left_shares = sell_order.amount - sell_order.already_deal;
        // unlock the left
        <bank::Module<T>>::unlock(who.clone(),sell_order.pair.share,left_shares);
        //modify the status to done
        sell_order.status = Status::Done;
        <SellOrdersOf<T>>::insert((who.clone(), sell_order.pair.clone(), sell_order.index), sell_order.clone());

        //deposit_event
        Self::deposit_event(RawEvent::CancelsellOrder(who.clone(), sell_order.pair.clone(), sell_order.index, ));
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

    impl Trait for Test {
        type Event = ();
    }

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }
    type Bank = bank::Module<Test>;
    type OTC = Module<Test>;

    #[test]
    fn order_pair_test() {
        with_externalities(&mut new_test_ext(), || {
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(OTC::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(OTC::is_valid_pair(&pair));
        });
    }


    #[test]
    fn put_order_test() {
        with_externalities(&mut new_test_ext(), || {
            // 先安排一个 1 ，2  的交易对
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(OTC::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(OTC::is_valid_pair(&pair));
            // 挂2个失败的单子 价格 数量都不能为零
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 0 , 10 ) ,"price cann't be 0");
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 0 ) ,"price cann't be 0");

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 10 ) ,"no data");
            assert_err!(OTC::put_order(Some(2).into() , pair.clone(), 10 , 10 ) ,"no data");
            // 挂上了
           // assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ));
            //有一个挂那里了
            //let aa = OTC::order_of( (1, pair.clone(), 1) ).unwrap();

            // 往账户里预先存点
            Bank::depositing_withdraw_record(1,50,1,true);
            Bank::depositing_withdraw_record(1,50,2,true);
            Bank::depositing_withdraw_record(1,50,3,true);
            assert_eq!(Bank::despositing_account(),vec![1]);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (50, 1), (50, 2), (50, 3), (0, 4)].to_vec());
            // 再挂单，挂上了
            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ));
            let aa = OTC::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (40, 1), (50, 2), (50, 3), (0, 4)].to_vec());
            assert_eq!(Bank::despositing_banance_reserved(1),[(0, 0), (10, 1), (0, 2), (0, 3), (0, 4)].to_vec());
            // 再安排个铁子，让他买一下
            Bank::depositing_withdraw_record(2,50,1,true);
            Bank::depositing_withdraw_record(2,50,2,true);
            assert_eq!(Bank::despositing_banance(2),[(0, 0), (50, 1), (50, 2), (0, 3), (0, 4)].to_vec());
            assert_ok!(OTC::buy(Some(2).into(),1, pair.clone(),1,5)); // 买4个币，花了4x10=40块 剩下10块
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (40, 1), (100, 2), (50, 3), (0, 4)].to_vec());
            assert_eq!(Bank::despositing_banance_reserved(1),[(0, 0), (5, 1), (0, 2), (0, 3), (0, 4)].to_vec());

            assert_eq!(Bank::despositing_banance(2),[(0, 0), (50, 1), (0, 2), (0, 3), (0, 4)].to_vec());
            let bb = OTC::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(bb.already_deal,5);
            // 继续买就会说没钱，
            assert_err!(OTC::buy(Some(2).into(),1, pair.clone(),1,1),"not_enough_money_error ");
            //assert_ok!(OTC::buy(Some(2).into(),1, pair.clone(),1,2));  //再买1个 ，剩下0元
           // assert_ok!(OTC::buy(Some(2).into(),1, pair.clone(),1,7));
        });
    }

    #[test]
    fn lock_test() {
        with_externalities(&mut new_test_ext(), || {
            // 先安排一个 1 ，2  的交易对
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(OTC::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(OTC::is_valid_pair(&pair));
            // 挂2个失败的单子 价格 数量都不能为零
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 0 , 10 ) ,"price cann't be 0");
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 0 ) ,"price cann't be 0");

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 10 ) ,"no data");
            assert_err!(OTC::put_order(Some(2).into() , pair.clone(), 10 , 10 ) ,"no data");
            // 挂上了
            // assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ));
            //有一个挂那里了
            //let aa = OTC::order_of( (1, pair.clone(), 1) ).unwrap();

            // 往账户里预先存点
            Bank::depositing_withdraw_record(1,50,1,true);
            Bank::depositing_withdraw_record(1,50,2,true);
            Bank::depositing_withdraw_record(1,50,3,true);
            assert_eq!(Bank::despositing_account(),vec![1]);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (50, 1), (50, 2), (50, 3), (0, 4)].to_vec());
            // 再挂单，挂上了
            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ));
            let aa = OTC::sell_order_of( (1, pair.clone(), 1) ).unwrap();

            // 再安排个铁子，让他买一下
            Bank::depositing_withdraw_record(2,50,1,true);
            Bank::depositing_withdraw_record(2,50,2,true);
            assert_eq!(Bank::despositing_banance(2),[(0, 0), (50, 1), (50, 2), (0, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ));
            let aa = OTC::sell_order_of( (1, pair.clone(), 2) ).unwrap();
            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 30, 10 ));
            // 挂单过多，本身只有50块钱，挂出去50，再挂10 就会报错，钱不够
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ),"not_enough_money_error ");
        });
    }
    #[test]
    fn cancel_order_test() {
        with_externalities(&mut new_test_ext(), || {

        });
    }
}