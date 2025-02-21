//! evm-proxy-tools is a library for detecting and analyzing Ethereum proxy contracts.
//! 
//! This crate provides tools to:
//! - Detect various proxy patterns in EVM bytecode
//! - Analyze proxy implementation contracts
//! - Resolve proxy implementations
//! 
//! # Example
//! ```no_run
//! use evm_proxy_tools::{get_proxy_type, ProxyType};
//! 
//! # async fn example() {
//! let contract_code = vec![/* contract bytecode */];
//! if let Some((proxy_type, dispatch)) = get_proxy_type(&contract_code) {
//!     println!("Detected proxy type: {:?}", proxy_type);
//! }
//! # }
//! ```

mod consts;
mod read;
mod detect;
mod types;
pub mod utils;
mod proxy_inspector;

pub use types::{ProxyType, ProxyDispatch};
pub use read::get_proxy_implementation;
pub use detect::get_proxy_type;

// Re-export common types for convenience
pub use revm::primitives::{Address, Bytes, U256};
