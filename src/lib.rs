//! # evm-proxy-tools
//!
//! Detect and read EVM proxy contract implementations.
//!
//! This crate provides tools to identify proxy patterns in EVM bytecode and
//! resolve the implementation addresses they point to. It supports a wide range
//! of proxy standards including minimal proxies, storage-based proxies, and
//! diamond proxies.
//!
//! ## Supported Proxy Types
//!
//! | Standard | Description |
//! |----------|-------------|
//! | [EIP-1167](https://eips.ethereum.org/EIPS/eip-1167) | Minimal Proxy Contract (clone factory) |
//! | [EIP-1967](https://eips.ethereum.org/EIPS/eip-1967) | Standard Proxy Storage Slots |
//! | [EIP-1822](https://eips.ethereum.org/EIPS/eip-1822) | Universal Upgradeable Proxy (UUPS) |
//! | [EIP-2535](https://eips.ethereum.org/EIPS/eip-2535) | Diamond Standard (multi-facet) |
//! | [EIP-3448](https://eips.ethereum.org/EIPS/eip-3448) | MetaProxy Standard |
//! | [EIP-7511](https://eips.ethereum.org/EIPS/eip-7511) | Minimal Proxy with PUSH0 |
//! | EIP-897 | DelegateProxy interface |
//!
//! ## Quick Start
//!
//! ```ignore
//! use evm_proxy_tools::{get_proxy_type, get_proxy_implementation, ProxyType, Dispatch};
//! use alloy::providers::ProviderBuilder;
//!
//! // Detect proxy type from bytecode
//! let bytecode = hex::decode("363d3d373d3d3d363d73...").unwrap();
//! if let Some((proxy_type, dispatch)) = get_proxy_type(&bytecode) {
//!     println!("Detected: {:?}", proxy_type);
//!     
//!     // Read implementation address
//!     let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
//!     let impl_addr = get_proxy_implementation(provider, &address, &dispatch, None).await?;
//! }
//! ```

mod consts;
mod read;
mod detect;
mod types;
pub mod utils;
mod proxy_inspector;

pub use types::{ProxyType, Dispatch, ProxyDispatch, Detection};
pub use read::{get_proxy_implementation, ProxyImplementation, ProxyReadError};
pub use detect::get_proxy_type;
