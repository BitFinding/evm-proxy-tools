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
    #[error("Failed to read implementation address for proxy at {address}: {message}")]
    ImplementationReadError {
        address: Address,
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Invalid storage slot access
    #[error("Invalid storage slot access at {slot}: {message}")]
    InvalidStorageAccess {
        slot: U256,
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// RPC communication error
    #[error("RPC error: {message}")]
    RpcError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Invalid or malformed bytecode
    #[error("Invalid bytecode for address {address}: {reason}")]
    InvalidBytecode {
        address: Address,
        reason: String,
    },

    /// Execution trace error
    #[error("Execution trace error: {message}")]
    TraceError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Generic proxy error with context
    #[error("{message}")]
    Other {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Result type for proxy operations
pub type Result<T> = std::result::Result<T, ProxyError>;
