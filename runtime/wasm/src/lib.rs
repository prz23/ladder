//! The Substrate runtime reexported for WebAssembly compile.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate node_runtime;
pub use node_runtime::*;
