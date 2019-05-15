// This file is generated by rust-protobuf 2.0.4. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct Transaction {
    // message fields
    pub to: ::std::string::String,
    pub nonce: ::std::string::String,
    pub quota: u64,
    pub valid_until_block: u64,
    pub data: ::std::vec::Vec<u8>,
    pub value: ::std::vec::Vec<u8>,
    pub chain_id: u32,
    pub version: u32,
    pub to_v1: ::std::vec::Vec<u8>,
    pub chain_id_v1: ::std::vec::Vec<u8>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

impl Transaction {
    pub fn new() -> Transaction {
        ::std::default::Default::default()
    }

    // string to = 1;

    pub fn clear_to(&mut self) {
        self.to.clear();
    }

    // Param is passed by value, moved
    pub fn set_to(&mut self, v: ::std::string::String) {
        self.to = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_to(&mut self) -> &mut ::std::string::String {
        &mut self.to
    }

    // Take field
    pub fn take_to(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.to, ::std::string::String::new())
    }

    pub fn get_to(&self) -> &str {
        &self.to
    }

    // string nonce = 2;

    pub fn clear_nonce(&mut self) {
        self.nonce.clear();
    }

    // Param is passed by value, moved
    pub fn set_nonce(&mut self, v: ::std::string::String) {
        self.nonce = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_nonce(&mut self) -> &mut ::std::string::String {
        &mut self.nonce
    }

    // Take field
    pub fn take_nonce(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.nonce, ::std::string::String::new())
    }

    pub fn get_nonce(&self) -> &str {
        &self.nonce
    }

    // uint64 quota = 3;

    pub fn clear_quota(&mut self) {
        self.quota = 0;
    }

    // Param is passed by value, moved
    pub fn set_quota(&mut self, v: u64) {
        self.quota = v;
    }

    pub fn get_quota(&self) -> u64 {
        self.quota
    }

    // uint64 valid_until_block = 4;

    pub fn clear_valid_until_block(&mut self) {
        self.valid_until_block = 0;
    }

    // Param is passed by value, moved
    pub fn set_valid_until_block(&mut self, v: u64) {
        self.valid_until_block = v;
    }

    pub fn get_valid_until_block(&self) -> u64 {
        self.valid_until_block
    }

    // bytes data = 5;

    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    // Param is passed by value, moved
    pub fn set_data(&mut self, v: ::std::vec::Vec<u8>) {
        self.data = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_data(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.data
    }

    // Take field
    pub fn take_data(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.data, ::std::vec::Vec::new())
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    // bytes value = 6;

    pub fn clear_value(&mut self) {
        self.value.clear();
    }

    // Param is passed by value, moved
    pub fn set_value(&mut self, v: ::std::vec::Vec<u8>) {
        self.value = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_value(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.value
    }

    // Take field
    pub fn take_value(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.value, ::std::vec::Vec::new())
    }

    pub fn get_value(&self) -> &[u8] {
        &self.value
    }

    // uint32 chain_id = 7;

    pub fn clear_chain_id(&mut self) {
        self.chain_id = 0;
    }

    // Param is passed by value, moved
    pub fn set_chain_id(&mut self, v: u32) {
        self.chain_id = v;
    }

    pub fn get_chain_id(&self) -> u32 {
        self.chain_id
    }

    // uint32 version = 8;

    pub fn clear_version(&mut self) {
        self.version = 0;
    }

    // Param is passed by value, moved
    pub fn set_version(&mut self, v: u32) {
        self.version = v;
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

    // bytes to_v1 = 9;

    pub fn clear_to_v1(&mut self) {
        self.to_v1.clear();
    }

    // Param is passed by value, moved
    pub fn set_to_v1(&mut self, v: ::std::vec::Vec<u8>) {
        self.to_v1 = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_to_v1(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.to_v1
    }

    // Take field
    pub fn take_to_v1(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.to_v1, ::std::vec::Vec::new())
    }

    pub fn get_to_v1(&self) -> &[u8] {
        &self.to_v1
    }

    // bytes chain_id_v1 = 10;

    pub fn clear_chain_id_v1(&mut self) {
        self.chain_id_v1.clear();
    }

    // Param is passed by value, moved
    pub fn set_chain_id_v1(&mut self, v: ::std::vec::Vec<u8>) {
        self.chain_id_v1 = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_chain_id_v1(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.chain_id_v1
    }

    // Take field
    pub fn take_chain_id_v1(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.chain_id_v1, ::std::vec::Vec::new())
    }

    pub fn get_chain_id_v1(&self) -> &[u8] {
        &self.chain_id_v1
    }
}

impl ::protobuf::Message for Transaction {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.to)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.nonce)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.quota = tmp;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.valid_until_block = tmp;
                },
                5 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.data)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.value)?;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.chain_id = tmp;
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.version = tmp;
                },
                9 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.to_v1)?;
                },
                10 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.chain_id_v1)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.to.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.to);
        }
        if !self.nonce.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.nonce);
        }
        if self.quota != 0 {
            my_size += ::protobuf::rt::value_size(3, self.quota, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.valid_until_block != 0 {
            my_size += ::protobuf::rt::value_size(4, self.valid_until_block, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.data.is_empty() {
            my_size += ::protobuf::rt::bytes_size(5, &self.data);
        }
        if !self.value.is_empty() {
            my_size += ::protobuf::rt::bytes_size(6, &self.value);
        }
        if self.chain_id != 0 {
            my_size += ::protobuf::rt::value_size(7, self.chain_id, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.version != 0 {
            my_size += ::protobuf::rt::value_size(8, self.version, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.to_v1.is_empty() {
            my_size += ::protobuf::rt::bytes_size(9, &self.to_v1);
        }
        if !self.chain_id_v1.is_empty() {
            my_size += ::protobuf::rt::bytes_size(10, &self.chain_id_v1);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.to.is_empty() {
            os.write_string(1, &self.to)?;
        }
        if !self.nonce.is_empty() {
            os.write_string(2, &self.nonce)?;
        }
        if self.quota != 0 {
            os.write_uint64(3, self.quota)?;
        }
        if self.valid_until_block != 0 {
            os.write_uint64(4, self.valid_until_block)?;
        }
        if !self.data.is_empty() {
            os.write_bytes(5, &self.data)?;
        }
        if !self.value.is_empty() {
            os.write_bytes(6, &self.value)?;
        }
        if self.chain_id != 0 {
            os.write_uint32(7, self.chain_id)?;
        }
        if self.version != 0 {
            os.write_uint32(8, self.version)?;
        }
        if !self.to_v1.is_empty() {
            os.write_bytes(9, &self.to_v1)?;
        }
        if !self.chain_id_v1.is_empty() {
            os.write_bytes(10, &self.chain_id_v1)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Transaction {
        Transaction::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "to",
                    |m: &Transaction| { &m.to },
                    |m: &mut Transaction| { &mut m.to },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "nonce",
                    |m: &Transaction| { &m.nonce },
                    |m: &mut Transaction| { &mut m.nonce },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "quota",
                    |m: &Transaction| { &m.quota },
                    |m: &mut Transaction| { &mut m.quota },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "valid_until_block",
                    |m: &Transaction| { &m.valid_until_block },
                    |m: &mut Transaction| { &mut m.valid_until_block },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "data",
                    |m: &Transaction| { &m.data },
                    |m: &mut Transaction| { &mut m.data },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "value",
                    |m: &Transaction| { &m.value },
                    |m: &mut Transaction| { &mut m.value },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "chain_id",
                    |m: &Transaction| { &m.chain_id },
                    |m: &mut Transaction| { &mut m.chain_id },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "version",
                    |m: &Transaction| { &m.version },
                    |m: &mut Transaction| { &mut m.version },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "to_v1",
                    |m: &Transaction| { &m.to_v1 },
                    |m: &mut Transaction| { &mut m.to_v1 },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "chain_id_v1",
                    |m: &Transaction| { &m.chain_id_v1 },
                    |m: &mut Transaction| { &mut m.chain_id_v1 },
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Transaction>(
                    "Transaction",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static Transaction {
        static mut instance: ::protobuf::lazy::Lazy<Transaction> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Transaction,
        };
        unsafe {
            instance.get(Transaction::new)
        }
    }
}

impl ::protobuf::Clear for Transaction {
    fn clear(&mut self) {
        self.clear_to();
        self.clear_nonce();
        self.clear_quota();
        self.clear_valid_until_block();
        self.clear_data();
        self.clear_value();
        self.clear_chain_id();
        self.clear_version();
        self.clear_to_v1();
        self.clear_chain_id_v1();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Transaction {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct UnverifiedTransaction {
    // message fields
    pub transaction: ::protobuf::SingularPtrField<Transaction>,
    pub signature: ::std::vec::Vec<u8>,
    pub crypto: Crypto,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

impl UnverifiedTransaction {
    pub fn new() -> UnverifiedTransaction {
        ::std::default::Default::default()
    }

    // .Transaction transaction = 1;

    pub fn clear_transaction(&mut self) {
        self.transaction.clear();
    }

    pub fn has_transaction(&self) -> bool {
        self.transaction.is_some()
    }

    // Param is passed by value, moved
    pub fn set_transaction(&mut self, v: Transaction) {
        self.transaction = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_transaction(&mut self) -> &mut Transaction {
        if self.transaction.is_none() {
            self.transaction.set_default();
        }
        self.transaction.as_mut().unwrap()
    }

    // Take field
    pub fn take_transaction(&mut self) -> Transaction {
        self.transaction.take().unwrap_or_else(|| Transaction::new())
    }

    pub fn get_transaction(&self) -> &Transaction {
        self.transaction.as_ref().unwrap_or_else(|| Transaction::default_instance())
    }

    // bytes signature = 2;

    pub fn clear_signature(&mut self) {
        self.signature.clear();
    }

    // Param is passed by value, moved
    pub fn set_signature(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.signature
    }

    // Take field
    pub fn take_signature(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.signature, ::std::vec::Vec::new())
    }

    pub fn get_signature(&self) -> &[u8] {
        &self.signature
    }

    // .Crypto crypto = 3;

    pub fn clear_crypto(&mut self) {
        self.crypto = Crypto::DEFAULT;
    }

    // Param is passed by value, moved
    pub fn set_crypto(&mut self, v: Crypto) {
        self.crypto = v;
    }

    pub fn get_crypto(&self) -> Crypto {
        self.crypto
    }
}

impl ::protobuf::Message for UnverifiedTransaction {
    fn is_initialized(&self) -> bool {
        for v in &self.transaction {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.transaction)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.signature)?;
                },
                3 => {
                    ::protobuf::rt::read_proto3_enum_with_unknown_fields_into(wire_type, is, &mut self.crypto, 3, &mut self.unknown_fields)?
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.transaction.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if !self.signature.is_empty() {
            my_size += ::protobuf::rt::bytes_size(2, &self.signature);
        }
        if self.crypto != Crypto::DEFAULT {
            my_size += ::protobuf::rt::enum_size(3, self.crypto);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.transaction.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if !self.signature.is_empty() {
            os.write_bytes(2, &self.signature)?;
        }
        if self.crypto != Crypto::DEFAULT {
            os.write_enum(3, self.crypto.value())?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> UnverifiedTransaction {
        UnverifiedTransaction::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Transaction>>(
                    "transaction",
                    |m: &UnverifiedTransaction| { &m.transaction },
                    |m: &mut UnverifiedTransaction| { &mut m.transaction },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature",
                    |m: &UnverifiedTransaction| { &m.signature },
                    |m: &mut UnverifiedTransaction| { &mut m.signature },
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<Crypto>>(
                    "crypto",
                    |m: &UnverifiedTransaction| { &m.crypto },
                    |m: &mut UnverifiedTransaction| { &mut m.crypto },
                ));
                ::protobuf::reflect::MessageDescriptor::new::<UnverifiedTransaction>(
                    "UnverifiedTransaction",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static UnverifiedTransaction {
        static mut instance: ::protobuf::lazy::Lazy<UnverifiedTransaction> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const UnverifiedTransaction,
        };
        unsafe {
            instance.get(UnverifiedTransaction::new)
        }
    }
}

impl ::protobuf::Clear for UnverifiedTransaction {
    fn clear(&mut self) {
        self.clear_transaction();
        self.clear_signature();
        self.clear_crypto();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for UnverifiedTransaction {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for UnverifiedTransaction {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Crypto {
    DEFAULT = 0,
    RESERVED = 1,
}

impl ::protobuf::ProtobufEnum for Crypto {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Crypto> {
        match value {
            0 => ::std::option::Option::Some(Crypto::DEFAULT),
            1 => ::std::option::Option::Some(Crypto::RESERVED),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [Crypto] = &[
            Crypto::DEFAULT,
            Crypto::RESERVED,
        ];
        values
    }

    fn enum_descriptor_static() -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Crypto", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Crypto {
}

impl ::std::default::Default for Crypto {
    fn default() -> Self {
        Crypto::DEFAULT
    }
}

impl ::protobuf::reflect::ProtobufValue for Crypto {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x11transaction.proto\"\x89\x02\n\x0bTransaction\x12\x0e\n\x02to\x18\
    \x01\x20\x01(\tR\x02to\x12\x14\n\x05nonce\x18\x02\x20\x01(\tR\x05nonce\
    \x12\x14\n\x05quota\x18\x03\x20\x01(\x04R\x05quota\x12*\n\x11valid_until\
    _block\x18\x04\x20\x01(\x04R\x0fvalidUntilBlock\x12\x12\n\x04data\x18\
    \x05\x20\x01(\x0cR\x04data\x12\x14\n\x05value\x18\x06\x20\x01(\x0cR\x05v\
    alue\x12\x19\n\x08chain_id\x18\x07\x20\x01(\rR\x07chainId\x12\x18\n\x07v\
    ersion\x18\x08\x20\x01(\rR\x07version\x12\x13\n\x05to_v1\x18\t\x20\x01(\
    \x0cR\x04toV1\x12\x1e\n\x0bchain_id_v1\x18\n\x20\x01(\x0cR\tchainIdV1\"\
    \x86\x01\n\x15UnverifiedTransaction\x12.\n\x0btransaction\x18\x01\x20\
    \x01(\x0b2\x0c.TransactionR\x0btransaction\x12\x1c\n\tsignature\x18\x02\
    \x20\x01(\x0cR\tsignature\x12\x1f\n\x06crypto\x18\x03\x20\x01(\x0e2\x07.\
    CryptoR\x06crypto*#\n\x06Crypto\x12\x0b\n\x07DEFAULT\x10\0\x12\x0c\n\x08\
    RESERVED\x10\x01J\xba\x08\n\x06\x12\x04\0\0\x18\x01\n\x08\n\x01\x0c\x12\
    \x03\0\0\x12\n\n\n\x02\x05\0\x12\x04\x02\0\x05\x01\n\n\n\x03\x05\0\x01\
    \x12\x03\x02\x05\x0b\n\x0b\n\x04\x05\0\x02\0\x12\x03\x03\x04\x10\n\x0c\n\
    \x05\x05\0\x02\0\x01\x12\x03\x03\x04\x0b\n\x0c\n\x05\x05\0\x02\0\x02\x12\
    \x03\x03\x0e\x0f\n\x0b\n\x04\x05\0\x02\x01\x12\x03\x04\x04\x11\n\x0c\n\
    \x05\x05\0\x02\x01\x01\x12\x03\x04\x04\x0c\n\x0c\n\x05\x05\0\x02\x01\x02\
    \x12\x03\x04\x0f\x10\n\n\n\x02\x04\0\x12\x04\x07\0\x12\x01\n\n\n\x03\x04\
    \0\x01\x12\x03\x07\x08\x13\n\x0b\n\x04\x04\0\x02\0\x12\x03\x08\x04\x12\n\
    \r\n\x05\x04\0\x02\0\x04\x12\x04\x08\x04\x07\x15\n\x0c\n\x05\x04\0\x02\0\
    \x05\x12\x03\x08\x04\n\n\x0c\n\x05\x04\0\x02\0\x01\x12\x03\x08\x0b\r\n\
    \x0c\n\x05\x04\0\x02\0\x03\x12\x03\x08\x10\x11\n\x0b\n\x04\x04\0\x02\x01\
    \x12\x03\t\x04\x15\n\r\n\x05\x04\0\x02\x01\x04\x12\x04\t\x04\x08\x12\n\
    \x0c\n\x05\x04\0\x02\x01\x05\x12\x03\t\x04\n\n\x0c\n\x05\x04\0\x02\x01\
    \x01\x12\x03\t\x0b\x10\n\x0c\n\x05\x04\0\x02\x01\x03\x12\x03\t\x13\x14\n\
    \x0b\n\x04\x04\0\x02\x02\x12\x03\n\x04\x15\n\r\n\x05\x04\0\x02\x02\x04\
    \x12\x04\n\x04\t\x15\n\x0c\n\x05\x04\0\x02\x02\x05\x12\x03\n\x04\n\n\x0c\
    \n\x05\x04\0\x02\x02\x01\x12\x03\n\x0b\x10\n\x0c\n\x05\x04\0\x02\x02\x03\
    \x12\x03\n\x13\x14\n\x0b\n\x04\x04\0\x02\x03\x12\x03\x0b\x04!\n\r\n\x05\
    \x04\0\x02\x03\x04\x12\x04\x0b\x04\n\x15\n\x0c\n\x05\x04\0\x02\x03\x05\
    \x12\x03\x0b\x04\n\n\x0c\n\x05\x04\0\x02\x03\x01\x12\x03\x0b\x0b\x1c\n\
    \x0c\n\x05\x04\0\x02\x03\x03\x12\x03\x0b\x1f\x20\n\x0b\n\x04\x04\0\x02\
    \x04\x12\x03\x0c\x04\x13\n\r\n\x05\x04\0\x02\x04\x04\x12\x04\x0c\x04\x0b\
    !\n\x0c\n\x05\x04\0\x02\x04\x05\x12\x03\x0c\x04\t\n\x0c\n\x05\x04\0\x02\
    \x04\x01\x12\x03\x0c\n\x0e\n\x0c\n\x05\x04\0\x02\x04\x03\x12\x03\x0c\x11\
    \x12\n\x0b\n\x04\x04\0\x02\x05\x12\x03\r\x04\x14\n\r\n\x05\x04\0\x02\x05\
    \x04\x12\x04\r\x04\x0c\x13\n\x0c\n\x05\x04\0\x02\x05\x05\x12\x03\r\x04\t\
    \n\x0c\n\x05\x04\0\x02\x05\x01\x12\x03\r\n\x0f\n\x0c\n\x05\x04\0\x02\x05\
    \x03\x12\x03\r\x12\x13\n\x0b\n\x04\x04\0\x02\x06\x12\x03\x0e\x04\x18\n\r\
    \n\x05\x04\0\x02\x06\x04\x12\x04\x0e\x04\r\x14\n\x0c\n\x05\x04\0\x02\x06\
    \x05\x12\x03\x0e\x04\n\n\x0c\n\x05\x04\0\x02\x06\x01\x12\x03\x0e\x0b\x13\
    \n\x0c\n\x05\x04\0\x02\x06\x03\x12\x03\x0e\x16\x17\n\x0b\n\x04\x04\0\x02\
    \x07\x12\x03\x0f\x04\x17\n\r\n\x05\x04\0\x02\x07\x04\x12\x04\x0f\x04\x0e\
    \x18\n\x0c\n\x05\x04\0\x02\x07\x05\x12\x03\x0f\x04\n\n\x0c\n\x05\x04\0\
    \x02\x07\x01\x12\x03\x0f\x0b\x12\n\x0c\n\x05\x04\0\x02\x07\x03\x12\x03\
    \x0f\x15\x16\n\x0b\n\x04\x04\0\x02\x08\x12\x03\x10\x04\x14\n\r\n\x05\x04\
    \0\x02\x08\x04\x12\x04\x10\x04\x0f\x17\n\x0c\n\x05\x04\0\x02\x08\x05\x12\
    \x03\x10\x04\t\n\x0c\n\x05\x04\0\x02\x08\x01\x12\x03\x10\n\x0f\n\x0c\n\
    \x05\x04\0\x02\x08\x03\x12\x03\x10\x12\x13\n\x0b\n\x04\x04\0\x02\t\x12\
    \x03\x11\x04\x1b\n\r\n\x05\x04\0\x02\t\x04\x12\x04\x11\x04\x10\x14\n\x0c\
    \n\x05\x04\0\x02\t\x05\x12\x03\x11\x04\t\n\x0c\n\x05\x04\0\x02\t\x01\x12\
    \x03\x11\n\x15\n\x0c\n\x05\x04\0\x02\t\x03\x12\x03\x11\x18\x1a\n\n\n\x02\
    \x04\x01\x12\x04\x14\0\x18\x01\n\n\n\x03\x04\x01\x01\x12\x03\x14\x08\x1d\
    \n\x0b\n\x04\x04\x01\x02\0\x12\x03\x15\x04\x20\n\r\n\x05\x04\x01\x02\0\
    \x04\x12\x04\x15\x04\x14\x1f\n\x0c\n\x05\x04\x01\x02\0\x06\x12\x03\x15\
    \x04\x0f\n\x0c\n\x05\x04\x01\x02\0\x01\x12\x03\x15\x10\x1b\n\x0c\n\x05\
    \x04\x01\x02\0\x03\x12\x03\x15\x1e\x1f\n\x0b\n\x04\x04\x01\x02\x01\x12\
    \x03\x16\x04\x18\n\r\n\x05\x04\x01\x02\x01\x04\x12\x04\x16\x04\x15\x20\n\
    \x0c\n\x05\x04\x01\x02\x01\x05\x12\x03\x16\x04\t\n\x0c\n\x05\x04\x01\x02\
    \x01\x01\x12\x03\x16\n\x13\n\x0c\n\x05\x04\x01\x02\x01\x03\x12\x03\x16\
    \x16\x17\n\x0b\n\x04\x04\x01\x02\x02\x12\x03\x17\x04\x16\n\r\n\x05\x04\
    \x01\x02\x02\x04\x12\x04\x17\x04\x16\x18\n\x0c\n\x05\x04\x01\x02\x02\x06\
    \x12\x03\x17\x04\n\n\x0c\n\x05\x04\x01\x02\x02\x01\x12\x03\x17\x0b\x11\n\
    \x0c\n\x05\x04\x01\x02\x02\x03\x12\x03\x17\x14\x15b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}