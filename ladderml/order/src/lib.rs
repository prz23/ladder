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

pub trait Trait: system::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type Symbol = u64;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OrderPair {
    #[codec(compact)]
    pub share: Symbol,
    #[codec(compact)]
    pub money: Symbol,
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
    pub longindex: u128,     // an unique index for each sell order
    pub reserved: bool,      // send the balance to bank or contract
    pub acc: Vec<u8>,        // sell's out side account
}

impl<pair,AccountID,symbol,status> OrderContent<pair,AccountID,symbol,status>{
    pub fn reserved(&self) -> bool {
        self.reserved
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum OtcStatus {
    New,
    Half,
    Done,
}

pub type OrderT<T> = OrderContent<
    OrderPair,
    <T as system::Trait>::AccountId,
    Symbol,
    OtcStatus,
>;

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId
    {
        SellOrder(AccountId,OrderPair,u64,Symbol,Symbol,u128), // seller pair index amount price uniqueindex
        Buy(AccountId,AccountId,OrderPair,u64,Symbol,u128,bool), //buyer seller pair index amount uniqueindex
        CancelsellOrder(AccountId,OrderPair,u64,u128),
        MatchOrder(u128, Symbol, AccountId,Vec<u8>, u64, bool, u64, AccountId,Vec<u8>, u64, bool,u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Order {

        /// all seller order infomation
        pub SellOrdersOf get(sell_order_of):map (T::AccountId, OrderPair, u64) => Option<OrderT<T>>;

        /// exist exchange pair list
        pub OrderPairList get(pair_list):  Vec<OrderPair>;

        /// sell order index assigned for a specific user
        pub LastSellOrderIndexOf get(last_sell_order_index_of): map(T::AccountId,OrderPair)=>Option<u64>;

        ///unique index -> all sell orders
        pub AllSellOrders get(all_sell_orders):map u128 => Option<OrderT<T>>;

        pub AllSellOrdersIndex get(all_sell_orders_index): u128;

        // orderpair --> unique index  Valid
        pub ValidOrderIndexByOrderpair get(valid_order_index_by_orderpair) : map OrderPair => Vec<u128>;

     }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;


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
    pub fn is_valid_pair(pair: &OrderPair) -> Result {
        let pair_list: Vec<OrderPair> = <OrderPairList<T>>::get();
        if pair_list.contains(pair) {
            Ok(())
        } else {
            Err("an invalid orderpair")
        }
    }

    /// Query the Bank Module，make sure the seller have enough money to sell
    /// and check the parmeters is not zero
    pub fn check_valid_order(who: T::AccountId, pair: &OrderPair, amount:u64, price:u64) -> Result {
        Self::is_valid_pair(&pair)?;
        Self::is_price_zero(amount)?;
        Self::is_price_zero(price)?;

        Ok(())
    }

    pub fn check_valid_buy(who:T::AccountId,amount:u64,sell_order:&mut OrderT<T>) -> Result {

        Self::is_price_zero(amount)?;
        // cant buy exceed the sell order
        let left_share = sell_order.amount - sell_order.already_deal;
        if amount > left_share {
            return Err("cant buy too much!");
        }

        Ok(())
    }

    pub fn generate_new_sell_order_and_put_into_list(who:T::AccountId,pair_type:OrderPair,amount:u64,per_price:u64,reserved:bool,acc:Vec<u8>){
        // assign a new sell order index
        let new_last_index = Self::last_sell_order_index_of((who.clone(), pair_type.clone())).unwrap_or_default() + 1;
        <LastSellOrderIndexOf<T>>::insert((who.clone(), pair_type.clone()), new_last_index);
        // a new unique index
        let new_unique_index = Self::all_sell_orders_index() + 1;
        <AllSellOrdersIndex<T>>::put(new_unique_index);

        // generate a new sell order
        let mut new_sell_order :OrderT<T> = OrderContent{
            pair:pair_type.clone(),
            index:new_last_index,
            who:who.clone(),
            amount: amount,      // pair.share
            price: per_price,    // pair.money
            already_deal:0,
            status: OtcStatus::New,  //Status
            longindex : new_unique_index,
            reserved : reserved,
            acc: acc,
        };
        // save the sell order in 3 different ways
        <SellOrdersOf<T>>::insert((who.clone(), pair_type.clone(), new_last_index), new_sell_order.clone());
        <AllSellOrders<T>>::insert(new_unique_index,new_sell_order.clone());
        let mut vec = <ValidOrderIndexByOrderpair<T>>::get(pair_type.clone());
        vec.push(new_unique_index);
        <ValidOrderIndexByOrderpair<T>>::insert(pair_type.clone(),vec);

        // lock the money in bank with the amount of the sell order
        // <bank::Module<T>>::lock(who.clone(),pair_type.share,amount);

        //deposit_event
        Self::deposit_event(RawEvent::SellOrder(who.clone(),pair_type.clone(),
                                                new_last_index,amount,per_price,new_unique_index));
    }

    // buy operate , lock the money and change the status
    pub fn buy_operate(buyer:T::AccountId,mut sell_order:&mut OrderT<T>, amount:u64, reserved:bool,acc:Vec<u8>) -> Result {
        // modify the exchanged money
        sell_order.already_deal = sell_order.already_deal+amount;
        // change sell order status
        if sell_order.already_deal < sell_order.amount {
            sell_order.status = OtcStatus::Half;
        }else if sell_order.amount == sell_order.already_deal {
            sell_order.status = OtcStatus::Done;
            Self::delete_from_pair_order_storage(sell_order.clone());
        }else { return  Err("overflow"); }

        // update the modified sell order
        <SellOrdersOf<T>>::insert((sell_order.who.clone(),sell_order.pair.clone(),sell_order.index.clone()),
                              sell_order.clone());
        <AllSellOrders<T>>::insert(sell_order.longindex,sell_order.clone());

        //deposit_event
        //MatchOrder(id, price, seller, saleAmount, reserved, buyer, purchasingAmount, reserved);
        Self::deposit_event(RawEvent::MatchOrder(sell_order.longindex,sell_order.price,sell_order.who.clone(),
                                                 sell_order.acc.clone(),amount,sell_order.reserved(),sell_order.pair.share,
                                                 buyer.clone(),acc,sell_order.price*amount,reserved,sell_order.pair.money));
        Self::deposit_event(RawEvent::Buy(buyer.clone(),sell_order.who.clone(),sell_order.pair.clone(),
                                          sell_order.index, amount, sell_order.longindex,reserved));
        Ok(())
    }

    /// cancel the sell order
    pub fn cancel_order_operate(who:T::AccountId,mut sell_order:&mut OrderT<T>) -> Result{
        // if the order is in Done status ,return directly
        if sell_order.status == OtcStatus::Done { return Err("Has been cancelled") ;}

        // modify the status to done
        sell_order.status = OtcStatus::Done;
        // update the sell order in 3 different storage structure
        <SellOrdersOf<T>>::insert((who.clone(), sell_order.pair.clone(), sell_order.index), sell_order.clone());
        <AllSellOrders<T>>::insert(sell_order.longindex, sell_order.clone());
        Self::delete_from_pair_order_storage(sell_order.clone());

        // deposit_event
        Self::deposit_event(RawEvent::CancelsellOrder(who.clone(), sell_order.pair.clone(), sell_order.index,
                                                      sell_order.longindex));
        Ok(())
    }

    pub fn delete_from_pair_order_storage(sell_order: OrderT<T>){
        let mut vec = <ValidOrderIndexByOrderpair<T>>::get(sell_order.pair.clone());
        let mut  mark = 0usize;
        vec.iter().enumerate().for_each(|(i,&index)|{
            if index == sell_order.longindex { mark = i ;}
        });
        vec.remove(mark);
        <ValidOrderIndexByOrderpair<T>>::insert(sell_order.pair.clone(),vec);
    }
}
