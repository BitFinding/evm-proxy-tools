use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use alloy_primitives::{Address, U256};
use crate::{ProxyType, ProxyDispatch};

/// Confidence level of the proxy detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionConfidence {
    /// High confidence (e.g., exact bytecode match)
    High,
    /// Medium confidence (e.g., storage pattern match)
    Medium,
    /// Low confidence (e.g., heuristic match)
    Low,
}

/// Method used to detect the proxy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Static analysis of bytecode
    Static,
    /// Dynamic analysis through execution
    Dynamic,
    /// Combination of methods
    Hybrid,
}

/// Detailed result of proxy detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyDetectionResult {
    /// Type of proxy detected
    pub proxy_type: ProxyType,
    /// Dispatch mechanism
    pub dispatch: ProxyDispatch,
    /// Confidence level of detection
    pub confidence: DetectionConfidence,
    /// Method used for detection
    pub method: DetectionMethod,
    /// Additional metadata about the detection
    #[serde(default)]
    pub meta: HashMap<String, String>,
}

impl ProxyDetectionResult {
    /// Creates a new detection result
    pub fn new(
        proxy_type: ProxyType,
        dispatch: ProxyDispatch,
        confidence: DetectionConfidence,
        method: DetectionMethod,
    ) -> Self {
        Self {
            proxy_type,
            dispatch,
            confidence,
            method,
            meta: HashMap::new(),
        }
    }

    /// Adds metadata to the detection result
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Configuration for proxy detection
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Maximum gas limit for dynamic analysis
    pub max_gas: u64,
    /// Contract address for dynamic analysis
    pub contract_address: Address,
    /// Caller address for dynamic analysis
    pub caller_address: Address,
    /// Storage slots to check
    pub storage_slots: Vec<U256>,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            max_gas: 30_000_000,
            contract_address: Address::from([0xff; 20]),
            caller_address: Address::from([0xfe; 20]),
            storage_slots: vec![],
        }
    }
}
