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
#[cfg(feature = "std")]
use runtime_io::with_storage;


// use Encode, Decode
use parity_codec::{Decode, Encode};
use rstd::ops::Div;

use support::traits::Currency;

use signcheck;


#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Status {
    Lcoking,
    Unlock,
    Withdraw,
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
enum LockCycle {
    None,          //0
    OneMonth,      //1
    ThreeMonth,    //2
    SixMonth,      //3
    OneYear        //4
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TokenInfo<AccountId,Balance,Status> {
    pub id: u64,               // id
    pub sender: Vec<u8>,          // saver
    pub beneficiary: AccountId,      // beneficiary
    pub value: Balance,       // ERC amount
    pub cycle: u64,           // lock cycle
    pub reward: Balance,      // ladder reward amount
    pub txhash: Vec<u8>,      //tx hash
    pub status: Status,
    pub now_cycle : u64,
}


type TokenInfoT<T> = TokenInfo<<T as system::Trait>::AccountId,
                               <T as balances::Trait>::Balance,
                               Status>;

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
        pub fn lock_erc(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            runtime_io::print("====================lock_erc===============");
/*
            //TODO: 在这里判断 sender 是否有权限提交 后期启动节点时写入
            let validators = <session::Module<T>>::validators();
            ensure!(validators.contains(&sender),"Not validator");
*/
            // resolving message --> Ethereum's transcation hash  tx_hash   accountid
            //                       depositing amount    signature_hash
            let (txhash, who, amount, cycle, sendervec,signature_hash,id) = Self::split_message(message.clone(),signature);

            let tx_hash = Decode::decode(&mut &message[..]).unwrap();
            //check the validity and number of signatures
            match  Self::check_signature(sender.clone(), tx_hash, signature_hash, message.clone()){
                Ok(y) =>  runtime_io::print("ok") ,
                Err(x) => return Err(x),
            }
            // save the erc info
            Self::save_erc_info(who.clone(),id,sendervec.clone(),T::Balance::sa(amount),cycle,txhash.clone())?;

            Self::depositing_withdraw_record(who.clone(),T::Balance::sa(amount),1,true);
            Self::calculate_total_deposit(T::Balance::sa(amount),true);

            // emit an event
            Self::deposit_event(RawEvent:: LockToken(id,who.clone(),sendervec, T::Balance::sa(amount),cycle,
                Self::unlock_time(cycle),Decode::decode(&mut &txhash[..]).unwrap()),);
            Ok(())
        }

        /// withdraw
        /// origin, message: Vec<u8>, signature: Vec<u8>
         pub fn unlock_erc(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            //let validators = <session::Module<T>>::validators();
            //ensure!(validators.contains(&sender),"Not validator");
            // resovling message --> hash  tag  id  amount
            let (txhash,who,amount,cycle,sendervec,signature_hash,id) = Self::split_message(message.clone(),signature);
            //let message_hash = Decode::decode(&mut &message.encode()[..]).unwrap();
             let tx_hash = Decode::decode(&mut &message[..]).unwrap();
            //check the validity and number of signatures
            match  Self::check_signature(sender.clone(), tx_hash, signature_hash, message.clone()){
                Ok(y) =>  runtime_io::print("ok") ,
                Err(x) => return Err(x),
            }
            // ensure no repeat
            ensure!(!Self::despositing_account().iter().find(|&t| t == &who).is_none(), "Cannot deposit if not depositing.");
            //ensure!(Self::intentions_withdraw_vec().iter().find(|&t| t == &who).is_none(), "Cannot withdraw2 if already in withdraw2 queue.");

            if Self::is_valid_unlock(who.clone(),id).is_ok(){
                Self::depositing_withdraw_record(who.clone(),T::Balance::sa(amount),1,false);
                Self::calculate_total_deposit(T::Balance::sa(amount),false);

                Self::change_status(who.clone(),id);
            }else{
                return Err("cant unlock, still locking");
            }
            // emit an event
             Self::deposit_event(RawEvent:: UnLockToken(id,who.clone(),sendervec, T::Balance::sa(amount),cycle,
                 Self::unlock_time(cycle),Decode::decode(&mut &txhash[..]).unwrap()),);
            Ok(())
        }

        /// set session lenth
        fn set_session_lenth(session_len: u64 ){
           ensure!(session_len >= 10,"the session lenth must larger than 10");
           <SessionLength<T>>::put(T::BlockNumber::sa(session_len));
        }



        /// a new session starts
		fn on_finalize(n: T::BlockNumber) {
		    Self::check_rotate_session(n);
		}
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Erc {
        /// bank & session

        /// record depositing info of balance & session_time
        DespoitingAccount get(despositing_account): Vec<T::AccountId>;
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

        /// Investment proportion. Controlling the Ratio of External Assets to Local Assets
        DespositExchangeRate get(desposit_exchange_rate) :  u64 = 10000000000000;
        // 总锁仓
        TotalLockToken get(total_lock_token) : T::Balance;

        // 所有人锁仓记录
        LockTokenList get(lock_token_list) : map T::AccountId => T::Balance;
        // all reward
        TotalReward get(total_reward): T::Balance;
        // Lockinfo list
        LockInfoList get(lock_info_list): map (T::AccountId,u64) => Option<TokenInfoT<T>>;
        // record one user's all index
        LockCount get(lock_count) : map T::AccountId => Vec<u64>;
    }
        add_extra_genesis {
        config(total) : u64;
        build(|storage: &mut sr_primitives::StorageOverlay, _: &mut sr_primitives::ChildrenStorageOverlay, config: &GenesisConfig<T>| {
            with_storage(storage, || {
                <Module<T>>::inilize_deposit_data();
            })
        })
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

        LockToken(u64, AccountId,Vec<u8>, Balance, u64,  u64, Hash),
        UnLockToken(u64,AccountId,Vec<u8>, Balance, u64,  u64, Hash),
    }
}

impl<T: Trait> Module<T>
{
    fn  split_message( message: Vec<u8>, signature: Vec<u8>) -> (Vec<u8>,T::AccountId,u64,u64,Vec<u8>,T::Hash,u64) {

        // message
        let mut messagedrain = message.clone();

        // Coin 0-32
        let mut id_vec: Vec<_> = messagedrain.drain(0..32).collect();
        id_vec.drain(0..16);
        let mut id = Self::u8array_to_u128(id_vec.as_slice());

        //sender
        let mut sender_vec: Vec<_> = messagedrain.drain(0..32).collect();
        let sender = sender_vec.drain(0..20).collect();

        // Who 33-64  beneficiary
        let mut who_vec: Vec<_> = messagedrain.drain(0..32).collect();
        let who: T::AccountId = Decode::decode(&mut &who_vec[..]).unwrap();

        //65-96  value
        let mut amount_vec:Vec<u8> = messagedrain.drain(0..32).collect();
        amount_vec.drain(0..16);
        let mut amountu128 = Self::u8array_to_u128(amount_vec.as_slice());
        amountu128 = ((amountu128 as f64)/<DespositExchangeRate<T>>::get() as f64) as u128;

        let amountu64= amountu128 as u64;

        //65-96  cycle
        let mut cycle_vec:Vec<u8> = messagedrain.drain(0..32).collect();
        cycle_vec.drain(0..16);
        let mut cycle128 = Self::u8array_to_u128(amount_vec.as_slice());

        let hash:Vec<u8> = messagedrain.drain(0..32).collect();
        //let tx_hash = Decode::decode(&mut &hash[..]).unwrap();

        // Signature_Hash
        let signature_hash =  Decode::decode(&mut &signature[..]).unwrap();

        return (hash, who, amountu64, cycle128 as u64, sender,signature_hash,id as u64);
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
            runtime_io::print("Start new session of erc");
            Self::rotate_session(is_final_block, apply_rewards);
        }
    }

    /// The last length change, if there was one, zero if not.
    pub fn last_length_change() -> T::BlockNumber {
        <LastLengthChange<T>>::get().unwrap_or_else(T::BlockNumber::zero)
    }

    /// Move onto next session: register the new authority set.
    pub fn rotate_session(is_final_block: bool, _apply_rewards: bool) {
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
            _ =>  Self::reward_deposit(),
            true =>  Self::reward_deposit(),
        }
    }

    fn adjust_deposit_list(){
        //do_nothing
        // update the session time of the depositing account
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            Self::lock_count(v).iter().enumerate().for_each(|(_i,&index)| {
                if let Some(mut r) = <LockInfoList<T>>::get((v.clone(),index)){
                    let now = <timestamp::Module<T>>::get();
                    let u64now = T::Moment::as_(now);
                    if u64now >= r.now_cycle {
                        r.status = Status::Unlock;
                    }
                }
            });
        });
    }


    /// Money awarded directly to a specified deposit account in each session
    /// each session calculate each lockerc's reward
    fn reward_deposit() {
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            Self::lock_count(v).iter().enumerate().for_each(|(_i,&index)| {
                if let Some(mut r) = <LockInfoList<T>>::get((v.clone(),index)){
                    if r.status == Status::Withdraw {
                        //ignore
                    }else {
                        //calculate reward
                        let reward = Self::calculate_reward(r.cycle,r.value);
                        //accmulate the total reward for all users
                        Self::calculate_total_reward(reward,true);
                        // accmulate the total reward for one user
                        let now_reward = <RewardRecord<T>>::get(v);
                        <RewardRecord<T>>::insert(v,reward+now_reward);
                        // reward to user
                        T::Currency::deposit_into_existing(&v, <BalanceOf<T> as As<u64>>::sa(T::Balance::as_(reward)));
                    }
                }
            });
        });
    }

    fn calculate_reward(cycle: u64, erc:T::Balance) -> T::Balance {
        let rewardrate = match cycle {
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            _ => 0,
        };
        T::Balance::sa((T::Balance::as_(erc) as f64 * rewardrate as f64/100f64  )as u64)
    }

    fn check_signature(who: T::AccountId, tx: T::Hash, signature: T::Hash,message_hash: Vec<u8>) -> Result {
        //ensure enough signature
        <signcheck::Module<T>>::check_signature(who,tx,signature ,message_hash)
    }

    pub fn check_secp512(signature: &[u8; 65], tx: &[u8; 32]) -> Result {
        Ok(())
    }

    pub fn initlize(who: T::AccountId){
        let mut depositing_erc = <LockTokenList<T>>::get(who.clone());
        depositing_erc = T::Balance::sa(0);
        <LockTokenList<T>>::insert(who.clone(),depositing_erc);
    }

    fn uninitlize(who: T::AccountId) {
        <LockTokenList<T>>::remove(who.clone());
    }

    /// Adjust deposing_list            beneficiary         value              cycle           in out
    pub fn depositing_withdraw_record(who: T::AccountId, balance:T::Balance, cycle:u64, d_w: bool){
        if Self::despositing_account().iter().find(|&t| t == &who).is_none() {
            Self::initlize(who.clone());
        }
        let mut depositing_erc = <LockTokenList<T>>::get(who.clone());
        let mut new_balance = T::Balance::sa(0);
         if d_w {
              //deposit
             new_balance = depositing_erc + balance;
         }else {
             //withdraw
             new_balance = depositing_erc - balance;
         }
        // change the depositing data vector and put it back to map
        <LockTokenList<T>>::insert(who.clone(),new_balance);
        Self::depositing_access_control(who);
    }

    /// use DepositngAccountTotal to control  DespoitingAccount
    pub fn depositing_access_control(who:T::AccountId){
        //let mut deposit_total = <DepositngAccountTotal<T>>::get(who.clone());
        let mut deposit_total = 0;
        let all_data_vec = <LockTokenList<T>>::get(who.clone());

        // if all type of coin is Zero ,  the accountid will be removed from the DepositingVec.
        if all_data_vec == T::Balance::sa(0) {
            let mut vec = Self::despositing_account();
            let mut mark = 0usize;
            vec.iter().enumerate().for_each(|(t,id)|{
                if id.clone() == who {
                    mark = t;
                }
            });
            vec.remove(mark);
            <DespoitingAccount<T>>::put(vec);
            Self::uninitlize(who.clone());
        }else {
            let mut vec = Self::despositing_account();
            if  !vec.contains(&who.clone()){
                vec.push(who.clone());
                <DespoitingAccount<T>>::put(vec);
            }
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

    pub fn u8array_to_u128(arr: &[u8]) -> u128 {
        let mut len = rstd::cmp::min(16, arr.len());
        let mut ret = 0u128;
        let mut i = 0u128;
        while len > 0 {
            ret += (arr[len-1] as u128) << (i * 8);
            len -= 1;
            i += 1;
        }
        ret
    }

    pub fn inilize_deposit_data() {
        <TotalLockToken<T>>::put(T::Balance::sa(0));
    }

    //
    pub fn calculate_total_deposit( balance:T::Balance, in_out:bool){
        let mut money = Self::total_lock_token();
        if in_out {
            money = money + balance;
        }else {
            money = money - balance;
        }
        <TotalLockToken<T>>::put(money);
    }

    pub fn calculate_total_reward( balance:T::Balance, in_out:bool){
        let mut money = Self::total_reward();
        if in_out {
            money = money + balance;
        }else {
            money = money - balance;
        }
        <TotalReward<T>>::put(money);
    }

    pub fn unlock_time(cycle:u64) -> u64 {
        let offset = match cycle {
            0 => 0u64,
            1 => 2592000,
            2 => 7776000,
            3 => 15552000,
            4 => 31104000,
            _ => 0,
        };
        let xx =T::Moment::as_(<timestamp::Module<T>>::get())+ offset;
        xx
    }

    pub fn save_erc_info(beneficiary:T::AccountId,index:u64,sender:Vec<u8>,value:T::Balance,cycle:u64,txhash: Vec<u8>) -> Result{
        // find the infomation
        if let Some(r) = <LockInfoList<T>>::get((beneficiary.clone(),index)){
            //  already have
            return Err("duplicate erc lock info");
        }else {
            let mut lock_count_vec = <LockCount<T>>::get(beneficiary.clone());
            lock_count_vec.push(index);
            <LockCount<T>>::insert(beneficiary.clone(),lock_count_vec);

            let new_info = TokenInfo{
                id:index,
                sender:sender.clone(),
                beneficiary:beneficiary.clone() ,
                value:value,
                cycle:cycle,
                reward: T::Balance::sa(0),
                txhash:txhash,
                status: Status::Lcoking,
                now_cycle:Self::unlock_time(cycle),
            };
            <LockInfoList<T>>::insert((beneficiary.clone(),index),new_info);
        }
        Ok(())
    }

    ///make sure the account lock the Regulations times
    pub fn is_valid_unlock(beneficiary:T::AccountId,index:u64,) -> Result{
        if let Some(mut r) = <LockInfoList<T>>::get((beneficiary.clone(),index)){
            //
            if r.status == Status::Unlock{
                return Ok(());
            }else {
                return Err("still locking,cant withdraw");
            }

        }else {
            //  already have
            return Err("no info find");
        }
    }

    pub fn change_status(beneficiary:T::AccountId,index:u64) -> Result{
        if let Some(mut r) = <LockInfoList<T>>::get((beneficiary.clone(), index)) {
            r.status = Status::Withdraw;
            <LockInfoList<T>>::insert((beneficiary.clone(),index),r);
            return Ok(());
        }else {
            return Err("not find record");
        }
    }

    pub fn get_reward(beneficiary:T::AccountId,index:u64){
        if let Some(mut r) = <LockInfoList<T>>::get((beneficiary.clone(),index)){
            let old_reward = r.reward;

        }
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

    impl Trait for Test {
        type Currency = balances::Module<Self>;
        type Event = ();
    }

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }

    type Erc = Module<Test>;

    #[test]
    fn resolving_data() {
        with_externalities(&mut new_test_ext(), || {
            let coin_vec: Vec<_> = [1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4].to_vec();
            // Self::save_erc_info(who.clone(),id,sendervec,T::Balance::sa(amount),cycle,txhash)?;
            assert_ok!(Erc::save_erc_info(5,4.clone(),coin_vec.clone(),T::Balance::sa(5),5,coin_vec));
        });
    }


    #[test]
    fn inilize_test() {
        with_externalities(&mut new_test_ext(), || {
            //assert_eq!(Bank::coin_deposit(0),<tests::Test as Trait>::Balance::sa(0));
        });
    }

}