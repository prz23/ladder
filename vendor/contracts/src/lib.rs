extern crate ethabi;
#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;

use_contract!(bridge, "./compiled_contracts/Bridge.abi");

use_contract!(mapper, "./compiled_contracts/LockableMapper.abi");