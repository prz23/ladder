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

use runtime_io::*;

pub trait Trait: system::Trait + session::Trait{
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
        MessageRecord get(message_record) : map T::Hash => Vec<u8>;

        /// Preventing duplicate signatures   map txHash => Vec<signatureHash>
        RepeatPrevent  get(repeat_prevent) : map T::Hash => Vec<T::Hash>;

        /// Transaction records that have been sent prevent duplication of events
        AlreadySentTx get(already_sent) : map T::Hash => u64;

        /// secp512  ladder accountid => pubkey
        LadderPubkey get(ladder_pubkey): map T::AccountId => Vec<u8>;
    }

    add_extra_genesis {
        config(pubkey) : (T::AccountId,Vec<u8>);
        build(|storage: &mut sr_primitives::StorageOverlay, _: &mut sr_primitives::ChildrenStorageOverlay, config: &GenesisConfig<T>| {
            with_storage(storage, || {
                let (a,b) = config.pubkey.clone();
                <Module<T>>::inilize_secp256(a,b);
            })
        })
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

        pub fn bond_pubkey(origin, pubkey: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            if <LadderPubkey<T>>::exists(&sender) {
				return Err("Pubkey already bonded")
			}
			<LadderPubkey<T>>::insert(sender.clone(), pubkey);
            Ok(())
        }

        pub fn unbond_pubkey(origin) -> Result {
            let sender = ensure_signed(origin)?;
            if !<LadderPubkey<T>>::exists(&sender) {
				return Err("Pubkey not bonded")
			}
			<LadderPubkey<T>>::remove(sender.clone());
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn _verify(_tx : T::Hash) -> Result{
        //TODO:verify signature or others
        Ok(())
    }

    pub fn is_validators(who: T::AccountId) -> bool{
        <session::Module<T>>::validators().contains(&who)
    }

    pub fn is_sessionkey(who: T::SessionKey) -> Result {
        if <consensus::Module<T>>::authorities().contains(&who) {
            return Ok(());
        }
        return Err("Not SessionKey");
    }

    /// determine if the current number of signatures is sufficient to send an event
    pub  fn check_signature(who: T::AccountId, transcation: T::Hash, sign: T::Hash, message: Vec<u8>) -> Result{

        let sender = who;

        let session_key: T::SessionKey = Decode::decode(&mut &sender.encode()[..]).unwrap();

        Self::is_sessionkey(session_key)?;

        // let s: T::SessionKey = Decode::decode(&mut sender.encode()).unwrap();
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
        <MessageRecord<T>>::insert(transcation.clone(),message.clone());

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


    pub fn inilize_secp256(id:T::AccountId,pubkey:Vec<u8>) {
        <LadderPubkey<T>>::insert(id,pubkey);
    }

    pub fn secp256_verify(who:T::AccountId, message: Vec<u8>, signature:Vec<u8>) -> Result{
        if !<LadderPubkey<T>>::exists(&who) {
            return Err("Pubkey not bonded")
        }
        let pubkey = Self::ladder_pubkey(&who);

        Self::secp256_verification(message,signature,pubkey)
    }

    pub fn secp256_verification(message: Vec<u8>, signature:Vec<u8>, pubkey:Vec<u8>) -> Result{
        let mut eth_header_vec:Vec<u8> = [25u8, 69, 116, 104, 101, 114, 101, 117, 109, 32, 83, 105, 103, 110, 101, 100, 32, 77, 101, 115, 115, 97, 103, 101, 58, 10].to_vec();
        let mut lenth_vec:Vec<u8> = Self::intu64_to_u8vec(message.len() as u64);
        let mut messageclone = message.clone();

        eth_header_vec.append(&mut lenth_vec);
        eth_header_vec.append(&mut messageclone);
        // now use eth_header_vec
        let message_to_verify = runtime_io::keccak_256(&eth_header_vec.as_slice());
        let mut tmp2: [u8; 65] = [0u8; 65];
        tmp2.copy_from_slice(signature.as_slice());

        match runtime_io::secp256k1_ecdsa_recover(&tmp2,&message_to_verify) {
            Ok(x) => {
                if &x[..] == pubkey.as_slice(){
                    return Ok(());
                }else {
                    return Err("Not Match!");
                }
            },
            Err(y) => return Err("Secp256k1 FAILED!"),
        }
    }

    /// convert intu64 into Vec<u8>
    pub fn intu64_to_u8vec(number: u64) -> Vec<u8> {
        let mut num = number.clone();
        let mut leftover = 0;
        let mut output :Vec<u8> = [].to_vec();
        loop{
            if num != 0 {
                leftover = num%10 + 48;
                num = num/10;
                output.push(leftover as u8);
            }else {
                break;
            }
        }
        output.reverse();
        output
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
            H256::from_low_u64_be(15),vec![1]));
            // the second tx will err
            assert_err!(Signcheck::check_signature(2,H256::from_low_u64_be(15),
            H256::from_low_u64_be(14),vec![1]),"This Transcation already been sent!");
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
            H256::from_low_u64_be(15),vec![1]),"Not enough signature!");
            assert_err!(Signcheck::check_signature(2,H256::from_low_u64_be(15),
            H256::from_low_u64_be(14),vec![1]),"Not enough signature!");
            assert_eq!(Signcheck::already_sent(H256::from_low_u64_be(15)),0);
            assert_err!(Signcheck::check_signature(3,H256::from_low_u64_be(15),
            H256::from_low_u64_be(13),vec![1]),"Not enough signature!");
            assert_err!(Signcheck::check_signature(4,H256::from_low_u64_be(15),
            H256::from_low_u64_be(12),vec![1]),"Not enough signature!");
            assert_ok!(Signcheck::check_signature(5,H256::from_low_u64_be(15),
            H256::from_low_u64_be(11),vec![1]));
            // already_sent == 1 means the Hash(15) has been accept
            assert_eq!(Signcheck::already_sent(H256::from_low_u64_be(15)),1);
        });
    }

    #[test]
    fn check_secp512() {
        with_externalities(&mut new_test_ext(), || {
            let mut data : Vec<u8>= "0000000000000000000000000000000100000000000000000000000000000002d573cade061a4a6e602f30931181805785b9505f000000000000000000000000000000000000000000000000000b1d3108ef2661a0dc7f47061b0b84e4dd3085f57e71c819ed6fb3ecc97fb8c31983c27f598fa6".from_hex().unwrap();
            let sign : Vec<u8>= "1a65da4f362ea2c1f374d22028a09c0587ca097abe2b5ae6c1dd25bae2a0e88a6d494c6da73e159dab4fa927b3d2837514f879253d17bdc274ccfdeea822d53401".from_hex().unwrap();
            let pubkey_real : Vec<u8>= "b59fe985fd3c00de38aa1094d7a8a34914771d025a8038238502d07e8b4048ac9afb68d22c77376921a056cf41f6f659f4f70f9b07d2887ffe3b9362d57070b6".from_hex().unwrap();
            assert_ok!(Signcheck::secp256_verification(data.clone(),sign.clone(),pubkey_real.clone()));
            let pubkey_fake : Vec<u8>= "14b59fe985fd3c00de38aa1094d7a8a34914771d025a8038238502d07e8b4048ac9afb68d22c77376921a056cf41f6f659f4f70f9b07d2887ffe3b9362d57070b6".from_hex().unwrap();
            assert_err!(Signcheck::secp256_verification(data.clone(),sign.clone(),pubkey_fake.clone()),"Not Match!");
        });
    }

    #[test]
    fn bond_pubkey_test() {
        with_externalities(&mut new_test_ext(), || {
            let mut data : Vec<u8>= "0000000000000000000000000000000100000000000000000000000000000002d573cade061a4a6e602f30931181805785b9505f000000000000000000000000000000000000000000000000000b1d3108ef2661a0dc7f47061b0b84e4dd3085f57e71c819ed6fb3ecc97fb8c31983c27f598fa6".from_hex().unwrap();
            let sign : Vec<u8>= "1a65da4f362ea2c1f374d22028a09c0587ca097abe2b5ae6c1dd25bae2a0e88a6d494c6da73e159dab4fa927b3d2837514f879253d17bdc274ccfdeea822d53401".from_hex().unwrap();
            let pubkey_real : Vec<u8>= "b59fe985fd3c00de38aa1094d7a8a34914771d025a8038238502d07e8b4048ac9afb68d22c77376921a056cf41f6f659f4f70f9b07d2887ffe3b9362d57070b6".from_hex().unwrap();
            assert_err!(Signcheck::unbond_pubkey(Origin::signed(5)),"Pubkey not bonded");
            assert_ok!(Signcheck::bond_pubkey(Origin::signed(5),pubkey_real.clone()));
            assert_err!(Signcheck::bond_pubkey(Origin::signed(5),pubkey_real.clone()),"Pubkey already bonded");
            assert_ok!(Signcheck::secp256_verify(5,data.clone(),sign.clone()));
        });
    }
}