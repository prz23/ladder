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

pub trait Trait: system::Trait + signcheck::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type Symbol = u64;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OrderPair {
    pub share: Symbol,
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
    pub acc: Vec<u8>,        // seller's send account
    pub acc2: Vec<u8>,       // seller's recive account
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
        MatchOrder(u128, Symbol, AccountId,Vec<u8>,Vec<u8>, u64, bool, u64, AccountId,Vec<u8>,Vec<u8>, u64, bool,u64),
        AlertOrder(u128,AccountId,Symbol,u64), // uniqueindex seller newamount cointype
        Settlement(Vec<u8>,Vec<u8>),
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

        PriceExchangeRate get(price_exchange_rate) : u64 = 100000;
     }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        pub fn match_order_verification(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

           //let (tag, bill, from, from_bond,to,to_bond,value,reserved) = Self::parse_matchdata(message.clone(),signature);
            let tx_hash = T::Hashing::hash( &message[..]);
            let signature_hash = T::Hashing::hash( &signature[..]);
            //check the validity and number of signatures
            match   <signcheck::Module<T>>::check_signature(sender.clone(), tx_hash, signature_hash, message.clone()){
                Ok(y) =>  runtime_io::print("ok") ,
                Err(x) => return Err(x),
            }
            //deposit_event
            Self::deposit_event(RawEvent::Settlement(message,signature));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /*
    pub fn parse_matchdata(message: Vec<u8>, signature: Vec<u8>) -> (u64,u64,Vec<u8>,T::AccountId,
                                                                      Vec<u8>,T::AccountId,u64,u64) {
        // message
        let mut messagedrain = message.clone();
        // Tag u64
        let mut tag_vec: Vec<_> = messagedrain.drain(0..32).collect();
        tag_vec.drain(0..24);
        let mut tag_u64 = Self::u8array_to_u64(tag_vec.as_slice());

        // Bill u64
        let mut bill_vec: Vec<_> = messagedrain.drain(0..32).collect();
        bill_vec.drain(0..24);
        let mut bill_u64 = Self::u8array_to_u64(bill_vec.as_slice());

        //from
        let mut from_vec: Vec<_> = messagedrain.drain(0..20).collect();
        let from = from_vec.drain(0..20).collect();

        // from bond
        let mut from_bond_vec: Vec<_> = messagedrain.drain(0..32).collect();
        let from_bond: T::AccountId = Decode::decode(&mut &from_bond_vec[..]).unwrap();

        //to
        let mut to_vec: Vec<_> = messagedrain.drain(0..20).collect();
        let to = to_vec.drain(0..20).collect();

        // to bond
        let mut to_bond_vec: Vec<_> = messagedrain.drain(0..32).collect();
        let to_bond: T::AccountId = Decode::decode(&mut &to_bond_vec[..]).unwrap();

        // value
        let mut value_vec:Vec<u8> = messagedrain.drain(0..32).collect();
        value_vec.drain(0..16);
        let mut value_u64 = Self::u8array_to_u128(value_vec.as_slice()) as u64;

        // reserved  u8
        let mut reserved_vec: Vec<_> = messagedrain.drain(0..32).collect();
        reserved_vec.drain(0..24);
        let mut reserved_u64 = Self::u8array_to_u64(reserved_vec.as_slice());

        return (tag_u64,bill_u64,from,from_bond,to,to_bond,value_u64,reserved_u64);
    }
    */

    pub fn exchange_price() -> f64 {
        Self::price_exchange_rate() as f64
    }

    pub fn init_basic_pair(){
        let mut pair_list: Vec<OrderPair> = <OrderPairList<T>>::get();
        pair_list.push(OrderPair{share:1,money:2});
        pair_list.push(OrderPair{share:2,money:1});
        <OrderPairList<T>>::put(pair_list);
    }

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

    pub fn generate_new_sell_order_and_put_into_list(who:T::AccountId,pair_type:OrderPair,amount:u64,per_price:u64,
                                                     reserved:bool,acc:Vec<u8>,acc2:Vec<u8>){
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
            acc2: acc2,
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
    pub fn buy_operate(buyer:T::AccountId,mut sell_order:&mut OrderT<T>, amount:u64, reserved:bool,acc:Vec<u8>,acc2:Vec<u8>) -> Result {
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

        let token_needed_105 = amount as f64 *sell_order.price as f64;
        let token_needed = token_needed_105/ Self::price_exchange_rate() as f64;

        //deposit_event
        //MatchOrder(id, price, seller, saleAmount, reserved, buyer, purchasingAmount, reserved);
        Self::deposit_event(RawEvent::MatchOrder(sell_order.longindex,sell_order.price,sell_order.who.clone(),
                                                 sell_order.acc.clone(),sell_order.acc2.clone(),amount,sell_order.reserved(),sell_order.pair.share,
                                                 buyer.clone(),acc,acc2,token_needed as u64,reserved,sell_order.pair.money));
        Self::deposit_event(RawEvent::Buy(buyer.clone(),sell_order.who.clone(),sell_order.pair.clone(),
                                          sell_order.index, amount, sell_order.longindex,reserved));
        Ok(())
    }

    /// cancel the sell order
    pub fn cancel_order_operate(who: &T::AccountId,mut sell_order:&mut OrderT<T>) -> Result{
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

    pub fn alert_order_operate(sell_order: OrderT<T>,new_amount: u64) {
        let mut sellorder = sell_order.clone();
        sellorder.amount = new_amount;
        // update the modified sell order
        <SellOrdersOf<T>>::insert((sellorder.who.clone(),sellorder.pair.clone(),sellorder.index.clone()),
                                  sellorder.clone());
        <AllSellOrders<T>>::insert(sellorder.longindex,sellorder.clone());

        // deposit_event
        Self::deposit_event(RawEvent::AlertOrder(sellorder.longindex,sellorder.who,sellorder.amount,sellorder.pair.share));
    }

    pub fn cancel_order_for_bank_withdraw(accountid: T::AccountId,coin_type:u64,acc:Vec<u8>){
        // find the valid sell order for the account
        let all_pair_vec = Self::pair_list();
        all_pair_vec.iter().enumerate().for_each(|(i,pair)|{
            if pair.share == coin_type {
                let new_last_index = Self::last_sell_order_index_of((accountid.clone(), pair.clone())).unwrap_or_default();
                if new_last_index != 0{
                    for i in 0..new_last_index+1{
                        if let Some(mut sellorder) = Self::sell_order_of((accountid.clone(),pair.clone(),i)){
                            if sellorder.acc == acc {
                                if sellorder.status != OtcStatus::Done {
                                    sellorder.status = OtcStatus::Done;
                                    //update the storage
                                    <SellOrdersOf<T>>::insert((accountid.clone(), pair.clone(), i), sellorder.clone());
                                    <AllSellOrders<T>>::insert(sellorder.longindex, sellorder.clone());
                                    Self::delete_from_pair_order_storage(sellorder.clone());
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
