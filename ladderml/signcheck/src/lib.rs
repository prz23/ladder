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
        /// minum number of signatures required
        SetMinRequreSignatures(u64),
        /// transcation = vec<verifier,singature>
        TranscationVerified(Hash,Vec<(AccountId,Hash)>),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Signature {

        /// Record the number of signatures per transaction got
        NumberOfSignedContract get(num_of_signed): map T::Hash => u64;

        /// These amount of signatures are needed to send the event that the transaction verified.
        MinNumOfSignature get(min_signature)  : u64 = 1;

        //record transaction   Hash => (accountid,sign)
        Record  get(record) : map T::Hash => Vec<(T::AccountId,T::Hash)>;

        /// Preventing duplicate signatures   map txHash => Vec<signatureHash>
        RepeatPrevent  get(repeat_prevent) : map T::Hash => Vec<T::Hash>;

        /// Transaction records that have been sent prevent duplication of events
        AlreadySentTx get(already_sent) : map T::Hash => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

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

    /// determine if the current number of signatures is sufficient to send an event
    pub  fn check_signature(who: T::AccountId, transcation: T::Hash, sign: T::Hash, message: T::Hash) -> Result{
        //TODO： 判断这个信息发送的人是否是validator     不在这里 已经前置了
        let sender = who;

        // Query whether a transaction already exists
        if !<NumberOfSignedContract<T>>::exists(transcation) {
            <NumberOfSignedContract<T>>::insert(&transcation,0);
            <AlreadySentTx<T>>::insert(&transcation,0);
        }

        /// Preventing duplication of same signature
        let mut repeat_vec = Self::repeat_prevent(transcation);
        ensure!(!repeat_vec.contains(&sign),"This signature is repeat!");
        // ensure!(!repeat_vec.iter().find(|&t| t == &sign).is_none(), "Cannot deposit if already in queue.");
        // Save one of the signatures of a transaction. Guarantee that subsequent signatures are not duplicated.
        repeat_vec.push(sign.clone());
        <RepeatPrevent<T>>::insert(transcation.clone(),repeat_vec.clone());

        // Preventing duplication of same signature. Record successful transactions
        if 1 == Self::already_sent(transcation){
            return Err("This Transcation already been sent!");
        }

        // Add a transcation record  include  tx-hash => vec <verifierId,signature-hash>
        let mut stored_vec = Self::record(transcation);
        stored_vec.push((sender.clone(),sign.clone()));
        <Record<T>>::insert(transcation.clone(),stored_vec.clone());

        // TODO: other verification
        Self::_verify(transcation)?;

        // Determine whether the number of signatures meets the specified requirements
        let numofsigned = Self::num_of_signed(&transcation);
        let newnumofsigned = numofsigned.checked_add(1)
        .ok_or("Overflow adding a new sign to Tx")?;

        <NumberOfSignedContract<T>>::insert(&transcation,newnumofsigned);
        if newnumofsigned < Self::min_signature() {
            return Err("Not enough signature!");
        }

        // Recording Transactions Sent
        <AlreadySentTx<T>>::insert(&transcation,1);

        // Emit an event include the transcation and all signatures
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

            assert_eq!(Signcheck::already_sent(H256::from_low_u64_be(15)),0);

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