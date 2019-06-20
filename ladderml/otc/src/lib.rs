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
    pub pair: pair,     // 交易对
    pub index: u64,          // 编号
    pub who: AccountID,      // 该交易挂单人 seller
    pub amount: symbol,      // pair.share的挂单数量
    pub price: symbol,       // pair.money的单价
    pub already_deal:symbol, // 已经交易掉的数量
    pub status: status,         //TODO 交易当前状态
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
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Signature {

        /// 全部挂出来的卖单信息
        pub OrdersOf get(order_of):map (T::AccountId, OrderPair, u64) => Option<OrderT<T>>;

        /// 链上已经存在的允许的 配对 表
        pub OrderPairList get(pair_list):  Vec<OrderPair>;

        /// 某个用户的某种配对的index     在挂出卖单时，分配一个
        pub LastOrderIndexOf get(last_order_index_of): map(T::AccountId,OrderPair)=>Option<u64>;
     }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// 增加一个交易对
        pub fn new_pair(origin,pair: OrderPair) -> Result {
            Self::add_pair(pair)?;
            Ok(())
        }

        /// 一个人挂出一个单子，内容是 卖btc 换eth   btc的数量是5  每btc个卖100eth
        pub fn put_order(origin, pair_type:OrderPair,amount:u64, per_price:u64) -> Result{
            let sender = ensure_signed(origin)?;

            // 判断该这个人在bank上面是否由对应的数量的btc余额,
            Self::check_valid_order(sender.clone(),pair_type.clone(),amount,per_price)?;

            //TODO::锁定bank里面对应的挂出来的pair.share的数量的资产（OK了）

            //生成新卖单，并加入挂单的列表 ，同时锁定对应资产
            Self::generate_new_sell_order_and_put_into_list(sender.clone(),pair_type.clone(),amount,per_price);

            //TODO::deposit_event
            Ok(())
        }

             /// 选择某个交易对的某个挂单进行买入操作， 买入量必须少于挂单上的量，
        pub fn buy(origin, seller:T::AccountId, pair:OrderPair ,index:u64, amount:u64) -> Result{
            let buyer = ensure_signed(origin)?;

            // 查找这个挂单，
            if let Some(mut sellorder) = Self::order_of((seller,pair,index)){

                // 检测买入操作 与 挂单是否合法
                Self::check_valid_buy(buyer.clone(),amount,sellorder.clone())?;

                // 进行买入操作，修改状态
                Self::buy_operate(buyer.clone(),sellorder.clone(),amount);
            }else{
                return Err("invalid sell order");
            }
            Ok(())
        }


        pub fn withdraw_order(origin,pair_type:OrderPair,index:u64) -> Result {
            let sender = ensure_signed(origin)?;

            // 先把这个单子找到，看看是否是已经完成的单子
           // Self::
            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    // 增加交易对
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

    //判定是否存在
    fn is_valid_pair(pair: &OrderPair) -> Result {
        let pair_list: Vec<OrderPair> = <OrderPairList<T>>::get();
        if pair_list.contains(pair) {
            Ok(())
        } else {
            Err("an invalid orderpair")
        }
    }

    /// 去Bank模块查询一下子，看看pair里面btc是否有足够的数量安排在那里。
    /// 顺便检测交易对等信息是否ok
    fn check_valid_order(who: T::AccountId, pair:OrderPair, amount:u64, price:u64) -> Result {
        // 交易对正确吗？
        Self::is_valid_pair(&pair)?;

        // 每个btc卖出定价不能为零
        Self::is_price_zero(amount)?;
        Self::is_price_zero(price)?;

        //bank---->pair.share 看看数量足够吗
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
        // 每个btc卖出定价不能为零
        Self::is_price_zero(amount)?;

        // 不能买的量 超过 卖单余量
        let left_share = sell_order.amount - sell_order.already_deal;
        if amount > left_share {
            return Err("cant buy too much!");
        }
        //去查看下bank里有没有钱
        let deposit_data = <bank::Module<T>>::despositing_banance(&who);
        if  deposit_data == [].to_vec() { return Err("no data"); }
        let mut  not_enough_money_error = false;
        deposit_data.iter().enumerate().for_each(|(i,&(balance,coin))|{
            if coin == sell_order.pair.share {
                if balance < T::Balance::sa(amount*sell_order.price) {
                    not_enough_money_error = true;
                }
            }
        });
        if not_enough_money_error == true { return Err("not_enough_money_error "); }
        Ok(())
    }

    fn generate_new_sell_order_and_put_into_list(who:T::AccountId, pair_type:OrderPair,amount:u64, per_price:u64 ){
        // 更新用户的交易对的挂单index
        let new_last_index = Self::last_order_index_of((who.clone(), pair_type.clone())).unwrap_or_default() + 1;
        <LastOrderIndexOf<T>>::insert((who.clone(), pair_type.clone()), new_last_index);
        // 生成挂单并加入挂单列表中
        let mut new_sell_order :OrderT<T> = OrderContent{
            pair:pair_type.clone(),
            index:new_last_index,
            who:who.clone(),
            amount: amount,      // pair.share的挂单数量
            price: per_price,    // pair.money的单价
            already_deal:0,      // 已经交易掉的数量
            status: Status::New, //交易当前状态
        };
        <OrdersOf<T>>::insert((who.clone(), pair_type.clone(), new_last_index), new_sell_order.clone());
        // 锁定挂出的单子的币的数量
        <bank::Module<T>>::lock(who.clone(),pair_type.share,amount);
        //Self::save_new_order_by_pair(pair_type,new_sell_order.clone());
    }

    // buy 的操作 修改各个挂单存储结构，修改bank对应数据，锁定相关资产。
    fn buy_operate(buyer:T::AccountId,mut sell_order: OrderT<T>, amount:u64) -> Result {
        // 修改已经交易额
        sell_order.already_deal = sell_order.already_deal+amount;
        // 修改挂单状态
        if sell_order.already_deal < sell_order.amount {
            sell_order.status = Status::Half;
        }else if sell_order.amount == sell_order.already_deal {
            sell_order.status = Status::Done;
        }else { return  Err("wrong!"); }

        // 把修改挂单数据保存
        <OrdersOf<T>>::insert((sell_order.who.clone(),sell_order.pair.clone(),sell_order.index.clone()),
                              sell_order.clone());
        //TODO::bank 修改双方pair.money的值
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
            let aa = OTC::order_of( (1, pair.clone(), 1) ).unwrap();

            // 再安排个铁子，让他买一下
            Bank::depositing_withdraw_record(2,50,1,true);
            Bank::depositing_withdraw_record(2,50,2,true);
            assert_eq!(Bank::despositing_banance(2),[(0, 0), (50, 1), (50, 2), (0, 3), (0, 4)].to_vec());
            assert_ok!(OTC::buy(Some(2).into(),1, pair.clone(),1,5));
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
            let aa = OTC::order_of( (1, pair.clone(), 1) ).unwrap();

            // 再安排个铁子，让他买一下
            Bank::depositing_withdraw_record(2,50,1,true);
            Bank::depositing_withdraw_record(2,50,2,true);
            assert_eq!(Bank::despositing_banance(2),[(0, 0), (50, 1), (50, 2), (0, 3), (0, 4)].to_vec());

            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ));
            let aa = OTC::order_of( (1, pair.clone(), 2) ).unwrap();
            assert_ok!(OTC::put_order(Some(1).into() , pair.clone(), 30, 10 ));
            // 挂单过多，本身只有50块钱，挂出去50，再挂10 就会报错，钱不够
            assert_err!(OTC::put_order(Some(1).into() , pair.clone(), 10, 10 ),"not_enough_money_error ");
        });
    }

}