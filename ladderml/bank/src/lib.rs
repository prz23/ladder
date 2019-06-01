#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

use sr_primitives::traits::{As, CheckedAdd, CheckedSub, Hash, One, Verify, Zero};
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

use support::traits::Currency;

use signcheck;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: balances::Trait + session::Trait + signcheck::Trait{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// deposit
        //(origin, message: Vec, signature: Vec)
        pub fn deposit(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            runtime_io::print("====================deposit===============");
/*
            //TODO: 在这里判断 sender 是否有权限提交 后期启动节点时写入
            let validators = <session::Module<T>>::validators();
            ensure!(validators.contains(&sender),"Not validator");
*/
            // resolving message --> Ethereum's transcation hash  tx_hash   accountid
            //                       depositing amount    signature_hash
            let (tx_hash, who, amount, signature_hash,coin_type) = Self::split_message(message.clone(),signature);

            // ensure no repeat desposit
            ensure!(Self::despositing_account().iter().find(|&t| t == &who).is_none(), "Cannot deposit if already depositing.");
            // ensure no repeat intentions to desposit
            ensure!(Self::intentions_desposit_vec().iter().find(|&t| t == &who).is_none(), "Cannot deposit if already in queue.");

            //check the validity and number of signatures
            match  Self::check_signature(sender.clone(), tx_hash, signature_hash, tx_hash){
                Ok(y) =>  runtime_io::print("ok") ,
                Err(x) => return Err(x),
            }

            // update the list of intentions to desposit
            <IntentionsDespositVec<T>>::put({
                let mut v =  Self::intentions_desposit_vec();
                v.push(who.clone());
                v
            });
            <IntentionsDesposit<T>>::insert(who.clone(),(T::Balance::sa(amount as u64) , coin_type));
            // emit an event
            Self::deposit_event(RawEvent::AddDepositingQueue(who));
            Ok(())
        }

        /// Take the initiative to receive awards
        pub fn draw_reward_all(origin) -> Result {
             let sender = ensure_signed(origin)?;

             ensure!(!Self::despositing_account().iter().find(|&t| t == &sender).is_none(), "Cannot draw if not depositing.");

             let reward = Self::count_draw_reward(sender.clone());
             let new_balance = <BalanceOf<T> as As<u64>>::sa(T::Balance::as_(reward));
             T::Currency::deposit_into_existing(&sender, new_balance).ok();
             Ok(())
        }

        /// withdraw
        /// origin, message: Vec<u8>, signature: Vec<u8>
         pub fn withdraw(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            let validators = <session::Module<T>>::validators();
            ensure!(validators.contains(&sender),"Not validator");
            // resovling message --> hash  tag  id  amount
            let (_tx_hash,who,amount,signature_hash,coin_type) = Self::split_message(message.clone(),signature);
            let message_hash = Decode::decode(&mut &message.encode()[..]).unwrap();

            //check the validity and number of signatures
            match  Self::check_signature(sender.clone(), message_hash, signature_hash, message_hash){
                Ok(y) =>  runtime_io::print("ok") ,
                Err(x) => return Err(x),
            }
            // ensure no repeat
            ensure!(!Self::despositing_account().iter().find(|&t| t == &who).is_none(), "Cannot deposit if not depositing.");
            ensure!(Self::intentions_withdraw_vec().iter().find(|&t| t == &who).is_none(), "Cannot withdraw2 if already in withdraw2 queue.");

            <IntentionsWithdrawVec<T>>::put({
                let mut v =  Self::intentions_withdraw_vec();
                v.push(who.clone());
                v
            });
            <IntentionsWithdraw<T>>::insert(who.clone(),(T::Balance::sa(amount as u64) , coin_type));

            // emit an event
            Self::deposit_event(RawEvent::AddWithdrawQueue(who));
            Ok(())
        }

        /// set reward factor
        ///   0-5000 5000-50000 50000-500000 500000-->
        ///      x1         x2         x3      x3
        ///     y1      y2         y3          y4
        fn set_session_reward_factor(_origin,session: Vec<u32>, session_factor: Vec<u8>)-> Result{
            // session
            ensure!(session.len() >= 3,"not enough session arguments.at least 3");
            ensure!(session_factor.len() >= 4,"not enough session_factor arguments.at least 4");

            <RewardSessionValue<T>>::put(session);
            <RewardSessionFactor<T>>::put(session_factor);
            Ok(())
            // money
        }

        /// set reward factor
        ///   0-5000 5000-50000 50000-500000 500000-->
        ///      x1         x2         x3      x3
        ///     y1      y2         y3          y4
        fn set_balance_reward_factor(_origin, money: Vec<T::Balance>, money_factor: Vec<u8>)-> Result{

            ensure!(money.len() >= 3,"not enough money arguments.at least 3");
            ensure!(money_factor.len() >= 4,"not enough money_factor arguments.at least 4");

            <RewardBalanceValue<T>>::put(money);
            <RewardBalanceFactor<T>>::put(money_factor);

            Ok(())
            // money
        }
        /// set session lenth
        fn set_session_lenth(session_len: u64 ){
           ensure!(session_len >= 10,"the session lenth must larger than 10");
           <SessionLength<T>>::put(T::BlockNumber::sa(session_len));
        }

        pub fn draw_reward(origin,id: T::AccountId) -> Result{
             ensure!(!Self::despositing_account().iter().find(|&t| t == &id).is_none(), "Cannot draw if not depositing.");

             let new_balance = <BalanceOf<T> as As<u64>>::sa(T::Balance::as_(Self::count_draw_reward(id.clone())));
             match T::Currency::deposit_into_existing(&id, new_balance){
                 Err(x) => Err(x),
                 _ => Ok(()),
             }
        }

        /// a new session starts
		fn on_finalize(n: T::BlockNumber) {
		    Self::check_rotate_session(n);
		}
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Bank {
        /// bank & session

        /// record depositing info of balance & session_time
        DespoitingAccount get(despositing_account): Vec<T::AccountId>;
        /// if the map is 0 ,delete the above Vec
        DepositngAccountTotal get(depositing_account_total) : map T::AccountId => u64;

        /// accountid => (balance , type)
        DespositingBalance get(despositing_banance): map T::AccountId => Vec<(T::Balance,u64)>;
        DespositingTime get(despositing_time): map T::AccountId => u32;

        /// All the accounts with a desire to deposit.  to control one time one deposit.
        IntentionsDespositVec  get(intentions_desposit_vec) :  Vec<T::AccountId>;
        /// accountid => balance cointype
        IntentionsDesposit  get(intentions_desposit): map T::AccountId => (T::Balance , u64);


        /// All the accounts with a desire to withdraw.  to control one time one deposit.
        IntentionsWithdrawVec  get(intentions_withdraw_vec): Vec<T::AccountId>;
         /// accountid => balance cointype
        IntentionsWithdraw get(intentions_withdraw): map T::AccountId => (T::Balance , u64);


        /// Bank session reward factor
        RewardSessionValue  get(reward_session_value) config(): Vec<u32>;
        RewardSessionFactor  get(reward_session_factor) config(): Vec<u8>;
        /// Bank balance reward factor
        RewardBalanceValue  get(reward_balance_value) config(): Vec<T::Balance>;
        RewardBalanceFactor  get(reward_balance_factor) config(): Vec<u8>;

        //RewardFactorS get(reward_factor) config():map u64 => RewardFactor<u64>;

        ///Session module
		/// Block at which the session length last changed.
		LastLengthChange: Option<T::BlockNumber>;
		/// Current length of the session.
		pub SessionLength get(length) config(session_length): T::BlockNumber = T::BlockNumber::sa(10);

		/// The next session length.
		NextSessionLength: Option<T::BlockNumber>;
		/// Timestamp when current session started.
		pub CurrentStart get(current_start) build(|_| T::Moment::zero()): T::Moment;
		/// Current index of the session.
		pub CurrentIndex get(current_index) build(|_| T::BlockNumber::sa(0)): T::BlockNumber;

		/// 新功能 => 模拟chainX 把奖励记录下来，点击领取才发钱 的存储
		RewardRecord get(reward_record):  map T::AccountId => T::Balance;
		/// true -- 领取模式  false -- 自动发放模式
		EnableRewardRecord get(enable_record) config(): bool;
        ///
        TotalDespositingBalacne  get(total_despositing_balance) config(): T::Balance;
        /// MAP of cointype => (TotalDeposit , AllReward)
        CoinDeposit get(coin_deposit) : map u64 => T::Balance;
        CoinReward get(coin_reward) : map u64 => T::Balance;

        /// Investment proportion. Controlling the Ratio of External Assets to Local Assets
        DespositExchangeRate get(desposit_exchange_rate) :  u64 = 10000000000000;

    }
}


decl_event! {
    pub enum Event<T> where
        <T as balances::Trait>::Balance,
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as system::Trait>::BlockNumber
    {
        Has(Hash),
        ///bank moduel
		/// All validators have been rewarded by the given balance.
		Reward(Balance),
		/// accountid added to the intentions to deposit queue
		AddDepositingQueue(AccountId),
		/// intentions to withdraw
		AddWithdrawQueue(AccountId),
        /// a new seesion start
        NewRewardSession(BlockNumber),

    }
}

impl<T: Trait> Module<T>
{
    fn  split_message( message: Vec<u8>, signature: Vec<u8>) -> (T::Hash,T::AccountId,u64,T::Hash,u64) {

        // 解析message --> hash  tag  id  amount
        let mut messagedrain = message.clone();

        // Coin 0-32
        let mut coin_vec: Vec<_> = messagedrain.drain(0..32).collect();
        coin_vec.reverse();
        let mut coin_type = Self::u8array_to_u64(coin_vec.as_slice());

        // Who 33-64
        let who_vec: Vec<_> = messagedrain.drain(0..32).collect();
        let who: T::AccountId = Decode::decode(&mut &who_vec[..]).unwrap();

        //65-96
        let mut amount_vec:Vec<u8> = messagedrain.drain(0..32).collect();
        amount_vec.drain(0..24);
        let mut amountu64 = Self::u8array_to_u64(amount_vec.as_slice());
        amountu64 = ((amountu64 as f64)/<DespositExchangeRate<T>>::get() as f64) as u64;

        // Tx_Hash 97-128
        let hash:Vec<u8> = messagedrain.drain(0..32).collect();
        let tx_hash = Decode::decode(&mut &hash[..]).unwrap();

        // Signature_Hash
        let signature_hash =  Decode::decode(&mut &signature[..]).unwrap();

        return (tx_hash,who,amountu64,signature_hash,coin_type);
    }

    pub fn signature521(signature: Vec<u8>, hash: Vec<u8>){
        //ensure the signature is valid
        let mut tx_hash_to_check:[u8;65] = [0;65];
        tx_hash_to_check.clone_from_slice(&hash);
        let mut signature_hash_to_check:[u8;32] = [0;32];
        signature_hash_to_check.clone_from_slice(&signature);
        Self::check_secp512(&tx_hash_to_check,&signature_hash_to_check).is_ok();
    }

    /// Hook to be called after transaction processing.
    pub fn check_rotate_session(block_number: T::BlockNumber) {
        // do this last, after the staking system has had chance to switch out the authorities for the
        // new set.
        // check block number and call next_session if necessary.
        let is_final_block = ((block_number - Self::last_length_change()) % Self::length()).is_zero();
        let (should_end_session, apply_rewards) = None
            .map_or((is_final_block, is_final_block), |apply_rewards| (true, apply_rewards));
        if should_end_session {
            runtime_io::print("Start new session of bank");
            Self::rotate_session(is_final_block, apply_rewards);
        }
    }

    /// The last length change, if there was one, zero if not.  查看lenth间隔长度
    pub fn last_length_change() -> T::BlockNumber {
        <LastLengthChange<T>>::get().unwrap_or_else(T::BlockNumber::zero)
    }

    /// Move onto next session: register the new authority set.
    /// 把新的 depositingqueue 加入 实际depositing列表   或者 把不存钱的账户从列表里删除
    /// 并且根据其存的金额 之后每个session都对列表里的人存的钱发一定比例到他的balance里
    pub fn rotate_session(is_final_block: bool, _apply_rewards: bool) {
        runtime_io::print("开始调整depositing列表，同时开始给他们发钱");
        let now = <timestamp::Module<T>>::get();
        let _time_elapsed = now.clone() - Self::current_start();
        let session_index = <CurrentIndex<T>>::get() + One::one();
        Self::deposit_event(RawEvent::NewRewardSession(session_index));

        // Increment current session index.
        <CurrentIndex<T>>::put(session_index);
        <CurrentStart<T>>::put(now);
        // Enact session length change.
        let len_changed = if let Some(next_len) = <NextSessionLength<T>>::take() {
            <SessionLength<T>>::put(next_len);
            true
        } else {
            false
        };
        if len_changed || !is_final_block {
            let block_number = <system::Module<T>>::block_number();
            <LastLengthChange<T>>::put(block_number);
        }
        Self::adjust_deposit_list();

        // 1. Reward directly to account 2. Click to get reward
        match Self::enable_record() {
            true => Self::reward_deposit_record(),
            _ =>  Self::reward_deposit(),
        }
    }

    fn adjust_deposit_list(){
        // Modify depositing part of all tables
        let mut int_des_vec =  Self::intentions_desposit_vec();
        while let  Some(who)=int_des_vec.pop(){
            runtime_io::print("intentions to depositing");
            // update the map of the depositing account
            let (balances,coin_type) = <IntentionsDesposit<T>>::get(who.clone());
            Self::depositing_withdraw_record(who.clone(),balances,coin_type,true);
            Self::deposit_statistics_reward(coin_type,balances,true);

            <DespositingTime<T>>::insert(who.clone(), 0);
            <IntentionsDesposit<T>>::remove(who);
        }
        <IntentionsDespositVec<T>>::put(int_des_vec);

        //===================================================================//
        // update the map of withdraw
        let mut vec_with =  Self::intentions_withdraw_vec();
        while let Some(who) = vec_with.pop() {
            runtime_io::print("withdraw to quit");
            let (balances,coin_type) = <IntentionsWithdraw<T>>::get(who.clone());
            Self::depositing_withdraw_record(who.clone(),balances,coin_type,false);
            Self::deposit_statistics_reward(coin_type,balances,false);

            <DespositingTime<T>>::remove(who.clone());
            <IntentionsWithdraw<T>>::remove(who);
        }
        <IntentionsWithdrawVec<T>>::put(vec_with);

        // update the session time of the depositing account
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            <DespositingTime<T>>::insert(v,Self::despositing_time(v)+1);
        });
    }

    /// Record the reward and click on it to get the money.
    fn count_draw_reward(accountid: T::AccountId) -> T::Balance {
        let mut reward= <RewardRecord<T>>::get(accountid.clone());
        <RewardRecord<T>>::insert(accountid,T::Balance::sa(0));
        reward
    }

    /// Record the reward and click on it to get the money.
    fn reward_deposit_record() {
        //循环历遍一下所有deposit列表，分别对每个账户的每种币的抵押的奖励进行记录
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            let depositing_vec = <DespositingBalance<T>>::get(v);
            depositing_vec.iter().enumerate().for_each(|(i,&(balances,cointype))| {
                let reward = Self::reward_set(v.clone(),<DespositingTime<T>>::get(v),balances);
                let now_reward = <RewardRecord<T>>::get(v);
                <RewardRecord<T>>::insert(v,reward+now_reward);
            });
        });
    }

    /// Money awarded directly to a specified deposit account in each session
    fn reward_deposit() {
        //循环历遍一下所有deposit列表，分别对每个账户进行单独的发钱处理
        //首先判断session 决定一个 时间比率
        //再判断balance 决定一个 存款比率
        //两个比率结合起来决定一个 乘积因子Xbalance => 然后往账户发
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            //let reward = Self::reward_set(v.clone(),<DespositingTime<T>>::get(v),<DespositingBalance<T>>::get(v));
           // let _ = <balances::Module<T>>::reward(v, reward);
        });
    }

    // RewardSessionValue  get(reward_session_value): Vec<u64>
    // RewardSessionFactor  get(reward_session_factor): Vec<u64>
    fn reward_set(who: T::AccountId, session: u32, money: T::Balance) -> T::Balance {

        let session_value = Self::reward_session_value();
        let session_factor = Self::reward_session_factor();

        let x1 = session_value[0];  let x2 = session_value[1];  let x3 = session_value[2];
        let y1 = session_factor[0];  let y2 = session_factor[1];  let y3 = session_factor[2]; let y4 = session_factor[3];

        #[warn(unused_assignments)]
            let mut final_se = 0;
        if session <= x1 {
            final_se = y1;
        } else if session <= x2 {
            final_se = y2;
        } else if session <= x3 {
            final_se = y3;
        } else {
            final_se = y4;
        }

        let balance_value = Self::reward_balance_value();
        let balance_factor = Self::reward_balance_factor();

        let xx1 = balance_value[0];  let xx2 = balance_value[1];  let xx3 = balance_value[2];
        let yy1 = balance_factor[0];  let yy2 = balance_factor[1];  let yy3 = balance_factor[2]; let yy4 = balance_factor[3];

        //let money2 = money as u64;
        let mut final_ba = 0;
        if money <= xx1 {
            final_ba = yy1;
        } else if money <= xx2 {
            final_ba = yy2;
        } else if money <= xx3 {
            final_ba = yy3;
        } else {
            final_ba = yy4;
        }

        let rate =  (final_se * final_ba);
        money*T::Balance::sa( rate as u64)/T::Balance::sa(100 as u64)
    }

    fn check_signature(who: T::AccountId, tx: T::Hash, signature: T::Hash,message_hash: T::Hash) -> Result {
        //ensure enough signature
        <signcheck::Module<T>>::check_signature(who,tx,signature ,message_hash)
    }

    pub fn check_secp512(signature: &[u8; 65], tx: &[u8; 32]) -> Result {
        //TODO: if runtime_io::secp256k1_ecdsa_recover(signature,tx).is_ok(){ } else {
        //  return Err(()); }
        //TODO:
        Ok(())
    }


    /// Record all reward statistics . The total deposit amout and reward of various types of coin.
    pub fn deposit_statistics_reward(coin_type: u64,  reward_amount: T::Balance, add: bool){
        let mut reward = <CoinReward<T>>::get(coin_type);
        if add {
            reward = reward + reward_amount;
            <CoinReward<T>>::insert(coin_type,reward);
        }else {
            //TODO:: 这里要不要判断下是否减成零了？
            reward = reward - reward_amount;
            <CoinReward<T>>::insert(coin_type,reward);
        }
    }

    /// Adjust deposing_list
    pub fn depositing_withdraw_record(who: T::AccountId, balance:T::Balance, coin_type:u64, d_w: bool){
        let mut depositing_vec = <DespositingBalance<T>>::get(who.clone());
        let mut  mark = 0usize;
        let mut new_balance = T::Balance::sa(0);
        depositing_vec.iter().enumerate().for_each( |(i,&(oldbalance,cointype))| {
            if  coin_type == cointype{
                if d_w {
                    //deposit
                    new_balance = oldbalance + balance;
                    mark = i;
                }else {
                    //withdraw
                    new_balance = oldbalance - balance;
                    mark = i;
                }
            }
        });
        // change the depositing data vector and put it back to map
        depositing_vec.insert(mark,(new_balance,coin_type));
        <DespositingBalance<T>>::insert(who.clone(),depositing_vec);

        Self::depositing_access_control(who);
    }

    /// use DepositngAccountTotal to control  DespoitingAccount
    pub fn depositing_access_control(who:T::AccountId){
        let mut deposit_total = <DepositngAccountTotal<T>>::get(who.clone());
        let all_data_vec = <DespositingBalance<T>>::get(who.clone());
        all_data_vec.iter().enumerate().for_each(|(i,&(bal,ctype))|{
            // check each cointype s deposit balance , if not zero , DepositngAccountTotal plus 1 .
            if bal != T::Balance::sa(0u64) {
                deposit_total = deposit_total + 1;
            }
        });
        <DepositngAccountTotal<T>>::insert(who.clone(),deposit_total);

        // if all type of coin is Zero ,  the accountid will be removed from the DepositingVec.
        if <DepositngAccountTotal<T>>::get(who.clone()) == 0 {
            let mut vec = Self::despositing_account();
            let mut mark = 0usize;
            vec.iter().enumerate().for_each(|(t,id)|{
                if id.clone() == who {
                    mark = t;
                }
            });
            vec.remove(mark);
            <DespoitingAccount<T>>::put(vec);
        }else {
            let mut vec = Self::despositing_account();
            vec.push(who.clone());
            <DespoitingAccount<T>>::put(vec);

        }

    }



    pub fn u8array_to_u64(arr: &[u8]) -> u64 {
        let mut len = rstd::cmp::min(8, arr.len());
        let mut ret = 0u64;
        let mut i = 0u64;
        while len > 0 {
            ret += (arr[len-1] as u64) << (i * 8);
            len -= 1;
            i += 1;
        }
        ret
    }

    // Input coin type , Output the corresponding amount of ladder balance
    pub fn deposit_exchange(coin_type:u64) -> u64 {
        5u64
    }

    pub fn balancetest(x1:T::Balance,x2:T::Balance){
        let aa = T::Balance::sa(4);
        <T as balances::Trait>::Balance::sa(5);
        x1.checked_add(&x2);
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
    use signcheck;

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

    impl Trait for Test {
        type Currency = balances::Module<Self>;
        type Event = ();
    }

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }

    type Bank = Module<Test>;

    #[test]
    fn resolving_data() {
        with_externalities(&mut new_test_ext(), || {
            //let coin_vec: Vec<_> = [1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4].to_vec();
            //let coin_hash= Decode::decode(&mut &coin_vec[..]).unwrap();
            //assert_eq!(coin_hash,44);
            let message = [0u8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,7,4,2,4,1,d,b,5,f,3,e,b,a,e,e,c,f,9,5,0,6,e,4,a,e,9,8, 8,1,8,6,0,9,3,3,4,1,6,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2, 3,8,6,f,2,6,f,c,1,0,0,0,0,e,3,5,5,4,a,6,b,1,e,f,7,3,5,8,c,d,b,8, 0,7,1,c,2,6,5,1,d,5,3,b,d,f,a,6,2,8,a,e,2,1,3,7,3,b,7,5,4,2,8,a,3,b,c,8,0,4,e,f,1,6,6,0,4].to_vec();
            //signature =[987e80e34cad5245bde33aa3df7132d6053e4d60bad87809476f46163c17d14d0c3786de0e05dcb8248658e7c4a9fd250ca889a3fba3c3b4a51078d10ca0992c00];
            let  signature = [1,2,3].to_vec();
            Bank::deposit(Origin::signed(1),message,signature);
        });
    }
}