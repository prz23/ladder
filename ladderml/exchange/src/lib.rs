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
        <T as system::Trait>::AccountId
    {
        SetMinRequreSignatures(u64),
        TranscationVerified(u64,Vec<(AccountId,u64)>),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Exchange {
        /// Record the number of signatures per transaction got
        NumberOfSignedContract get(num_of_signed): map (u64,u64) => u64;
        // Latest Time Record of other assets' shared latest timestamp!
        LatestTime get(latest_time):  u64 ;
        LatestExchangeRate get(latest_exrate): map u64 => u64;
        /// These amount of signatures are needed to send the event that the transaction verified.
        MinNumOfSignature get(min_signature): u64;
        //record transaction   Hash => (accountid,sign)
        IdSignTxList  get(all_list) : map u64 => (T::AccountId,u64);
        IdSignTxListB  get(all_list_b) : map u64 => Vec<(T::AccountId,u64)>;
        RepeatPrevent  get(repeat) : map (u64,u64) => Vec<T::AccountId>;

        /// Transaction records that have been sent prevent duplication of events
        AlreadySentTx get(already_sent) : map (u64,u64) => u64;
        /// History Exchange Data. All Coin share the same time stamp.  timestamp => [(type,rate),(type,rate),....]
        HistoricalData get(historial_data) : map u64 => Vec<(u64,u64)>;
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

        pub  fn check_exchange(origin, message: Vec<u8>, signature: Vec<u8>) -> Result{
            let sender = ensure_signed(origin)?;
            let (exchangerate, time, extype) = Self::parse_tx_data(message);

            match Self::check_signature(sender,exchangerate,time,extype) {
                Ok(()) => return Ok(()),
                Err(x) => return Err(x),
            }
        }
    }
}

impl<T: Trait> Module<T> {
    fn _verify(_tx: T::Hash) -> Result {
        //TODO:verify signature or others
        Ok(())
    }

    pub fn check_validator(accountid: &T::AccountId) -> bool {
        return false;
    }

    /// Sign and determine if the current number of signatures is sufficient to send an event
    pub fn check_signature(who: T::AccountId, exchangerate: u64, time: u64, extype: u64) -> Result {
        let sender = who;

        if !<NumberOfSignedContract<T>>::exists((time,extype)) {
            <NumberOfSignedContract<T>>::insert(&(time,extype), 0);
            <AlreadySentTx<T>>::insert(&(time,extype), 0);
        }

        let mut repeat_vec = Self::repeat((time,extype));
        ensure!(!repeat_vec.contains(&sender), "repeat!");

        if 1 == Self::already_sent((time,extype)) {
            return Err("has been sent");
        }

        <IdSignTxList<T>>::insert(time.clone(), (sender.clone(), exchangerate.clone()));

        let mut stored_vec = Self::all_list_b(time);
        stored_vec.push((sender.clone(), exchangerate.clone()));
        <IdSignTxListB<T>>::insert(time.clone(), stored_vec.clone());
        repeat_vec.push(sender.clone());
        <RepeatPrevent<T>>::insert((time,extype), repeat_vec.clone());

        let numofsigned = Self::num_of_signed(&(time,extype));
        let newnumofsigned = numofsigned
            .checked_add(1)
            .ok_or("Overflow adding a new sign to Tx")?;
        <NumberOfSignedContract<T>>::insert(&(time,extype), newnumofsigned);
        if newnumofsigned <= Self::min_signature() {
            return Err("not enough signatusign_and_checkre");
        }

        Self::save_lastexchange_data(time,exchangerate,extype);
        // Record the transaction and sending event
        <AlreadySentTx<T>>::insert(&(time,extype), 1);
        Self::deposit_event(RawEvent::TranscationVerified(time, stored_vec));
        Ok(())
    }

    fn parse_data(message: Vec<u8>, signature: Vec<u8>) -> (T::AccountId, u64, u64, u64) {
        let mut messagedrain = message.clone();
        let hash: Vec<_> = messagedrain.drain(0..32).collect();

        let tx_hash: T::Hash = Decode::decode(&mut &hash[..]).unwrap();

        let signature_hash: T::Hash = Decode::decode(&mut &signature[..]).unwrap();

        let time: Vec<_> = messagedrain.drain(0..8).collect();
        let who: T::AccountId = Decode::decode(&mut &time[..]).unwrap();

        let rate_vec: Vec<u8> = messagedrain.drain(32..40).collect();
        let mut rate: u64 = 0;
        let mut i = 0;
        rate_vec.iter().for_each(|x| {
            let exp = (*x as u64) ^ i;
            rate = rate + exp;
            i = i + 1;
        });

        let time_vec: Vec<u8> = messagedrain.drain(32..40).collect();
        let mut time: u64 = 0;
        let mut v = 0;
        time_vec.iter().for_each(|x| {
            let exp = (*x as u64) ^ v;
            time = time + exp;
            v = v + 1;
        });

        let type_vec: Vec<u8> = messagedrain.drain(32..40).collect();
        let mut extype: u64 = 0;
        let mut q = 0;
        type_vec.iter().for_each(|x| {
            let exp = (*x as u64) ^ q;
            extype = extype + exp;
            q = q + 1;
        });
        //ensure the signature is valid
        let mut tx_hash_to_check: [u8; 65] = [0; 65];
        tx_hash_to_check.clone_from_slice(&hash);
        let mut signature_hash_to_check: [u8; 32] = [0; 32];
        signature_hash_to_check.clone_from_slice(&signature);
        //<bank::Module<T>>::check_secp512(&tx_hash_to_check, &signature_hash_to_check).is_ok();

        return (who, rate, time, extype);
    }

    /// parse_tx_data
    fn parse_tx_data(message: Vec<u8>) -> (u64, u64, u64) {
        let mut messagedrain = message.clone();

        // exchange rate
        let mut rate_vec: Vec<u8> = messagedrain.drain(0..8).collect();
        rate_vec.reverse();
        let mut rate_u64 = Self::u8array_to_u64(rate_vec.as_slice());


        // time of the exchangerate
        let mut time_vec: Vec<_> = messagedrain.drain(0..8).collect();
        time_vec.reverse();
        let mut time_u64 = Self::u8array_to_u64(time_vec.as_slice());

        // pair type of the rate
        let mut pair_vec: Vec<_> = messagedrain.drain(0..8).collect();
        pair_vec.reverse();
        let mut pair_u64 = Self::u8array_to_u64(pair_vec.as_slice());

        return (rate_u64, time_u64, pair_u64);
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

    fn save_lastexchange_data(timestamp:u64, rate:u64, exchangetype:u64) {
        <LatestTime<T>>::put(timestamp);
        <LatestExchangeRate<T>>::insert(exchangetype,rate);

        //Save history record
        let mut vec = Self::historial_data(&timestamp);
        vec.push((exchangetype,rate));
        <HistoricalData<T>>::insert(timestamp,vec);
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


    impl Trait for Test {
        type Event = ();
    }

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        runtime_io::TestExternalities::new(t)
    }

    type Exchange = Module<Test>;

    #[test]
    fn resolving_data_test() {
        with_externalities(&mut new_test_ext(), || {
            let (a,b,c) = Exchange::parse_tx_data([0,0,0,0,0,0,6,5,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,9].to_vec());
            assert_eq!(a,1541);
            assert_eq!(b,7);
            assert_eq!(c,9);
        });
    }

    #[test]
    fn save_data_test() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(Exchange::check_exchange(Origin::signed(6),[0,0,0,0,0,0,6,5,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,1].to_vec(),[1].to_vec()));
            assert_eq!(Exchange::already_sent((7,1)),1);

            assert_eq!(Exchange::already_sent((8,1)),0);
            assert_ok!(Exchange::check_exchange(Origin::signed(6),[0,0,0,0,0,0,6,5,0,0,0,0,0,0,0,8,0,0,0,0,0,0,0,1].to_vec(),[1].to_vec()));
            assert_eq!(Exchange::already_sent((8,1)),1);
            //test if the exchangerate pass
            assert_eq!(Exchange::already_sent((8,2)),0);
            assert_ok!(Exchange::check_exchange(Origin::signed(6),[0,0,0,0,0,0,6,5,0,0,0,0,0,0,0,8,0,0,0,0,0,0,0,2].to_vec(),[1].to_vec()));
            assert_eq!(Exchange::already_sent((8,2)),1);

            //test if the exchangerate change by time
            assert_eq!(Exchange::get_latest_exchangerate(2),(8,1541));
            assert_ok!(Exchange::check_exchange(Origin::signed(6),[0,0,0,0,0,0,7,5,0,0,0,0,0,0,0,9,0,0,0,0,0,0,0,2].to_vec(),[1].to_vec()));
            assert_eq!(Exchange::get_latest_exchangerate(2),(9,1797));
        });
    }

}