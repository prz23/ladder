#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

use sr_primitives::traits::{As, CheckedSub, Hash, One, Zero};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap,
    StorageValue,
};

use system::ensure_signed;

use rstd::prelude::*;

#[cfg(feature = "std")]
pub use std::fmt;
#[cfg(feature = "std")]
use runtime_io::with_storage;

// use Encode, Decode
use parity_codec::{Decode, Encode};

use support::traits::Currency;

use signcheck;

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum LockType {
    OTCLock,
    WithDrawLock,
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum TokenType {
    Free,
    OTC,
    WithDraw,
    Reward,
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: balances::Trait + order::Trait{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// deposit some token into the specific account
        pub fn deposit(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            if message.len() != 124 { return Err("Message Invalid Length");}
            // Parsing data
            let (tx_hash, who, amount, signature_hash, coin_type, sendervec) = Self::split_message(message.clone(),signature);
            if amount == 0 { return Err("Deposit token of the amout of Zero is Prohibited")};

            //check the validity and number of signatures
            match  Self::check_signature(sender.clone(), tx_hash, signature_hash, message.clone()){
                Ok(_y) =>  runtime_io::print("Signature is Verified") ,
                Err(x) => return Err(x),
            }

            // Modify the storage of the token and relate support data.
            Self::deposit_to_free(who.clone(), sendervec.clone(), coin_type, amount)?;

            //a support data record the total deposit token
            Self::calculate_total_deposit(coin_type, T::Balance::sa(amount), true);
            <DespositingTime<T>>::insert(who.clone(), 0);
            // emit an event
            Self::deposit_event(RawEvent::Depositing(coin_type, who, sendervec, T::Balance::sa(amount), tx_hash));
            Ok(())
        }

        /// Take the initiative to receive awards
        pub fn draw_reward_uesless(origin) -> Result {
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

            //let validators = <session::Module<T>>::validators();
            //ensure!(validators.contains(&sender),"Not validator");
            if message.len() != 156 { return Err("Message Invalid Length");}
            // Parsing data
            let (tx_hash,who,amount,signature_hash,coin_type,id,sendervec) = Self::split_message2(message.clone(),signature);

            //check the validity and number of signatures
            match  Self::check_signature(sender.clone(), tx_hash, signature_hash, message.clone()){
                Ok(_y) =>  runtime_io::print("Signature is Verified") ,
                Err(x) => return Err(x),
            }

            // ensure no repeat
            //ensure!(!Self::despositing_account().iter().find(|&t| t == &who).is_none(), "Cannot deposit if not depositing.");

            Self::withdraw_from_withdraw(who.clone(),sendervec.clone(),coin_type,amount)?;
            // Self::depositing_withdraw_record(who.cle(),T::Balance::sa(amount),coin_type,false);
            //Self::lock_unlock_record(who.clone(),T::Balance::sa(amount),coin_type,LockType::WithDrawLock,false);
            Self::calculate_total_deposit(coin_type,T::Balance::sa(amount),false);
            // emit an event
            Self::deposit_event(RawEvent::Withdraw(coin_type,id,who,sendervec,T::Balance::sa(amount),tx_hash));
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

        /// withdraw the reward
        pub fn draw_reward(origin) -> Result{
             //ensure!(!Self::despositing_account().iter().find(|&t| t == &id).is_none(), "Cannot draw if not depositing.");

            let who = ensure_signed(origin)?;
            let mut reward = 0u64;
            // calculate the reward of who's all cointype and outside address
            Self::iterator_all_token_for_who(who.clone(),|coin_type,sender|{
                reward = reward + Self::deposit_reward_token((who.clone(),sender.clone(),coin_type));
                Self::take_out_reward_token(who.clone(),sender.clone(),coin_type)?;
                Ok(())
            });

             <RewardRecord<T>>::insert(&who,T::Balance::sa(0));

            let new_balance = <BalanceOf<T> as As<u64>>::sa(reward);
            T::Currency::deposit_creating(&who, new_balance);
            Ok(())
        /*    match T::Currency::deposit_creating(&who, new_balance){
                 Err(x) => Err(x),
                 _ => Ok(()),
             }*/
        }

        pub fn request(origin, message: Vec<u8>, signature: Vec<u8>) -> Result{
            let sender = ensure_signed(origin)?;

            //let validators = <session::Module<T>>::validators();
            //ensure!(validators.contains(&sender),"Not validator");

            if message.len() != 156 { return Err("Message Invalid Length");}
            // Parsing data
            let (tx_hash,who,amount,signature_hash,coin_type,id,sendervec) = Self::split_message2(message.clone(),signature);

            //check the validity and number of signatures
            match  Self::check_signature(sender.clone(), tx_hash, signature_hash, message.clone()){
                Ok(_y) =>  runtime_io::print("Signature is Verified") ,
                Err(x) => return Err(x),
            }
            // ensure no repeat
            ensure!(!Self::deposit_ladder_account_list().iter().find(|&t| t == &who).is_none(), "Cannot deposit if not depositing.");
/*
            if amount > Self::ptotential_withdrawable_coin(who.clone(),sendervec.clone(),coin_type){
                return Err("Cant request too much");
            }
            */
            let lock_withdraw_balance = Self::withdraw_request(who.clone(),amount,coin_type,sendervec.clone());

            if lock_withdraw_balance == T::Balance::sa(0) { return Ok(());}

            Self::deposit_event(RawEvent::WithdrawRequest(coin_type,id,who,sendervec,lock_withdraw_balance,tx_hash));
            Ok(())
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

        // Token Record
        DepositFreeToken get(deposit_free_token) : map (T::AccountId,Vec<u8>,u64) => u64;
        DepositOtcToken get(deposit_otc_token) : map (T::AccountId,Vec<u8>,u64) => u64;
        DepositWithdrawToken get(deposit_withdraw_token) : map (T::AccountId,Vec<u8>,u64) => u64;
        DepositRewardToken get(deposit_reward_token) : map (T::AccountId,Vec<u8>,u64) => u64;
        // attached record
        DepositLadderAccountList get(deposit_ladder_account_list) : Vec<T::AccountId>;
        DepositAccountCoinList get(deposit_account_coin_list) : map T::AccountId => Vec<u64>;
        DepositSenderList get(deposit_sender_list) : map (T::AccountId,u64) => Vec<Vec<u8>>;
        // statistic record
        TotalTokenCoin get(total_token_coin) : map u64 => u128 ;  // coin => total

        /// accountid => (balance , type)
        DespositingBalance get(despositing_banance): map T::AccountId => Vec<(T::Balance,u64)>;
        DespositingTime get(despositing_time): map T::AccountId => u32;
        /// Locked token for otc
        DespositingBalanceReserved get(despositing_banance_reserved): map T::AccountId => Vec<(T::Balance,u64)>;
        /// Locked token for withdraw
        DespositingBalanceWithdraw get(despositing_banance_withdraw): map T::AccountId => Vec<(T::Balance,u64)>;

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

		/// record the reward
		RewardRecord get(reward_record):  map T::AccountId => T::Balance;
		///
		EnableRewardRecord get(enable_record) config(): bool;
        ///
        TotalDespositingBalacne  get(total_despositing_balance) : T::Balance;
        /// MAP of cointype => (TotalDeposit , AllReward)
        CoinDeposit get(coin_deposit) : map u64 => T::Balance;
        CoinReward get(coin_reward) : map u64 => T::Balance;

        /// Investment proportion. Controlling the Ratio of External Assets to Local Assets
        DespositExchangeRate get(desposit_exchange_rate) :  u64 = 1000000000;

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
        <T as system::Trait>::BlockNumber,
        <T as system::Trait>::Hash
    {
        ///bank moduel
		/// All validators have been rewarded by the given balance.
		Reward(Balance),
		/// accountid added to the intentions to deposit queue
		/// Self::deposit_event(RawEvent::Depositing(coin_type,who,senderH160,T::Balance::sa(amount),tx_hash));
		Depositing(u64,AccountId,Vec<u8>,Balance,Hash),

		//Self::deposit_event(RawEvent::Withdraw(coin_type,who,sendervec,T::Balance::sa(amount),tx_hash));
		Withdraw(u64,u64,AccountId,Vec<u8>,Balance,Hash),
		 //Self::deposit_event(RawEvent::WithdrawRequest(coin_type,id,who,sendervec,T::Balance::sa(withdraw_amount),tx_hash));
		WithdrawRequest(u64,u64,AccountId,Vec<u8>,Balance,Hash),

        /// a new seesion start
        NewRewardSession(BlockNumber),
    }
}

impl<T: Trait> Module<T>
{
    fn  split_message( message: Vec<u8>, signature: Vec<u8>) -> (T::Hash,T::AccountId,u64,T::Hash,u64,Vec<u8>) {

        // message --> hash  tag  id  amount
        let mut messagedrain = message.clone();

        // Coin 0-32
        let coin_vec: Vec<_> = messagedrain.drain(0..8).collect();
        let coin_type = Self::u8array_to_u64(coin_vec.as_slice());
        //sender
        let mut sender_vec: Vec<_> = messagedrain.drain(0..20).collect();
        let sender: Vec<u8> = sender_vec.drain(0..20).collect();

        // Who 33-64
        let who_vec: Vec<_> = messagedrain.drain(0..32).collect();
        let who: T::AccountId = Decode::decode(&mut &who_vec[..]).unwrap();

        //65-96
        let mut amount_vec:Vec<u8> = messagedrain.drain(0..32).collect();
        amount_vec.drain(0..16);
        let mut amountu128 = Self::u8array_to_u128(amount_vec.as_slice());
        amountu128 = ((amountu128 as f64)/<DespositExchangeRate<T>>::get() as f64) as u128;

        let amountu64= amountu128 as u64;
        // Tx_Hash 97-128
        let hash:Vec<u8> = messagedrain.drain(0..32).collect();
        let tx_hash = T::Hashing::hash( &hash[..]);

        // Signature_Hash
        let signature_hash =  T::Hashing::hash( &signature[..]);

        return (tx_hash,who,amountu64,signature_hash,coin_type,sender);
    }

    fn  split_message2( message: Vec<u8>, signature: Vec<u8>) -> (T::Hash,T::AccountId,u64,T::Hash,u64,u64,Vec<u8>) {

        // message --> hash  tag  id  amount
        let mut messagedrain = message.clone();

        // Coin 0-32
        let coin_vec: Vec<_> = messagedrain.drain(0..8).collect();
        //coin_vec.drain(0..24);
        let coin_type = Self::u8array_to_u64(coin_vec.as_slice());

        // Id 0-32
        let mut id_vec: Vec<_> = messagedrain.drain(0..32).collect();
        id_vec.drain(0..16);
        let id = Self::u8array_to_u128(id_vec.as_slice());


        //sender
        let mut sender_vec: Vec<_> = messagedrain.drain(0..20).collect();
        let sender:Vec<u8> = sender_vec.drain(0..20).collect();

        // Who 33-64
        let who_vec: Vec<_> = messagedrain.drain(0..32).collect();
        let who: T::AccountId = Decode::decode(&mut &who_vec[..]).unwrap();

        //65-96
        let mut amount_vec:Vec<u8> = messagedrain.drain(0..32).collect();
        amount_vec.drain(0..16);
        let mut amountu128 = Self::u8array_to_u128(amount_vec.as_slice());
        amountu128 = ((amountu128 as f64)/<DespositExchangeRate<T>>::get() as f64) as u128;

        let amountu64= amountu128 as u64;
        // Tx_Hash 97-128
        let hash:Vec<u8> = messagedrain.drain(0..32).collect();
        let tx_hash = T::Hashing::hash( &hash[..]);

        // Signature_Hash
        let signature_hash =  T::Hashing::hash( &signature[..]);

        return (tx_hash,who,amountu64,signature_hash,coin_type,id as u64,sender);
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
            true => { //Self::reward_deposit_record();
                     Self::calculate_reward_and_reward();},
            _ =>  Self::reward_deposit(),
        }
    }

    fn adjust_deposit_list(){
        // update the session time of the depositing account
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            <DespositingTime<T>>::insert(v,Self::despositing_time(v)+1);
        });
    }

    /// Record the reward and click on it to get the money.
    fn count_draw_reward(accountid: T::AccountId) -> T::Balance {
        let reward= <RewardRecord<T>>::get(accountid.clone());
        <RewardRecord<T>>::insert(accountid,T::Balance::sa(0));
        reward
    }

    /* Record the reward and click on it to get the money.
    fn reward_deposit_record() {
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            let depositing_vec = <DespositingBalance<T>>::get(v);
            depositing_vec.iter().enumerate().for_each(|(_i,&(balances,cointype))| {
                let reward = Self::reward_set(v.clone(),<DespositingTime<T>>::get(v),balances);
                let now_reward = <RewardRecord<T>>::get(v);
                <RewardRecord<T>>::insert(v,reward+now_reward);
                Self::calculate_total_reward(cointype,balances,true);
            });
        });
    }
    */

    /// Money awarded directly to a specified deposit account in each session
    fn reward_deposit() {
        Self::despositing_account().iter().enumerate().for_each(|(_i,_v)|{
        });
    }

/*
    fn reward_set(_who: T::AccountId, session: u32, money: T::Balance) -> T::Balance {

        let session_value = Self::reward_session_value();
        let session_factor = Self::reward_session_factor();

        let x1 = session_value[0];  let x2 = session_value[1];  let x3 = session_value[2];
        let y1 = session_factor[0];  let y2 = session_factor[1];  let y3 = session_factor[2]; let y4 = session_factor[3];

        let final_se =
        if session <= x1 {
            y1
        } else if session <= x2 {
            y2
        } else if session <= x3 {
            y3
        } else {
            y4
        };

        let balance_value = Self::reward_balance_value();
        let balance_factor = Self::reward_balance_factor();

        let xx1 = balance_value[0];  let xx2 = balance_value[1];  let xx3 = balance_value[2];
        let yy1 = balance_factor[0];  let yy2 = balance_factor[1];  let yy3 = balance_factor[2]; let yy4 = balance_factor[3];

        let final_ba =
        if money <= xx1 {
            yy1
        } else if money <= xx2 {
            yy2
        } else if money <= xx3 {
            yy3
        } else {
            yy4
        };

        let rate =  final_se * final_ba;
        money*T::Balance::sa(rate as u64)/T::Balance::sa(100 as u64)
    }
*/
    fn check_signature(who: T::AccountId, tx: T::Hash, signature: T::Hash,message_hash: Vec<u8>) -> Result {
        //ensure enough signature
        <signcheck::Module<T>>::check_signature(who,tx,signature ,message_hash)
    }


    pub fn initlize(who: T::AccountId){
        let mut depositing_vec = <DespositingBalance<T>>::get(who.clone());
        for i in 0u64..5u64{
            depositing_vec.push((T::Balance::sa(0),i));
        }
        <DespositingBalance<T>>::insert(who.clone(),depositing_vec);
    }

    fn uninitlize(who: T::AccountId) {
        <DespositingBalance<T>>::remove(who.clone());
    }

    /// Adjust lock_money
    pub fn lock_unlock_record(who: T::AccountId, balance:T::Balance, coin_type:u64, lock_type:LockType,d_w: bool){
        // Determine if the person has a list, create a new one if not
        Self::check_lock_exist(who.clone(),lock_type);
        // modify the balance
        let mut new_lock_bal = T::Balance::sa(0);
        let mut mark = 0usize;
        let mut depositing_vec_lock = match lock_type {
            LockType::OTCLock => <DespositingBalanceReserved<T>>::get(who.clone()) ,
            LockType::WithDrawLock => <DespositingBalanceWithdraw<T>>::get(who.clone())
        };
        depositing_vec_lock.iter().enumerate().for_each( |(i,&(bal,cointype))| {
            if cointype == coin_type {
                if d_w {
                    new_lock_bal = bal + balance;
                    mark = i;
                }else{
                    //new_lock_bal = bal - balance;
                    new_lock_bal = match bal.checked_sub(&balance) {
                        Some(a) => a,
                        None => T::Balance::sa(0),
                    };
                    mark = i;
                }
            }
        });
        depositing_vec_lock[mark]= (new_lock_bal,coin_type);
        match lock_type {
            LockType::OTCLock => <DespositingBalanceReserved<T>>::insert(who.clone(),depositing_vec_lock) ,
            LockType::WithDrawLock => <DespositingBalanceWithdraw<T>>::insert(who.clone(),depositing_vec_lock)
        };
        Self::lock_access_control(who.clone(),lock_type);
    }

    /// Find out if the person has locked token , uninitialization of storage
    pub fn lock_access_control(who:T::AccountId, lock_type:LockType){
        let mut deposit_total = 0;
        let all_data_vec = match lock_type {
            LockType::OTCLock => <DespositingBalanceReserved<T>>::get(who.clone()) ,
            LockType::WithDrawLock => <DespositingBalanceWithdraw<T>>::get(who.clone())
        };
        all_data_vec.iter().enumerate().for_each(|(_i,&(bal,_ctype))|{
            // check each cointype s deposit balance , if not zero , DepositngAccountTotal plus 1 .
            if bal != T::Balance::sa(0u64) {
                deposit_total = deposit_total + 1;
            }
        });
        if deposit_total == 0{
            Self::uninit_lock(who.clone(),lock_type);
        }
     }

    /// Adjust deposing_list
    pub fn depositing_withdraw_record(who: T::AccountId, balance:T::Balance, coin_type:u64, d_w: bool){
        if Self::despositing_account().iter().find(|&t| t == &who).is_none() {
            Self::initlize(who.clone());
        }
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
                    //new_balance = oldbalance - balance;
                    new_balance = match oldbalance.checked_sub(&balance) {
                        Some(a) => a,
                        None => T::Balance::sa(0),
                    };
                    mark = i;
                }
            }
        });
        // change the depositing data vector and put it back to map
        depositing_vec[mark] = (new_balance,coin_type);
        //depositing_vec.insert(mark,(new_balance,coin_type));
        <DespositingBalance<T>>::insert(who.clone(),depositing_vec);
        Self::depositing_access_control(who);
    }
    /// use DepositngAccountTotal to control  DespoitingAccount
    pub fn depositing_access_control(who:T::AccountId){
        //let mut deposit_total = <DepositngAccountTotal<T>>::get(who.clone());
        let mut deposit_total = 0;
        let all_data_vec = <DespositingBalance<T>>::get(who.clone());
        all_data_vec.iter().enumerate().for_each(|(_i,&(bal,_ctype))|{
            if bal != T::Balance::sa(0u64) {
                deposit_total = deposit_total + 1;
            }
        });
        <DepositngAccountTotal<T>>::insert(who.clone(),deposit_total);
        // if all type of coin is Zero ,  the accountid will be removed from the DepositingVec.
        if <DepositngAccountTotal<T>>::get(who.clone()) == 0 {
            let mut vec = Self::despositing_account();
            if vec == [].to_vec() { return ;}
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
        for i in 0u64..5u64 {
            <CoinDeposit<T>>::insert(i,T::Balance::sa(0));
            <CoinReward<T>>::insert(i,T::Balance::sa(0));
        }
        <order::Module<T>>::init_basic_pair();
    }

    pub fn calculate_total_deposit(coin_type:u64, balance:T::Balance, in_out:bool){
        let mut money = Self::coin_deposit(coin_type);
        if in_out {
            money = money + balance;
        }else {
            //money = money - balance;
            money = match money.checked_sub(&balance) {
                Some(a) => a,
                None => T::Balance::sa(0),
            };
        }
        <CoinDeposit<T>>::insert(coin_type,money);
    }

    pub fn calculate_total_reward(coin_type:u64, balance:T::Balance, in_out:bool){
        let mut money = Self::coin_reward(coin_type);
        if in_out {
            money = money + balance;
        }else {
            //money = money - balance;
            money = match money.checked_sub(&balance) {
                Some(a) => a,
                None => T::Balance::sa(0),
            };
        }
        <CoinReward<T>>::insert(coin_type,money);
    }

    // Input coin type , Output the corresponding amount of ladder balance
    pub fn deposit_exchange(_coin_type:u64) -> u64 {
        5u64
    }
/*
    pub fn balancetest(x1:T::Balance,x2:T::Balance){
        //let aa = T::Balance::sa(4);
        //<T as balances::Trait>::Balance::sa(5);
        //x1.checked_add(&x2);
    }
*/
    // make sure the lock record exist
    pub fn check_lock_exist(who:T::AccountId,lock_type:LockType) {
        //let mut is_none = false;
        let depositing_vec_lock = match lock_type {
            LockType::OTCLock => <DespositingBalanceReserved<T>>::get(who.clone()) ,
            LockType::WithDrawLock => <DespositingBalanceWithdraw<T>>::get(who.clone())
        };
        if depositing_vec_lock == [].to_vec(){
            // if not make a new lock record
            Self::init_lock(who.clone(),lock_type);
        }
    }
    pub fn init_lock(who :T::AccountId , lock_type: LockType){
        let mut depositing_vec_lock = match lock_type {
            LockType::OTCLock => <DespositingBalanceReserved<T>>::get(who.clone()) ,
            LockType::WithDrawLock => <DespositingBalanceWithdraw<T>>::get(who.clone())
        };
        for i in 0u64..5u64{
            depositing_vec_lock.push((T::Balance::sa(0),i));
        }
        match lock_type {
            LockType::OTCLock => <DespositingBalanceReserved<T>>::insert(who.clone(),depositing_vec_lock) ,
            LockType::WithDrawLock => <DespositingBalanceWithdraw<T>>::insert(who.clone(),depositing_vec_lock)
        };
    }

    pub fn uninit_lock(who :T::AccountId,lock_type:LockType){
        match lock_type {
            LockType::OTCLock => <DespositingBalanceReserved<T>>::remove(who.clone()) ,
            LockType::WithDrawLock => <DespositingBalanceWithdraw<T>>::remove(who.clone())
        };
    }

    /// lock the amount of coin of who
    pub fn lock(who:T::AccountId, coin: u64 , amount: u64, lock_type:LockType){
        Self::lock_unlock_record(who.clone(),T::Balance::sa(amount),coin,lock_type,true);
        Self::depositing_withdraw_record(who.clone(),T::Balance::sa(amount),coin,false);
    }

    /// unlock the amount of coin of who
    pub fn unlock(who:T::AccountId, coin: u64 , amount: u64 , lock_type:LockType){
        Self::lock_unlock_record(who.clone(),T::Balance::sa(amount),coin,lock_type,false);
        Self::depositing_withdraw_record(who.clone(),T::Balance::sa(amount),coin,true);
    }

    /// After the purchase operation, the amount of the two parties is modified
    /// by modifying the buyer's ID and the buyer's bid amount of two kinds of currencies.
    pub fn buy_operate(buyer:T::AccountId,seller:T::AccountId, type_share:u64,
                       type_money:u64 ,price:u64 ,amount:u64, sell_res:bool, buy_res:bool ,
                       sellsender:Vec<u8>, sellrevicer:Vec<u8>, buyersender:Vec<u8>, buyerreciver:Vec<u8>) -> Result{
        let token_needed_105 = amount as f64 * price as f64;
        let token_needed = token_needed_105/ <order::Module<T>>::price_exchange_rate() as f64;

        // Deducting the amount locked in by the seller and transferring the buyer's money.
        // unlock the reserved token anyway
        Self::modify_token(seller.clone(),sellsender.clone(),type_share,amount,TokenType::OTC,false)?;
        if sell_res {
            Self::modify_token(seller.clone(),sellrevicer.clone(),type_money,token_needed as u64,TokenType::Free,true)?;
        }
        // At the same time deduct the buyer's money anyway
        Self::modify_token(buyer.clone(),buyersender.clone(),type_money,token_needed as u64,TokenType::Free,false)?;
        if buy_res {
            Self::modify_token(buyer.clone(),buyerreciver.clone(),type_share,amount,TokenType::Free,true)?;
        }
        Ok(())
    }

    // return the amout of the token of  the cointype of who
    pub fn specific_token_free_amount(who:T::AccountId , cointype: u64) -> T::Balance{
        let mut amount_on_sale = T::Balance::sa(0);
        let all_data_vec = <DespositingBalance<T>>::get(who.clone());
        if all_data_vec == [].to_vec() { return amount_on_sale;}

        all_data_vec.iter().enumerate().for_each(|(_i,&(bal,ctype))|{
            // check each cointype s deposit balance , if not zero , DepositngAccountTotal plus 1 .
            if ctype == cointype {
                if bal == T::Balance::sa(0) {
                }else {
                    amount_on_sale = bal;
                }
            }
        });
        amount_on_sale
    }

    /// Find out if the person(who) has locked token(cointype) ,is true return the amount
    pub fn is_specific_token_on_sale (who:T::AccountId , cointype: u64 ,_lock_type:LockType) -> T::Balance{
        let all_data_vec = <DespositingBalanceReserved<T>>::get(who.clone());
        let mut amount_on_sale = T::Balance::sa(0);
        all_data_vec.iter().enumerate().for_each(|(_i,&(bal,ctype))|{
            // check each cointype s deposit balance , if not zero , DepositngAccountTotal plus 1 .
            if ctype == cointype {
                if bal == T::Balance::sa(0) {
                }else {
                    amount_on_sale = bal;
                }
            }
        });
        amount_on_sale
    }

    /// Insufficient token when withdrawing, it will cancel the sell order to get some free token
    pub fn withdraw_request(who:T::AccountId,withdraw_amount:u64,cointype:u64,sender:Vec<u8>) -> T::Balance {
       // println!("withdraw_amount {} cointype {}",withdraw_amount,cointype);
        // get the current amount of free token , if the data in uninited ,return zero.
        let free_token = Self::free_token_for_specific_coin(who.clone(),sender.clone(),cointype);
        // if free token is enough to withdraw , withdraw it.
        if free_token >= withdraw_amount {
            // free token is enough for the withdraw , do the lock/unlock operation
            match Self::lock_token(&who,sender.clone(),cointype,withdraw_amount,TokenType::WithDraw){
                Ok(()) =>   return T::Balance::sa(withdraw_amount),
                Err(_x) => return  T::Balance::sa(0),
            }
        }
        // if the free token is less than the requested amount
        // cancel all sell order and  turn The remaining money in sell orders into free token
        // calculate the total amount of specified cointype for who in sell orders
        let amount_on_sale = Self::selling_token_for_specific_coin(who.clone(),sender.clone(),cointype);
        if amount_on_sale == 0 {
            //if there is no sell order , directly do the lock/unlock operation
            // TODO::this situation isn't supposed  to happen when free_token is zero.
            // in another word, when into this case ,  the free_token must not be zero.
            // so the free token is reachable.
            match Self::lock_token(&who,sender.clone(),cointype,free_token,TokenType::WithDraw){
                Ok(()) =>  return T::Balance::sa(free_token),
                Err(_x) => return  T::Balance::sa(0),
            }
        }else {
            //Cancel all sell orders  , TODO:: Processing only part of sell orders
            <order::Module<T>>::cancel_order_for_bank_withdraw(who.clone(),cointype,sender.clone());
            // if success canceled , Then turn (amount_on_sale) from OTC_lock into free
            // in this case , if the free token is zero , the unlock will init the data
            // else , business as usual
            match Self::unlock_token(&who,sender.clone(),cointype,amount_on_sale,TokenType::OTC){
                Ok(()) => runtime_io::print("ok"),
                Err(_x) => return  T::Balance::sa(0),
            };
            // retrieve the updated free token  notice that [ new_free_amount = amount_on_sale + free_token ]
            let new_free_amount = Self::free_token_for_specific_coin(who.clone(),sender.clone(),cointype);
            // lock the The Optimum Range of Lockable token
            if new_free_amount >= withdraw_amount{
                // free token is sufficient,  Then turn (withdraw_amount) from free into Withdraw_lock
                match  Self::lock_token(&who,sender.clone(),cointype,withdraw_amount,TokenType::WithDraw){
                    Ok(()) =>  return T::Balance::sa(withdraw_amount),
                    Err(_x) => return T::Balance::sa(0),
                }
            }else {
                //TODO::this situation is supposed not to happen
                // free token is not sufficient,  Then turn all(new_free_amount) from free into Withdraw_lock
                match Self::lock_token(&who,sender.clone(),cointype,new_free_amount,TokenType::WithDraw){
                    Ok(()) => return T::Balance::sa(new_free_amount),
                    Err(_x) => return T::Balance::sa(0),
                }
            }
        }
    }
    /*------------new---------------*/
    fn reward_calcul(total_token:u64,) -> u64 {
        (total_token as f64 * 0.0001f64) as u64
    }

    /// with sender
    pub fn modify_token(who:T::AccountId,sender:Vec<u8>,coin_type:u64,amount:u64,token_type:TokenType,plus_or_minus:bool) -> Result {
        let mut current_token = match token_type {
            TokenType::Free => Self::deposit_free_token((who.clone(),sender.clone(),coin_type)),
            TokenType::OTC => Self::deposit_otc_token((who.clone(),sender.clone(),coin_type)),
            TokenType::WithDraw => Self::deposit_withdraw_token((who.clone(),sender.clone(),coin_type)),
            TokenType::Reward => Self::deposit_reward_token((who.clone(),sender.clone(),coin_type)),
        };
        if plus_or_minus {
            //current_token = current_token + amount;
            current_token = current_token.checked_add(amount)
                .ok_or_else(|| "account has too much funds")?;
        }else {
            if  current_token < amount{ return Err("insufficient token"); }
            current_token = current_token.checked_sub(amount)
                .ok_or_else(|| "account has too few funds")?;
        }
        match token_type {
            TokenType::Free => <DepositFreeToken<T>>::insert((who.clone(),sender.clone(),coin_type),current_token),
            TokenType::OTC => <DepositOtcToken<T>>::insert((who.clone(),sender.clone(),coin_type),current_token),
            TokenType::WithDraw => <DepositWithdrawToken<T>>::insert((who.clone(),sender.clone(),coin_type),current_token),
            TokenType::Reward => <DepositRewardToken<T>>::insert((who.clone(),sender.clone(),coin_type),current_token),
        };
        Self::modify_the_vec(who.clone(),coin_type,sender.clone());
        Ok(())
    }

    /// basic token revise function for Deposit
    pub fn deposit_to_free(who:T::AccountId,sender:Vec<u8>,coin_type:u64,amount:u64) -> Result {
        Self::modify_token(who.clone(),sender.clone(),coin_type,amount,TokenType::Free,true)?;
        Ok(())
    }
    /// basic token revise function for Deposit
    pub fn withdraw_from_withdraw(who:T::AccountId,sender:Vec<u8>,coin_type:u64,amount:u64) -> Result {
        Self::modify_token(who.clone(),sender.clone(),coin_type,amount,TokenType::WithDraw,false)?;
        Ok(())
    }

    ///  free -> otc/withdraw
    pub fn lock_token(who:&T::AccountId,sender:Vec<u8>,coin_type:u64,amount:u64,token_type:TokenType) -> Result{
        Self::modify_token(who.clone(),sender.clone(),coin_type,amount,TokenType::Free,false)?;
        Self::modify_token(who.clone(),sender.clone(),coin_type,amount,token_type,true)?;
        Ok(())
    }
    ///  otc/withdraw -> free
    pub fn unlock_token(who:&T::AccountId,sender:Vec<u8>,coin_type:u64,amount:u64,token_type:TokenType) -> Result{
        Self::modify_token(who.clone(),sender.clone(),coin_type,amount,token_type,false)?;
        Self::modify_token(who.clone(),sender.clone(),coin_type,amount,TokenType::Free,true)?;
        Ok(())
    }
    /// reward functions
    pub fn reward_token(who:&T::AccountId,sender:Vec<u8>,coin_type:u64,amount:u64) -> Result {
        Self::modify_token(who.clone(),sender.clone(),coin_type,amount,TokenType::Reward,true)?;
        Ok(())
    }

    pub fn take_out_reward_token(who:T::AccountId,sender:Vec<u8>,coin_type:u64) -> Result {
        let remove_amount = Self::deposit_reward_token((who.clone(),sender.clone(),coin_type));
        Self::modify_token(who.clone(),sender.clone(),coin_type,remove_amount,TokenType::Reward,false)?;
        Ok(())
    }

    pub fn iterator_all_token_for_who<F>(who:T::AccountId,mut func: F)
        where F: FnMut(u64,Vec<u8>) -> Result
    {
        let accountid_coin_vec = Self::deposit_account_coin_list(who.clone());
        accountid_coin_vec.iter().enumerate().for_each(|(_ib,&coin_type)|{
            let sender_vec = Self::deposit_sender_list((who.clone(),coin_type));
            sender_vec.iter().enumerate().for_each(|(_ic,sender)|{
                func(coin_type,sender.to_vec()).ok();;
            });
        });
    }

    pub fn iterator_all_token<F>(mut func: F)
        where F: FnMut(&T::AccountId,u64,Vec<u8>) -> Result
    {
        let all_deposit_account = Self::deposit_ladder_account_list();
        all_deposit_account.iter().enumerate().for_each(|(_ia,accountid)|{
            let accountid_coin_vec = Self::deposit_account_coin_list(accountid);
            accountid_coin_vec.iter().enumerate().for_each(|(_ib,&coin_type)|{
                let sender_vec = Self::deposit_sender_list((accountid.clone(),coin_type));
                sender_vec.iter().enumerate().for_each(|(_ic,sender)|{
                    func(accountid,coin_type,sender.to_vec()).ok();
                });
            });
        });
    }

    pub fn free_token_for_specific_coin(who:T::AccountId,sender:Vec<u8>,coin_type:u64) -> u64{
        Self::deposit_free_token((who.clone(),sender.clone(),coin_type))
    }

    pub fn selling_token_for_specific_coin(who:T::AccountId,sender:Vec<u8>,coin_type:u64) -> u64{
        Self::deposit_otc_token((who.clone(),sender.clone(),coin_type))
    }

    pub fn ptotential_withdrawable_coin(who:T::AccountId,sender:Vec<u8>,coin_type:u64) -> u64 {
        Self::deposit_free_token((who.clone(),sender.clone(),coin_type))
            + Self::deposit_otc_token((who.clone(),sender.clone(),coin_type))
    }

    pub fn total_token_for_specific_coin(who:&T::AccountId,sender:&Vec<u8>,coin_type:u64) -> u64{
        Self::deposit_free_token((who.clone(),sender.clone(),coin_type))
            + Self::deposit_otc_token((who.clone(),sender.clone(),coin_type))
            + Self::deposit_withdraw_token((who.clone(),sender.clone(),coin_type))
    }
    pub fn calculate_reward_and_reward(){
        Self::iterator_all_token(|accountid,coin_type,sender|{
            let total_token_for_this_coin = Self::total_token_for_specific_coin(&accountid,&sender,coin_type);
            let reward = Self::reward_calcul(total_token_for_this_coin);
            Self::reward_token(&accountid,sender.clone(),coin_type,reward)?;
            <RewardRecord<T>>::mutate(accountid,|bal| *bal = *bal+ T::Balance::sa(reward) );
            Self::calculate_total_reward(coin_type,T::Balance::sa(reward),true);
            Ok(())
        });
    }

    /// Modify the 3 auxiliary vectors of the 4 Main Storage maps
    pub fn modify_the_vec(who:T::AccountId,coin_type:u64,sender:Vec<u8>){
        // first, deposit some token , it wont do the delete , there must be some token in Main Storage maps
        if Self::total_token_for_specific_coin(&who,&sender,coin_type) == 0 {
            // when some token was fully deleted , need to delete the relate content in the support vector .
            // first,  delete the outside address
            let mut sender_vec = Self::deposit_sender_list((who.clone(),coin_type));
            if sender_vec.iter().find(|&t| t == &sender).is_none(){ /* not happen */}else{
                let mut  mark = 0usize;
                sender_vec.iter().enumerate().for_each(|(i,v)|{
                    if sender == *v{
                        mark = i;
                    }
                });
                sender_vec.remove(mark);
                <DepositSenderList<T>>::insert((who.clone(),coin_type),sender_vec);
            }
            // then, check the same owner and  the same coin type  with different senders are zero
            let sender_vec_2 = Self::deposit_sender_list((who.clone(),coin_type));
                 if sender_vec_2.is_empty(){
                // delete the coin_type
                let mut coin_type_vec = Self::deposit_account_coin_list(who.clone());
                if coin_type_vec.iter().find(|&t| t == &coin_type).is_none(){ /* not happen */ }else{
                let mut mark = 0usize;
                coin_type_vec.iter().enumerate().for_each(|(i,&v)|{
                    if v == coin_type{ mark = i ;}
                });
                coin_type_vec.remove(mark);
                <DepositAccountCoinList<T>>::insert(&who,coin_type_vec);
                }
            }else{ return; }
            // if all coin type is delete , the accout will be removed.
            let who_coin_type_vec = Self::deposit_account_coin_list(&who);
            if who_coin_type_vec.is_empty(){
                // delete the accountid
                let mut accountid_vec = Self::deposit_ladder_account_list();
                if accountid_vec.iter().find(|&t| t == &who).is_none(){ /* not happen */ }else{
                let mut mark = 0usize;
                accountid_vec.iter().enumerate().for_each(|(i,v)|{
                    if *v == who.clone(){  mark = i ;}
                });
                accountid_vec.remove(mark);
                <DepositLadderAccountList<T>>::put(accountid_vec);
                }
            }else{ return;}
            return;
        }
        // then, put the deposit data to the Vectors,if they already in the vectors ,skip to next process
        let mut deposit_account_vec = Self::deposit_ladder_account_list();
        if deposit_account_vec.iter().find(|&t| t == &who).is_none() {
            // dont have it , add the accountid
            deposit_account_vec.push(who.clone());
            <DepositLadderAccountList<T>>::put(deposit_account_vec);
        }else {
            //have it , do nothing ,skip
        }
        // then , put the coin type of the token to the second support vector, if it already exist ,skip
        let mut coin_type_vec = Self::deposit_account_coin_list(who.clone());
        if coin_type_vec.iter().find(|&t| t == &coin_type).is_none(){
            // none , add it
            coin_type_vec.push(coin_type);
            <DepositAccountCoinList<T>>::insert(&who,coin_type_vec);
        }else{
            // do nothing ,skip
        }
        // fially, put the outside address into the vector , if it already exist ,skip
        let mut sender_vec = Self::deposit_sender_list((who.clone(),coin_type));
        if sender_vec.iter().find(|&t| t == &sender).is_none(){
            // none , add it
            sender_vec.push(sender);
            <DepositSenderList<T>>::insert((who.clone(),coin_type),sender_vec);
        }else{
            // do nothing
        }
        // some amount of a new token was put into the map  and  the corresponding support vector is modified.
    }
    /*------------new---------------*/

    // send who the amount of balance
    pub fn deposit_reward(who:&T::AccountId , amount:u64){
        let new_balance = <BalanceOf<T> as As<u64>>::sa(amount);
        T::Currency::deposit_creating(who, new_balance);
    }

    /// return one's deposit amount
    pub fn deposit_amount_statics(){

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
    #[cfg(feature = "std")]
    use rustc_hex::*;
    use runtime_io::*;

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

    impl order::Trait for Test {
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

        });
    }

    #[test]
    fn deposit_test() {
        with_externalities(&mut new_test_ext(), || {
            println!("Test");
            Bank::depositing_withdraw_record(1,50,2,true);
            Bank::depositing_withdraw_record(1,50,3,true);

            Bank::depositing_withdraw_record(1,50,3,true);
            assert_eq!(Bank::despositing_account(),vec![1]);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (0, 1), (50, 2), (100, 3), (0, 4)].to_vec());
            Bank::depositing_withdraw_record(1,55,3,true);
            assert_eq!(Bank::despositing_banance(1),[(0, 0), (0, 1), (50, 2), (155, 3), (0, 4)].to_vec());
        });
    }

    #[test]
    fn depositing_test() {
        with_externalities(&mut new_test_ext(), || {
/*
            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::despositing_account(),[11744161374129632607].to_vec());
            assert_eq!(Bank::despositing_banance(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (0, 3), (0, 4)].to_vec());

            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_err!(Bank::deposit(Origin::signed(5),data,sign),"This signature is repeat!");
            assert_eq!(Bank::despositing_banance(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (0, 3), (0, 4)].to_vec());

            let mut data : Vec<u8>= "0000000000000002f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad00000000000000000000000000000000000000000000000000000000000000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "99ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf619".from_hex().unwrap();
            assert_err!(Bank::deposit(Origin::signed(5),data,sign),"cant deposit zero");
          //  assert_eq!(Bank::despositing_banance(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (1000, 3), (0, 4)].to_vec());

            let mut data : Vec<u8>= "0000000000000003f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea4".from_hex().unwrap();
            let sign : Vec<u8>= "12ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Origin::signed(5),data,sign));
            assert_eq!(Bank::despositing_banance(11744161374129632607),[(0, 0), (10000000, 1), (0, 2), (10000000, 3), (0, 4)].to_vec());
*/
        });
    }

    #[test]
    fn withdraw_test() {
        with_externalities(&mut new_test_ext(), || {
/*
            let mut data : Vec<u8>= "000000000000000000000000000000000000000000000000000000000000000100000000000000000000000074241db5f3ebaeecf9506e4ae988186093341604000000000000000000000000000000000000000000000000002386f26fc10000aba050dcf46dd539049458d8c25b29433b4ce2f194191d4e438c49e2db3a9be2".from_hex().unwrap();
            let sign : Vec<u8>= "c19901eae9150cea37f443c82319e1b656616f59fd0e810cdc9ea0667d81988b59b3292a8fc6100702c7d9a1e700a9f204156e32e44219af992933a3fbd6ff6701".from_hex().unwrap();
            assert_ok!(Bank::deposit(Origin::signed(5),data,sign));
            // assert_eq!(Bank::despositing_account(),[5].to_vec());
            assert_eq!(Bank::despositing_banance(0),[(0, 0), (1000, 1), (0, 2), (0, 3), (0, 4)].to_vec());


            let mut data : Vec<u8>= "000000000000000000000000000000000000000000000000000000000000000100000000000000000000000074241db5f3ebaeecf9506e4ae988186093341604000000000000000000000000000000000000000000000000002386f26fc10000aba050dcf46dd539049458d8c25b29433b4ce2f194191d4e438c49e2db3a9be3".from_hex().unwrap();
            let sign : Vec<u8>= "219901eae9150cea37f443c82319e1b656616f59fd0e810cdc9ea0667d81988b59b3292a8fc6100702c7d9a1e700a9f204156e32e44219af992933a3fbd6ff6701".from_hex().unwrap();
            assert_ok!(Bank::withdraw(Origin::signed(5),data,sign));
            assert_eq!(Bank::despositing_banance(0),[].to_vec());
            */
        });
    }

    #[test]
    fn withdraw_request_test() {
        with_externalities(&mut new_test_ext(), || {
            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::deposit_free_token((11744161374129632607,[247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec(),1))
                       ,100000000000);

            let mut data2 : Vec<u8>= "00000000000000010000000000000000000000000000000000000000000000000000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea4".from_hex().unwrap();
            let sign2 : Vec<u8>= "b36bba3f9e7138e45b9ff9918a0759623ca146b3956174efaadb37635c2adb440f9fd75e7773803337d4802d94f5c78788121dccd4b698080b047171966483711b".from_hex().unwrap();
            assert_ok!(Bank::request(Origin::signed(5),data2,sign2));
            assert_eq!(Bank::deposit_free_token((11744161374129632607,[247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec(),1))
                       ,0);
            assert_eq!(Bank::deposit_withdraw_token((11744161374129632607,[247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec(),1))
                       ,100000000000);
        });
    }

    #[test]
    fn new_test() {
        with_externalities(&mut new_test_ext(), || {
            let sender = [1,2].to_vec();
            let sender2 = [3,4].to_vec();
            //deposit
            Bank::deposit_to_free(1,sender.clone(),2,1000000000);
            assert_eq!(Bank::deposit_free_token((1,sender.clone(),2)),1000000000);

            // lock to otc
            assert_ok!(Bank::lock_token(&1,sender.clone(),2,1000,TokenType::OTC));
            assert_eq!(Bank::deposit_free_token((1,sender.clone(),2)),999999000);
            assert_eq!(Bank::deposit_otc_token((1,sender.clone(),2)),1000);

            //iter test
            Bank::calculate_reward_and_reward();
            assert_eq!(Bank::deposit_reward_token((1,sender.clone(),2)),100000);

            // withdraw fail test
            //assert_err!(Bank::withdraw_from_free(1,sender.clone(),2,1000000000),"insufficient token");
            assert_eq!(Bank::deposit_free_token((1,sender.clone(),2)),999999000);

            // unlock to free
            assert_ok!(Bank::unlock_token(&1,sender.clone(),2,1000,TokenType::OTC));
            assert_eq!(Bank::deposit_free_token((1,sender.clone(),2)),1000000000);
            assert_eq!(Bank::deposit_otc_token((1,sender.clone(),2)),0);

            assert_eq!(Bank::deposit_ladder_account_list(),[1].to_vec());
            assert_eq!(Bank::deposit_account_coin_list(1),[2].to_vec());
            println!("deposit_account_coin_list{:?}",Bank::deposit_account_coin_list(1));
            //assert_eq!(Bank::deposit_sender_list((1,2)),[].to_vec());

            //iter test
            Bank::calculate_reward_and_reward();
            assert_eq!(Bank::deposit_reward_token((1,sender.clone(),2)),200000);

            //withdraw request
            println!("request {}",Bank::withdraw_request(1,500000,2,sender.clone()));
            assert_eq!(Bank::deposit_free_token((1,sender.clone(),2)),999500000);
            assert_eq!(Bank::deposit_withdraw_token((1,sender.clone(),2)),500000);

            //iter test
            Bank::calculate_reward_and_reward();  //300000
            assert_eq!(Bank::reward_record(1),300000);
            Bank::calculate_reward_and_reward();
            assert_eq!(Bank::deposit_reward_token((1,sender.clone(),2)),400000);
            assert_eq!(Bank::reward_record(1),400000);

            // draw reward
            assert_ok!(Bank::draw_reward(Some(1).into()));
            assert_eq!(Bank::deposit_reward_token((1,sender.clone(),2)),0);
            assert_eq!(Bank::reward_record(1),0);

            //deposit twice
            Bank::deposit_to_free(1,sender.clone(),3,1000000000);
            Bank::deposit_to_free(1,sender2.clone(),1,1000000000);
            assert_eq!(Bank::deposit_free_token((1,sender.clone(),3)),1000000000);
            assert_eq!(Bank::deposit_free_token((1,sender2.clone(),1)),1000000000);

            assert_eq!(Bank::reward_record(1),0);
            //iter test2
            Bank::calculate_reward_and_reward();  //300000
            assert_eq!(Bank::reward_record(1),300000);

            assert_eq!(Bank::coin_reward(1),100000);
            assert_eq!(Bank::coin_reward(2),500000);
            assert_eq!(Bank::coin_reward(3),100000);
            assert_eq!(Bank::coin_reward(4),0);
            // draw reward
            assert_ok!(Bank::draw_reward(Some(1).into()));
            assert_eq!(Bank::deposit_reward_token((1,sender.clone(),3)),0);
            assert_eq!(Bank::deposit_reward_token((1,sender2.clone(),1)),0);
            assert_eq!(Bank::reward_record(1),0);
        });
    }

    #[test]
    fn new_depositing_test() {
        with_externalities(&mut new_test_ext(), || {

            let mut data : Vec<u8>= "0000000000000001f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea3".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::deposit_ladder_account_list(),[11744161374129632607].to_vec());
            assert_eq!(Bank::deposit_account_coin_list(11744161374129632607),[1].to_vec());
            println!(" {:?}", Bank::deposit_sender_list((11744161374129632607,1)));
            assert_eq!(Bank::deposit_sender_list((11744161374129632607,1)),[[247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec()].to_vec());
            assert_eq!(Bank::deposit_free_token((11744161374129632607,[247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec(),1)),100000000000);

            let mut data : Vec<u8>= "0000000000000003f758e53313Fa9264E1E23bF0Bd9b14A7E98C82745f35dce98ba4fba25530a026ed80b2cecdaa31091ba4958b99b52ea1d068adad0000000000000000000000000000000000000000000000056bc75e2d631000001bc8676204852133d9b70bfef9ac4bedec87e281458ae052a76139a28fa8cea4".from_hex().unwrap();
            let sign : Vec<u8>= "11ee83fc6db16b233d763fc71efe8f0b8db95df8403a2a87b34f51cb3d7b4e136cf66a4ef0f685b3b7ac74644577154899e55cb398cd538bc615cc5e0ab6acf61c".from_hex().unwrap();
            assert_ok!(Bank::deposit(Some(1).into(),data,sign));
            assert_eq!(Bank::deposit_ladder_account_list(),[11744161374129632607].to_vec());
            assert_eq!(Bank::deposit_account_coin_list(11744161374129632607),[1,3].to_vec());
            println!(" {:?}", Bank::deposit_sender_list((11744161374129632607,3)));
            assert_eq!(Bank::deposit_sender_list((11744161374129632607,3)),[[247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec()].to_vec());
            assert_eq!(Bank::deposit_free_token((11744161374129632607,[247, 88, 229, 51, 19, 250, 146, 100, 225, 226, 59, 240, 189, 155, 20, 167, 233, 140, 130, 116].to_vec(),1)),100000000000);
        });
    }

}