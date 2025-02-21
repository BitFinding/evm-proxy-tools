use crate::{ProxyType, ProxyDispatch, Result};
use alloy_primitives::Bytes;

/// Core trait for implementing proxy detection strategies
pub trait DetectionStrategy {
    /// Attempt to detect proxy pattern
    fn detect(&self, code: &Bytes) -> Result<Option<(ProxyType, ProxyDispatch)>>;

    /// Name of the detection strategy
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProxyType;
    use alloy_primitives::hex;

    #[test]
    fn test_proxy_detector_empty_code() {
        let detector = ProxyDetector::new();
        let empty_code = Bytes::new();
        assert!(detector.detect(&empty_code).unwrap().is_none());
    }

    #[test]
    fn test_proxy_detector_invalid_code() {
        let detector = ProxyDetector::new();
        let invalid_code = Bytes::from(vec![0xFF; 32]);
        assert!(detector.detect(&invalid_code).unwrap().is_none());
    }

    #[test]
    fn test_proxy_detector_strategies_order() {
        let detector = ProxyDetector::new();
        // Static analysis should be tried first
        assert_eq!(detector.strategies[0].name(), "StaticDetector");
        // Dynamic analysis should be tried second
        assert_eq!(detector.strategies[1].name(), "DynamicDetector");
    }
}

/// Static analysis based detection (bytecode patterns)
pub mod static_detector;
/// Dynamic analysis based detection (execution tracing)
pub mod dynamic_detector;

// Re-export specific detectors
pub use static_detector::StaticDetector;
pub use dynamic_detector::DynamicDetector;

/// Unified proxy detector that combines multiple strategies
pub struct ProxyDetector {
    strategies: Vec<Box<dyn DetectionStrategy>>,
}

impl Default for ProxyDetector {
    fn default() -> Self {
        Self {
            strategies: vec![
                Box::new(StaticDetector::default()),
                Box::new(DynamicDetector::default()),
            ]
        }
    }
}

impl ProxyDetector {
    /// Creates a new detector with default strategies
    pub fn new() -> Self {
        Self::default()
    }

    /// Detect proxy type using all available strategies
    pub fn detect(&self, code: &Bytes) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        for strategy in &self.strategies {
            if let Some(result) = strategy.detect(code)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}
