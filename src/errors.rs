//! Error types for the evm-proxy-tools crate.

use thiserror::Error;
use alloy_primitives::{Address, U256};

/// Errors that can occur when working with proxy contracts
#[derive(Error, Debug)]
pub enum ProxyError {
    /// Failed to detect proxy type from bytecode
    #[error("Failed to detect proxy type: {0}")]
    DetectionFailed(String),

    /// Failed to read implementation address
    #[error("Failed to read implementation address for proxy at {address}")]
    ImplementationReadError {
        address: Address,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Invalid storage slot access
    #[error("Invalid storage slot access at {slot}")]
    InvalidStorageAccess {
        slot: U256,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// RPC communication error
    #[error("RPC error: {0}")]
    RpcError(String),

    /// Invalid or malformed bytecode
    #[error("Invalid bytecode for address {address}: {reason}")]
    InvalidBytecode {
        address: Address,
        reason: String,
    },

    /// Generic proxy error with context
    #[error("{0}")]
    Other(String),
}

/// Result type for proxy operations
pub type Result<T> = std::result::Result<T, ProxyError>;
