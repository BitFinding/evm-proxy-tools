//! evm-proxy-tools is a library for detecting and analyzing Ethereum proxy contracts.
//! 
//! This crate provides tools for analyzing different types of proxy patterns commonly used
//! in Ethereum smart contracts, including:
//! 
//! - [EIP-1167](https://eips.ethereum.org/EIPS/eip-1167) Minimal Proxy Contract
//! - [EIP-1967](https://eips.ethereum.org/EIPS/eip-1967) Standard Proxy Storage Slots
//! - [EIP-2535](https://eips.ethereum.org/EIPS/eip-2535) Diamond Standard
//! - Custom proxy implementations
//! 
//! # Quick Start
//! 
//! ```no_run
//! use evm_proxy_tools::{ProxyDetector, ProxyType, Result};
//! 
//! # async fn example() -> Result<()> {
//!     let contract_code = vec![/* contract bytecode */];
//!     
//!     // Detect proxy type
//!     if let Some((proxy_type, dispatch)) = ProxyDetector::detect(&contract_code)? {
//!         println!("Found proxy type: {:?}", proxy_type);
//!         
//!         // Get implementation address
//!         let implementation = dispatch.get_implementation().await?;
//!         println!("Implementation at: {:?}", implementation);
//!     }
//!     
//!     Ok(())
//! # }
//! ```
//! 
//! # Features
//! 
//! - Static analysis of contract bytecode
//! - Detection of standard and custom proxy patterns
//! - Implementation contract resolution
//! - Async-first API design
//! - Comprehensive error handling
//! 
//! # Error Handling
//! 
//! This crate uses custom error types via [`ProxyError`](errors::ProxyError).
//! All public functions return [`Result<T>`](errors::Result) which should be properly handled.

mod consts;
mod detect;
mod errors;
mod proxy_inspector;
mod read;
mod types;
pub mod utils;

pub use detect::ProxyDetector;
pub use errors::{ProxyError, Result};
pub use read::get_proxy_implementation;
pub use types::{ProxyDispatch, ProxyImplementation, ProxyType};

// Re-export common types for convenience
pub use alloy_primitives::{Address, Bytes, U256};
