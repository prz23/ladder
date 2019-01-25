//! A `CodeExecutor` specialisation which uses natively compiled runtime when the wasm to be
//! executed is equivalent to the natively compiled code.

#![cfg_attr(feature = "benchmarks", feature(test))]

extern crate node_runtime;
#[macro_use] extern crate substrate_executor;
#[cfg_attr(test, macro_use)] extern crate substrate_primitives as primitives;

pub use substrate_executor::NativeExecutor;
native_executor_instance!(pub Executor, node_runtime::api::dispatch, node_runtime::native_version, include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/node_runtime.compact.wasm"));