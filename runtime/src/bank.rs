extern crate srml_session as session;
extern crate srml_balances as balances;
extern crate sr_io as runtime_io;
extern crate substrate_primitives as primitives;

use codec::{Decode, Encode};


use session::*;
use rstd::prelude::Vec;
use runtime_primitives::traits::*;
use srml_support::{StorageValue, StorageMap, dispatch::Result};
use system::{self, ensure_signed};
use sigcount;


pub trait Trait: balances::Trait + session::Trait + sigcount::Trait{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// deposit
        //offset 0:32  who         ::   AccountID
        //offset 32:48 bytes        ::   money
        //origin, hash 32: T::Hash, tag 32: T::Hash, id 20: T::AccountId, amount 32: T::Balance, signature: Vec<u8>
        //(origin, message: Vec, signature: Vec)
        pub fn deposit(origin, message: Vec<u8>, signature: Vec<u8>) -> Result {
            let _sender = ensure_signed(origin)?;
            // 解析message --> hash  tag  id  amount
            let (tx_hash,who,amount,signature_hash) = Self::split_message(message,signature);
            //check the validity and number of signatures
            if Self::check_signature(who.clone(),tx_hash,signature_hash).is_ok(){
                runtime_io::print("deposit -> account balance");
            } else{
                return Err("not enough signature or bad signature");
            }

            // ensure no repeat desposit
            ensure!(Self::despositing_account().iter().find(|&t| t == &who).is_none(), "Cannot deposit if already depositing.");
            // ensure no repeat intentions to desposit
            ensure!(Self::intentions_desposit_vec().iter().find(|&t| t == &who).is_none(), "Cannot deposit if already in queue.");
            // update the list of intentions to desposit
            <IntentionsDespositVec<T>>::put({
                let mut v =  Self::intentions_desposit_vec();
                v.push(who.clone());
                v
            });
            <IntentionsDesposit<T>>::insert(who.clone(),T::Balance::sa(amount as u64));
            Self::deposit_event(RawEvent::AddDepositingQueue(who));
            Ok(())
        }

        pub fn deposit2  (origin, _hash: T::Hash, _tag: T::Hash, id: T::AccountId,amount: T::Balance, _signature: Vec<u8>) -> Result {
            let _sender = ensure_signed(origin)?;
            let who =  id;
            // ensure no repeat
            ensure!(Self::despositing_account().iter().find(|&t| t == &who).is_none(), "Cannot deposit if already depositing.");
            // ensure no repeat
            ensure!(Self::intentions_desposit_vec().iter().find(|&t| t == &who).is_none(), "Cannot deposit if already in queue.");


            // update the list of intentions to desposit
            <IntentionsDespositVec<T>>::put({
                let mut v =  Self::intentions_desposit_vec();
                v.push(who.clone());
                v
            });

            <IntentionsDesposit<T>>::insert(who.clone(),amount);
            Self::deposit_event(RawEvent::AddDepositingQueue(who));
            Ok(())
        }


        /// withdraw
        /// origin, message: Vec<u8>, signature: Vec<u8>
         pub fn withdraw(_origin, message: Vec<u8>, signature: Vec<u8>) -> Result {

            // 解析message --> hash  tag  id  amount
            let (tx_hash,who,_amount,signature_hash) = Self::split_message(message,signature);
            //check the validity and number of signatures
            if Self::check_signature(who.clone(),tx_hash,signature_hash).is_ok(){
                runtime_io::print("deposit -> account balance");
            } else{
                return Err("not enough signature or bad signature");
            }

            // ensure no repeat
            ensure!(!Self::despositing_account().iter().find(|&t| t == &who).is_none(), "Cannot deposit if not depositing.");
            ensure!(Self::intentions_withdraw().iter().find(|&t| t == &who).is_none(), "Cannot withdraw2 if already in withdraw2 queue.");
            runtime_io::print("============withdraw2===========");
            <IntentionsWithdraw<T>>::put({
                let mut v =  Self::intentions_withdraw();
                v.push(who.clone());
                v
            });
            Ok(())
        }

        pub fn withdraw2(_origin, _hash: T::Hash, _tag: T::Hash, id: T::AccountId,_amount: T::Balance, _signature: Vec<u8>) -> Result {
            // ensure no repeat
            ensure!(!Self::despositing_account().iter().find(|&t| t == &id).is_none(), "Cannot deposit if not depositing.");
            ensure!(Self::intentions_withdraw().iter().find(|&t| t == &id).is_none(), "Cannot withdraw2 if already in withdraw2 queue.");
            runtime_io::print("============withdraw2===========");
            <IntentionsWithdraw<T>>::put({
                let mut v =  Self::intentions_withdraw();
                v.push(id.clone());
                v
            });
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

        /// a new session starts
		fn on_finalise(n: T::BlockNumber) {
		    Self::check_rotate_session(n);
		}
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Matrix {
        /// bank & session
        /// record deposit info       AccountId -> message & signature
        DepositInfo get(deposit_info) : map  T::AccountId => (Vec<u8>,Vec<u8>);
        /// record depositing info of balance & session_time
        DespoitingAccount get(despositing_account): Vec<T::AccountId>;
        DespositingBalance get(despositing_banance): map T::AccountId => T::Balance;
        DespositingTime get(despositing_time): map T::AccountId => u32;

        /// All the accounts with a desire to deposit
        IntentionsDespositVec  get(intentions_desposit_vec) config():  Vec<T::AccountId>;
        IntentionsDesposit  get(intentions_desposit): map T::AccountId => T::Balance;
       	/// The block at which the `who`'s funds become entirely liquid.
		pub DepositBondage get(deposit_bondage): map T::AccountId => T::BlockNumber;
        /// All the accounts with a desire to withdraw
        IntentionsWithdraw  get(intentions_withdraw): Vec<T::AccountId>;

        /// Bank session reward factor
        RewardSessionValue  get(reward_session_value): Vec<u32>;
        RewardSessionFactor  get(reward_session_factor): Vec<u8>;
        /// Bank balance reward factor
        RewardBalanceValue  get(reward_balance_value): Vec<T::Balance>;
        RewardBalanceFactor  get(reward_balance_factor): Vec<u8>;

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
        /// a new seesion start
        NewRewardSession(BlockNumber),

    }
}

impl<T: Trait> Module<T>
{
    fn  split_message( message: Vec<u8>, signature: Vec<u8>) -> (T::Hash,T::AccountId,u64,T::Hash) {
        // 解析message --> hash  tag  id  amount
        let mut messagedrain = message.clone();

        let hash:Vec<_> = messagedrain.drain(0..32).collect();
        let tx_hash: T::Hash = Decode::decode(&mut &hash.encode()[..]).unwrap();

        let signature_hash =  Decode::decode(&mut &signature.encode()[..]).unwrap();

        let id:Vec<_> = messagedrain.drain(32..64).collect();
        let who: T::AccountId = Decode::decode(&mut &id.encode()[..]).unwrap();
        let amount_vec:Vec<u8> = messagedrain.drain(32..40).collect();
        let mut amount: u64 = 0;
        let mut i = 0;
        amount_vec.iter().for_each(|x|{
            let exp = (*x as u64)^i;
            amount = amount+exp;
            i = i+1;
        });

        //ensure the signature is valid
        //ensure!(Self::check_secp512(&hash[..],&signature[..]),"not valid signature");
       
        return (tx_hash,who,amount,signature_hash);
    }

    /// Hook to be called after transaction processing.  间隔一段时间才触发 rotate_session
    pub fn check_rotate_session(block_number: T::BlockNumber) {
        // do this last, after the staking system has had chance to switch out the authorities for the
        // new set.
        // check block number and call next_session if necessary.
        let is_final_block = ((block_number - Self::last_length_change()) % Self::length()).is_zero();
        let (should_end_session, apply_rewards) = <ForcingNewSession<T>>::take()
            .map_or((is_final_block, is_final_block), |apply_rewards| (true, apply_rewards));
        if should_end_session {
            runtime_io::print("--------安排上了04-01--------");
            Self::rotate_session(is_final_block, apply_rewards);
        }
    }

    /// The last length change, if there was one, zero if not.  查看lenth间隔长度
    pub fn last_length_change() -> T::BlockNumber {
        <LastLengthChange<T>>::get().unwrap_or_else(T::BlockNumber::zero)
    }

    /// Move onto next session: register the new authority set.
    /// 把新的 depositingqueue 加入 实际depositing列表   或者 把不存钱的老铁的名字从列表里删除
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
        Self::reward_deposit();
    }

    fn adjust_deposit_list(){
        //修改全部的表 deposit 部分   已经判断过重复了所以直接天加
        let mut int_des_vec =  Self::intentions_desposit_vec();
        let mut des_vec = Self::despositing_account();
        while let  Some(who)=int_des_vec.pop(){
            runtime_io::print("========add===========");
            //更新正在抵押人列表
            <DespositingBalance<T>>::insert(who.clone(), <IntentionsDesposit<T>>::get(who.clone()));
            <DespositingTime<T>>::insert(who.clone(), 0);
            des_vec.push(who.clone());
            <IntentionsDesposit<T>>::remove(who);
        }
        <DespoitingAccount<T>>::put(des_vec);
        <IntentionsDespositVec<T>>::put(int_des_vec);

        /////////////////////////////////////////////////////////a
        let mut des_vec2 = Self::despositing_account();
        let mut vec_with =  Self::intentions_withdraw();
        while let Some(who) = vec_with.pop() {
            runtime_io::print("========remove===========");
            //增加 despoit 记录  同时创建session记录
            <DespositingBalance<T>>::remove(who.clone());
            des_vec2.push(who.clone());
            <DespositingTime<T>>::remove(who.clone());
        }
        <IntentionsWithdraw<T>>::put(vec_with);
        <DespoitingAccount<T>>::put(des_vec2);
        
        runtime_io::print("对表进行session time更新");
        //对表进行session time更新
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            <DespositingTime<T>>::insert(v,Self::despositing_time(v)+1);
        });
    }


    fn reward_deposit() {
        //TODO  发钱
        //循环历遍一下所有deposit列表，分别对每个账户进行单独的发钱处理
        //首先判断session 决定一个 时间比率
        //再判断balance 决定一个 存款比率
        //两个比率结合起来决定一个 乘积因子Xbalance => 然后往账户发
        runtime_io::print("发钱发钱发钱发钱发钱发钱发钱发钱发钱发钱");
        Self::despositing_account().iter().enumerate().for_each(|(_i,v)|{
            let reward = Self::reward_set(v.clone(),<DespositingTime<T>>::get(v),<DespositingBalance<T>>::get(v));
            let _ = <balances::Module<T>>::reward(v, reward);
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

        <DespositingBalance<T>>::get(who)*T::Balance::sa((final_se * final_ba)as u64)
    }

    fn check_signature(who: T::AccountId, tx: T::Hash, signature: T::Hash) -> Result {
        //ensure enough signature
        <sigcount::Module<T>>::check_signature(who,tx,signature)
    }

    fn check_secp512(tx: &[u8; 65], signature: &[u8; 32]) -> bool {
        runtime_io::print("asd");
        //TODO: if runtime_io::secp256k1_ecdsa_recover(signature,tx).is_ok(){ true} else { false }
        //TODO:
        true
    }
}