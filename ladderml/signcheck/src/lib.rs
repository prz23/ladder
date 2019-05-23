#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

use sr_primitives::traits::{CheckedAdd, CheckedSub, Hash, Verify, Zero};
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

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
    {
        Created(AccountId, Hash),
        SetMinRequreSignatures(u64),
        Txisok(Hash),
        // 交易 = vec<id，签名>
        TranscationVerified(Hash,Vec<(AccountId,Hash)>),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Signature {

        /// 记录每个交易的签名的数量
        /// Record the number of signatures per transaction got
        NumberOfSignedContract get(num_of_signed): map T::Hash => u64;

        /// 需要这些数量的签名，才发送这个交易通过的事件
        /// These amount of signatures are needed to send the event that the transaction verified.
        MinNumOfSignature get(min_signature)  : u64 = 1;

        //record transaction   Hash => (accountid,sign)
        //IdSignTxList  get(all_list) : map T::Hash => (T::AccountId,T::Hash);
        Record  get(record) : map T::Hash => Vec<(T::AccountId,T::Hash)>;
       // IdSignTxListC  get(all_list_c) : map T::Hash => Vec<T::AccountId>;

        /// 是否有重复的签名   map 交易Hash => 签名Hash列表
        RepeatPrevent  get(repeat_prevent) : map T::Hash => Vec<T::Hash>;

        /// 已经发送过的交易记录  防止重复发送事件
        /// Transaction records that have been sent prevent duplication of events
        AlreadySentTx get(already_sent) : map T::Hash => u64;

        txsave get(tx_save) : Vec<T::Hash>;
        // Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// 设置最小要求签名数量
        /// Set the minimum required number of signatures
        pub  fn set_min_num(origin,new_num: u64) -> Result{
            let _sender = ensure_signed(origin)?;
            let newmin = new_num;
            if newmin < 5 {
                return Err("too small,should bigger than 5");
            }
            <MinNumOfSignature<T>>::put(newmin) ;
            Self::deposit_event(RawEvent::SetMinRequreSignatures(newmin));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn _verify(_tx : T::Hash) -> Result{
        //TODO:verify signature or others
        Ok(())
    }

    /// 签名并判断如果当前签名数量足够就发送一个事件
       /// Sign and determine if the current number of signatures is sufficient to send an event
    pub  fn check_signature(who: T::AccountId, transcation: T::Hash, sign: T::Hash, message: T::Hash) -> Result{
        //TODO： 判断这个信息发送的人是否是validator     不在这里 已经前置了
        let sender = who;
        runtime_io::print("111111");
        <txsave<T>>::put({
            let mut xx = Self::tx_save();
            xx.push(transcation.clone());
            xx  }
        );
        runtime_io::print("222222");
        //查看该交易是否已经存在，没得话添加上去
        if !<NumberOfSignedContract<T>>::exists(transcation) {
            <NumberOfSignedContract<T>>::insert(&transcation,0);
            <AlreadySentTx<T>>::insert(&transcation,0);
        }

        /// 防止签名重复
        // 防止一个交易的相同签名的是否重复发送
        let mut repeat_vec = Self::repeat_prevent(transcation);
        ensure!(!repeat_vec.contains(&sign),"This signature is repeat!");
        // ensure!(!repeat_vec.iter().find(|&t| t == &sign).is_none(), "Cannot deposit if already in queue.");
        // 把某个交易hash的一个签名hash保存，以验证后来的是否重复
        repeat_vec.push(sign.clone());
        <RepeatPrevent<T>>::insert(transcation.clone(),repeat_vec.clone());

        // 防止一个交易被重复发送，已发送过的会有记录
        if 1 == Self::already_sent(transcation){
            return Err("This Transcation already been sent!");
        }

        //增加一条记录  包含  交易hash => vec (验证者,签名hash)
        let mut stored_vec = Self::record(transcation);
        stored_vec.push((sender.clone(),sign.clone()));
        <Record<T>>::insert(transcation.clone(),stored_vec.clone());

        //TODO:其他验证
        Self::_verify(transcation)?;

        // 判断签名数量是否达到指定要求
        let numofsigned = Self::num_of_signed(&transcation);
        let newnumofsigned = numofsigned.checked_add(1)
        .ok_or("Overflow adding a new sign to Tx")?;

        <NumberOfSignedContract<T>>::insert(&transcation,newnumofsigned);
        if newnumofsigned < Self::min_signature() {
            return Err("Not enough signature!");
        }

        // 记录已发送的交易防止重复发送 Record the transaction and sending event
        <AlreadySentTx<T>>::insert(&transcation,1);

        // 抛出事件
        Self::deposit_event(RawEvent::Txisok(transcation));
        Self::deposit_event(RawEvent::TranscationVerified(transcation,stored_vec));
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

    impl Trait for Test {
        type Event = ();
    }

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }

    type Signcheck = Module<Test>;

    #[test]
    fn set_minum_signature() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Signcheck::min_signature(),1);
            assert_ok!(Signcheck::set_min_num(Origin::signed(2),5));
            assert_eq!(Signcheck::min_signature(),5);
            assert_ok!(Signcheck::set_min_num(Origin::signed(3),500));
            assert_eq!(Signcheck::min_signature(),500);

            assert_err!(Signcheck::set_min_num(Origin::signed(4),2),"too small,should bigger than 5");
            assert_eq!(Signcheck::min_signature(),500);
        });
    }

    #[test]
    fn check_single_signature() {
        with_externalities(&mut new_test_ext(), || {
            // when min_signature is 1  the first tx will success
            assert_ok!(Signcheck::check_signature(1,H256::from_low_u64_be(15),
            H256::from_low_u64_be(15),H256::from_low_u64_be(15)));
            // the second tx will err
            assert_err!(Signcheck::check_signature(2,H256::from_low_u64_be(15),
            H256::from_low_u64_be(14),H256::from_low_u64_be(15)),"This Transcation already been sent!");
        });
    }

    #[test]
    fn check_muilt_signature() {
        with_externalities(&mut new_test_ext(), || {
            // now set the min_sig to 5
            assert_ok!(Signcheck::set_min_num(Origin::signed(2),5));
            assert_eq!(Signcheck::min_signature(),5);

            // then only the 5th transcation will success
            assert_err!(Signcheck::check_signature(1,H256::from_low_u64_be(15),
            H256::from_low_u64_be(15),H256::from_low_u64_be(15)),"Not enough signature!");
            assert_err!(Signcheck::check_signature(2,H256::from_low_u64_be(15),
            H256::from_low_u64_be(14),H256::from_low_u64_be(15)),"Not enough signature!");
            assert_err!(Signcheck::check_signature(3,H256::from_low_u64_be(15),
            H256::from_low_u64_be(13),H256::from_low_u64_be(15)),"Not enough signature!");
            assert_err!(Signcheck::check_signature(4,H256::from_low_u64_be(15),
            H256::from_low_u64_be(12),H256::from_low_u64_be(15)),"Not enough signature!");
            assert_ok!(Signcheck::check_signature(5,H256::from_low_u64_be(15),
            H256::from_low_u64_be(11),H256::from_low_u64_be(15)));

            // already_sent == 1 means the Hash(15) has been accept
            assert_eq!(Signcheck::already_sent(H256::from_low_u64_be(15)),1);
        });
    }

}