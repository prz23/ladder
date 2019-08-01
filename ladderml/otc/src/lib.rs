#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

//use sr_primitives::traits::{As,CheckedAdd, CheckedSub, Hash, Verify, Zero};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, StorageMap,
    StorageValue,
};

use system::ensure_signed;

use rstd::prelude::*;

#[cfg(feature = "std")]
pub use std::fmt;
#[cfg(feature = "std")]
use runtime_io::with_storage;

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
                // each coin => amount of exchanged
        TradingVolumeTotal get(trading_volume_total): map u64 => u128;

        // (who,coin) => amount of exchanged
        TradingVolumePerson get(trading_volume_person) : map (T::AccountId,u64) => u128;

        // (who,coin) => amount of exchanged
        TradingVolumePersonTemp get(trading_volume_person_temp) : map (T::AccountId,u64) => u128;

        //
        TransactionsQuantityTotal get(transactions_quantity_total) : map u64 => u64;

        //
        TransactionsQuantityPerson get(transactions_quantity_person) : map (T::AccountId,u64) => u64;

        //
        TransactionsQuantityPersonTemp get(transactions_quantity_person_temp) : map (T::AccountId,u64) => u64;
        //
        PeriodicHoldings get(periodic_holdings) : map (T::AccountId,u64) => u128;

        //
        AllCoinType get(all_coin_type) : Vec<u64>;
        // support vec Vec<AccountId> record all buyer seller
        Participant get(participant) : Vec<(T::AccountId,u64)>;

        //default exchangerate to lad  coin => rate*10000
        ExchangeToLad get(exchange_to_lad) : map u64 => u64;
    }

        add_extra_genesis {
        config(athorities) : Vec<T::SessionKey>;
        config(exchangelad) : Vec<(u64,u64)>;
        build(|storage: &mut sr_primitives::StorageOverlay, _: &mut sr_primitives::ChildrenStorageOverlay, config: &GenesisConfig<T>| {
            with_storage(storage, || {
              let exchangelad_vec = config.exchangelad.clone();
                <Module<T>>::inilize_exchange_data(exchangelad_vec);
            })
        })
    }

}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// add an exchange pair
        pub fn new_pair(_origin,pair: OrderPair) -> Result {
            <order::Module<T>>::add_pair(pair)?;
            Ok(())
        }

        ///  put up a sell order to sell the amount of pair.share for per_price of pair.money
        pub fn put_order(origin, pair_type:OrderPair,amount:u64, per_price:u64,acc:Vec<u8>,acc2:Vec<u8>,reserved:bool) -> Result{
            let sender = ensure_signed(origin)?;
            // make sure the sell order is valid
            Self::check_valid_order(sender.clone(),pair_type.clone(),amount,per_price,acc.clone())?;
            // generate a new sell order , lock the money in bank,deposit_event and put up the order
            Self::generate_new_sell_order_and_put_into_list(sender.clone(),pair_type.clone(),amount,per_price,reserved,acc,acc2);

            Ok(())
        }

        /// chose an sell order to buy the amount of pair.share
        pub fn buy(origin, seller:T::AccountId, pair:OrderPair ,index:u64, amount:u64,
                   acc:Vec<u8>,acc2:Vec<u8>,reserved:bool) -> Result{
            let buyer = ensure_signed(origin)?;

            // find the sell order
            if let Some(sellorder) = <order::Module<T>>::sell_order_of((seller,pair,index)){
                // make sure the buy operate is valid
                Self::check_valid_buy(buyer.clone(),amount,sellorder.clone(),acc.clone())?;
                // do the buy operate and modify the order's status
                Self::buy_operate(buyer.clone(),sellorder.clone(),amount,reserved,acc,acc2)?;
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }

        pub fn cancel_order(origin,pair_type:OrderPair,index:u64) -> Result {
            let sender = ensure_signed(origin)?;

            // find  the order
            if let Some(sellorder) = <order::Module<T>>::sell_order_of((sender.clone(),pair_type,index)){
                // cancel the sell order and unlock the share not deal yet
                Self::cancel_order_operate(&sender,sellorder)?;
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }

        pub fn cancel_order_with_uniqueindex(origin, index:u128) -> Result {
            let sender = ensure_signed(origin)?;

            // find  the order
            if let Some(sellorder) = <order::Module<T>>::all_sell_orders(index){
                // cancel the sell order and unlock the share not deal yet
                Self::cancel_order_operate(&sender,sellorder)?;
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }

        pub fn alert_order(origin,index:u128,amount:u64) -> Result {
            //
            let sender = ensure_signed(origin)?;
            // find  the order
            if let Some(sellorder) = <order::Module<T>>::all_sell_orders(index){
                // cancel the sell order and unlock the share not deal yet
                Self::alert_order_operate(sender.clone(),sellorder,amount)?;
            }else{
                return Err("invalid sell order");
            }
            Ok(())

        }

    }
}

impl<T: Trait> Module<T> {

    /// Query the Bank Module，make sure the seller have enough money to sell
    /// and check the parmeters is not zero
    fn check_valid_order(who: T::AccountId, pair:OrderPair, amount:u64, price:u64,acc:Vec<u8>) -> Result {

        <order::Module<T>>::check_valid_order(who.clone(),&pair,amount,price)?;

        //bank---->pair.share  money enough?
        let free_token = <bank::Module<T>>::deposit_free_token((who.clone(),acc.clone(),pair.share));
        if free_token < amount {
            return Err("not_enough_money_error check_valid_order");
        }
        Ok(())
    }

    fn check_enough_token(who:&T::AccountId,acc:Vec<u8>,cointype:u64,amount:u64) -> Result{
        let free_token = <bank::Module<T>>::deposit_free_token((who.clone(),acc.clone(),cointype));
        if free_token < amount {
            return Err("not_enough_money_error check_enough_token");
        }
        Ok(())
    }

    fn check_valid_buy(who:T::AccountId,amount:u64,sellorder:OrderT<T>,acc:Vec<u8>) -> Result {

        let mut sell_order = sellorder.clone();
        <order::Module<T>>::check_valid_buy(who.clone(),amount,&mut sell_order)?;

        let free_token = <bank::Module<T>>::deposit_free_token((who.clone(),acc.clone(),sellorder.pair.money));
        let token_needed = Self::price_restoration(amount,sell_order.price.clone());

        if (free_token as f64) < token_needed  {
            return Err("not_enough_money_error check_valid_buy");
        }

        Ok(())
    }

    fn generate_new_sell_order_and_put_into_list(who:T::AccountId,pair_type:OrderPair,amount:u64,per_price:u64,reserved:bool,acc:Vec<u8>,acc2:Vec<u8>){
        // put sell order in order Module
        <order::Module<T>>::generate_new_sell_order_and_put_into_list(who.clone(),pair_type.clone(),amount,per_price,
                                                                      reserved,acc.clone(),acc2.clone());
        // lock the specific kingd of money of amount in bank Module
        <bank::Module<T>>::lock_token(&who,acc.clone(),pair_type.share,amount,bank::TokenType::OTC).ok();
    }

    // buy operate , lock the money and change the status
    fn buy_operate(buyer:T::AccountId,sellorder: OrderT<T>, amount:u64, reserved:bool,acc:Vec<u8>,acc2:Vec<u8>) -> Result {

        let mut sell_order = sellorder.clone();
        // Judge and process this buy operation and update the sell order
        <order::Module<T>>::buy_operate(buyer.clone(),&mut sell_order,amount,reserved,acc.clone(),acc2.clone())?;

        // exchange/unlock/lock the token in bank for buyer & seller
        <bank::Module<T>>::buy_operate(buyer.clone(),sell_order.who.clone(),sell_order.pair.share.clone(),
                                       sell_order.pair.money.clone(),sell_order.price.clone(),
                                       amount,sell_order.reserved(),reserved,
                                       sell_order.acc,sell_order.acc2,acc,acc2)?;

        // every settlement , record the trade volum and number
        Self::volum_and_number_rocord(&sell_order.who,sell_order.pair.share.clone(),&buyer,sell_order.pair.money.clone(),
                                      amount,Self::price_restoration(amount,sell_order.price.clone())as u64);
        Self::record_the_last_exchange_data(sell_order.pair.share.clone(),sell_order.pair.money.clone(),amount,sell_order.price.clone());
        Ok(())
    }

    /// price_restoration / 10000
    pub fn price_restoration(amount:u64,price:u64) -> f64{
        let token_needed_105 = amount as f64 * price as f64;
        let token_needed = token_needed_105/ <order::Module<T>>::price_exchange_rate() as f64;
        token_needed
    }

    /// cancel the sell order
    pub fn cancel_order_operate(who:&T::AccountId,mut sell_order:OrderT<T>) -> Result{

        match <order::Module<T>>::cancel_order_operate(&who,&mut sell_order){
            Err("Has been cancelled")  => return Ok(()),
            Ok(()) => {
                // unlock the left share of seller
                let left_shares = sell_order.amount - sell_order.already_deal;
                <bank::Module<T>>::unlock_token(&who,sell_order.acc,sell_order.pair.share,
                                                left_shares,bank::TokenType::OTC)?;
            },
            _ => return Err("unknown err"),
        }
        Ok(())
    }

    // alert sell order
    pub fn alert_order_operate(seller:T::AccountId , sell_order: OrderT<T> ,amount: u64) -> Result {
        if sell_order.status == order::OtcStatus::Done { return Err("Cant modify completed order");}
        if amount == sell_order.amount { return Err("Same amount in the sell order");}
        if amount > sell_order.amount {
            //
            let increment_amount = amount - sell_order.amount;
            Self::check_enough_token(&seller,sell_order.acc.clone(),sell_order.pair.share,increment_amount)?;
            <bank::Module<T>>::lock_token(&seller,sell_order.acc.clone(),sell_order.pair.share,increment_amount,bank::TokenType::OTC)?;
        }else {
            if amount <= sell_order.already_deal {return Err("Cant smaller than already dealed amount.");}
            //
            let decrease_amount = sell_order.amount - amount;
            <bank::Module<T>>::unlock_token(&seller,sell_order.acc.clone(),sell_order.pair.share,decrease_amount,bank::TokenType::OTC)?;
        }
        <order::Module<T>>::alert_order_operate(sell_order,amount);
        Ok(())
    }

    /*-----------------record------------------*/
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
        <TransactionsQuantityTotal<T>>::insert(coin_type,new_volume as u64);
    }

    pub fn number_of_trading_person(who:&T::AccountId,coin_type:u64){
        let new_times = 1 + Self::transactions_quantity_person((who.clone(),coin_type));
        <TransactionsQuantityPerson<T>>::insert((who.clone(),coin_type),new_times);
    }

    /// in OTC module , when order settle , record data , in bank each period of session clean the data
    pub fn volum_and_number_rocord(seller:&T::AccountId, share:u64, buyer:&T::AccountId, money:u64, amount_s:u64, amount_b:u64){

        if !Self::participant().contains(&(seller.clone(),share)){
            // not exist , init it to zero
            <TradingVolumePerson<T>>::insert((seller.clone(),share),0u128);
            <TransactionsQuantityPerson<T>>::insert((seller.clone(),share),0u64);
        }
        if !Self::participant().contains(&(buyer.clone(),money)){
            // not exist , init it to zero
            <TradingVolumePerson<T>>::insert((buyer.clone(),money),0u128);
            <TransactionsQuantityPerson<T>>::insert((buyer.clone(),money),0u64);
        }

        Self::increase_trading_volume_total(share,amount_s);
        Self::increase_trading_volume_total(money,amount_b);

        Self::increase_trading_volume_person(seller,share,amount_s);
        Self::increase_trading_volume_person(buyer,money,amount_b);

        Self::number_of_trading_total(share);
        Self::number_of_trading_total(money);

        Self::number_of_trading_person(seller,share);
        Self::number_of_trading_person(buyer,money);

        let mut buyersellervec = Self::participant();
        if buyersellervec.contains(&(seller.clone(),money)) { }else { buyersellervec.push((seller.clone(),share)); <Participant<T>>::put(buyersellervec); }
        let mut buyersellervec = Self::participant();
        if buyersellervec.contains(&(buyer.clone(),money)) { }else { buyersellervec.push((buyer.clone(),money)); <Participant<T>>::put(buyersellervec); }

        let mut coin_type = Self::all_coin_type();
        if !Self::all_coin_type().contains(&share){
            coin_type.push(share);
        }
        if !Self::all_coin_type().contains(&money){
            coin_type.push(money);
        }
        <AllCoinType<T>>::put(coin_type);
    }

    pub fn periodical_clean() {
        // put a new empty vector into the Participant storage
        <Participant<T>>::put([].to_vec() as Vec<(T::AccountId,u64)>);
        // clear the total data using cointype
        Self::all_coin_type().iter().enumerate().for_each(|(_i,coin)|{
            <TransactionsQuantityTotal<T>>::insert(coin,0u64);
            <TradingVolumeTotal<T>>::insert(coin,0u128);
        });
        //clear cointype
        <AllCoinType<T>>::put([].to_vec() as Vec<u64>);
    }

    pub fn inilize_exchange_data(data:Vec<(u64,u64)>){
        //ExchangeToLad get(exchange_to_lad) : map u64 => u64;
        data.iter().enumerate().for_each(|(_i,(coin,exchange_rate))|{
            <ExchangeToLad<T>>::insert(coin,exchange_rate);
        });
    }

    pub fn record_the_last_exchange_data(share:u64, money:u64, amount:u64, price:u64){
        //TODO:: from every otc transcation, calculate the exchange rate of the order pair to LAD
        //get the real amount
        let share_amount = amount;
        let money_amount = Self::price_restoration(amount,price)as u64;
        //calculate the new exchange rate to lad
        let old_share_to_lad = <ExchangeToLad<T>>::get(share);
        let new_money_to_lad =  (old_share_to_lad as f64 * money_amount as f64) / share_amount as f64 ;
        //update the data
        <ExchangeToLad<T>>::insert(share,old_share_to_lad);
        <ExchangeToLad<T>>::insert(money,new_money_to_lad as u64);
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

            let seller_acc : Vec<u8> = [2,3,4,95].to_vec();
            let seller_acc2 : Vec<u8> = [3,4,5,6,1].to_vec();

            let buyer_acc : Vec<u8> = [10,15,68].to_vec();
            let buyer_acc2 : Vec<u8> = [55,41,12].to_vec();

            let pair:OrderPair = OrderPair{ share:1 ,money:2};
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

        });
    }

    #[test]
    fn cancel_order_test() {
        with_externalities(&mut new_test_ext(), || {
            let seller_acc : Vec<u8> = [2,3,4,5].to_vec();
            let seller_acc2 : Vec<u8> = [3,4,5,6].to_vec();

            let buyer_acc : Vec<u8> = [5,6,7,8].to_vec();
            let buyer_acc2 : Vec<u8> = [7,8,9,10].to_vec();

            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_err!(Order::is_valid_pair(&pair) , "an invalid orderpair");
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(Order::is_valid_pair(&pair));

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 0 , 10 ,seller_acc.clone(),seller_acc2.clone(),true) ,"price cann't be 0");
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 0 ,seller_acc.clone(),seller_acc2.clone(),true) ,"price cann't be 0");

            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10 , 10 ,seller_acc.clone(),seller_acc2.clone(),true) ,"not_enough_money_error check_valid_order");
            assert_err!(OTC::put_order(Some(2).into() , pair.clone(), 10 , 10 ,seller_acc.clone(),seller_acc2.clone(),true) ,"not_enough_money_error check_valid_order");

            Bank::modify_token(1,seller_acc.clone(),1,50,bank::TokenType::Free,true);
            Bank::modify_token(1,seller_acc.clone(),2,50,bank::TokenType::Free,true);
            Bank::modify_token(1,seller_acc.clone(),3,50,bank::TokenType::Free,true);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),1)),50);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),2)),50);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),3)),50);

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ,seller_acc.clone(),seller_acc2.clone(),true));
            println!("  put order ");
            // a new order's status is new
            let aa = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(aa.status,order::OtcStatus::New);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),1)),40);
            assert_eq!(Bank::deposit_otc_token((1,seller_acc.clone(),1)),10);

            // an order was put up , cancel it, and see the status changes into done
            assert_ok!(OTC::cancel_order(Some(1).into() , pair.clone(),1));
            let bb = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(bb.status,order::OtcStatus::Done);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),1)),50);
            assert_eq!(Bank::deposit_otc_token((1,seller_acc.clone(),1)),0);
        });
    }

    #[test]
    fn cancel_all_order_test() {
        with_externalities(&mut new_test_ext(), || {
            let seller_acc : Vec<u8> = [2,3,4,5].to_vec();
            let seller_acc2 : Vec<u8> = [3,4,5,6].to_vec();

            let buyer_acc : Vec<u8> = [5,6,7,8].to_vec();
            let buyer_acc2 : Vec<u8> = [7,8,9,10].to_vec();
            let pair:OrderPair = OrderPair{ share:3 ,money:2};
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));

            let pair2:OrderPair = OrderPair{ share:3 ,money:1};
            assert_ok!(OTC::new_pair(Some(1).into() , pair2.clone()));


            Bank::modify_token(1,seller_acc.clone(),1,50,bank::TokenType::Free,true);
            Bank::modify_token(1,seller_acc.clone(),2,50,bank::TokenType::Free,true);
            Bank::modify_token(1,seller_acc.clone(),3,50,bank::TokenType::Free,true);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),1)),50);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),2)),50);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),3)),50);

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 15, 10 ,seller_acc.clone(),seller_acc2.clone(),true));
            // a new order's status is new
            let aa = Order::sell_order_of( (1, pair.clone(), 1) ).unwrap();
            assert_eq!(aa.status,order::OtcStatus::New);
            assert_eq!(Bank::deposit_free_token((1,seller_acc.clone(),3)),35);
            assert_eq!(Bank::deposit_otc_token((1,seller_acc.clone(),3)),15);

            assert_ok!(OTC::put_order(Some(1).into() , pair2.clone(), 15, 10 ,seller_acc.clone(),seller_acc2.clone(),true));
            // an account 1 has type3 token 40 free and 15 locked by put up sell order
            // now cancel all, the status turn into done
            Order::cancel_order_for_bank_withdraw(1,pair.share,seller_acc.clone());

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
            let seller_acc : Vec<u8> = [247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec();
            let seller_acc2 : Vec<u8> = [3,4,5,6].to_vec();

            let buyer_acc : Vec<u8> = [5,6,7,8].to_vec();
            let buyer_acc2 : Vec<u8> = [7,8,9,10].to_vec();


            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::deposit_free_token((11744161374129632607,seller_acc.clone(),1)),100000000000);


            //assert_eq!(Bank::coin_deposit(0),<tests::Test as Trait>::Balance::sa(0));
            let mut data2 : Vec<u8>= "00000000000000010000000000000000000000000000000000000000000000000000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea4".from_hex().unwrap();
            let sign2 : Vec<u8>= "b36bba3f9e7138e45b9ff9918a0759623ca146b3956174efaadb37635c2adb440f9fd75e7773803337d4802d94f5c78788121dccd4b698080b047171966483711b".from_hex().unwrap();
            assert_ok!(Bank::request(Origin::signed(5),data2,sign2));
            assert_eq!(Bank::deposit_free_token((11744161374129632607,seller_acc.clone(),1)),0);

        });
    }

    #[test]
    fn withdraw_request_withdraw_sell_order_test() {
        with_externalities(&mut new_test_ext(), || {
            let seller_acc : Vec<u8> = [247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec();
            let seller_acc2 : Vec<u8> = [3,4,5,6].to_vec();

            let buyer_acc : Vec<u8> = [5,6,7,8].to_vec();
            let buyer_acc2 : Vec<u8> = [7,8,9,10].to_vec();

            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::deposit_free_token((11744161374129632607,seller_acc.clone(),1)),100000000000);

            // put order
            let pair:OrderPair = OrderPair{ share:1 ,money:2};
            assert_ok!(OTC::new_pair(Some(1).into() , pair.clone()));
            assert_ok!(OTC::put_order(Some(11744161374129632607).into() , pair.clone(), 10, 10 ,seller_acc.clone(),seller_acc2.clone(),true));
            assert_eq!(Bank::deposit_free_token((11744161374129632607,seller_acc.clone(),1)),99999999990);

            //assert_eq!(Bank::coin_deposit(0),<tests::Test as Trait>::Balance::sa(0));
            let mut data2 : Vec<u8>= "00000000000000010000000000000000000000000000000000000000000000000000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea4".from_hex().unwrap();
            let sign2 : Vec<u8>= "b36bba3f9e7138e45b9ff9918a0759623ca146b3956174efaadb37635c2adb440f9fd75e7773803337d4802d94f5c78788121dccd4b698080b047171966483711b".from_hex().unwrap();
            assert_ok!(Bank::request(Origin::signed(5),data2,sign2));

            assert_eq!(Bank::deposit_free_token((11744161374129632607,seller_acc.clone(),1)),0);
            assert_eq!(Bank::deposit_withdraw_token((11744161374129632607,seller_acc.clone(),1)),100000000000);
        });
    }
}