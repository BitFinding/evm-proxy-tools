use alloy_primitives::{Address, Bytes, U256};
use revm::{inspector_handle_register, primitives::{TransactTo, TxEnv}, EvmBuilder};
use crate::{ProxyType, ProxyDispatch, Result, proxy_inspector::{ProxyInspector, ProxyDetectDB}};
use super::DetectionStrategy;

/// Detector using dynamic execution analysis
#[derive(Default)]
pub struct DynamicDetector {
    test_inputs: Vec<Bytes>,
}

#[derive(Debug)]
pub struct TraceConfig {
    pub contract_address: Address,
    pub caller_address: Address,
    pub gas_limit: u64,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            contract_address: Address::from(hex_literal::hex!("00ff0000ff0000ff0000ff0000ff0000ff0000ff")),
            caller_address: Address::from(hex_literal::hex!("11ff0000ff0000ff0000ff0000ff0000ff0000ff")),
            gas_limit: 30_000_000,
        }
    }
}

impl DynamicDetector {
    pub fn new() -> Self {
        Self {
            test_inputs: vec![
                Bytes::from(vec![0xaa, 0xcc, 0xbb, 0xdd]),
                Bytes::from(vec![0xcc, 0xbb, 0xdd, 0xf1, 0xf1, 0xf1, 0xf1, 0xf1, 0xf1, 0xf1]),
                Bytes::from(vec![0x01, 0x02, 0x04, 0x11])
            ]
        }
    }

    pub fn with_test_inputs(inputs: Vec<Bytes>) -> Self {
        Self { test_inputs: inputs }
    }

    fn check_all_are_equal(&self,  &[InspectorData]) -> bool {
        if data.is_empty() {
            return false;
        }
        let first = &data[0];
        data.iter().all(|e| e == first)
    }

    fn check_trace_validity(&self, trace: &ProxyInspector) -> Result<()> {
        if trace.storage_access.is_empty() && 
           trace.delegatecall_storage.is_empty() && 
           trace.delegatecall_unknown.is_empty() && 
           trace.external_calls.is_empty() {
            return Err(ProxyError::DetectionFailed(
                "No relevant operations found in trace".into()
            ));
        }
        Ok(())
    }

    fn identify_proxy_by_storage(&self, storage: &U256) -> ProxyType {
        if let Some(proxy) = EIP_1967_DEFAULT_STORAGE.get(storage) {
            *proxy
        } else if *storage > U256::from(0x100) {
            ProxyType::EIP_1967_CUSTOM
        } else {
            ProxyType::EIP_897
        }
    }

    fn has_diamond_selector(&self, code: &Bytes) -> bool {
        find_bytes(code, &hex_literal::hex!("637a0ed627")).is_some()
    }

    fn has_diamond_storage_pattern(&self, code: &Bytes) -> bool {
        find_bytes(code, &DIAMOND_STANDARD_STORAGE_SLOT_LESSBYTES).is_some()
    }

    fn execute_trace(&self, code: &Bytes, input: &Bytes, config: &TraceConfig) -> Result<ProxyInspector> {
        let mut db = ProxyDetectDB::new(config.contract_address);
        db.install_contract(config.contract_address, code)
            .map_err(|e| ProxyError::DetectionFailed(
                format!("Failed to install contract: {}", e)
            ))?;

        let inspector = ProxyInspector::new();

        let mut evm = EvmBuilder::default()
            .with_db(db)
            .with_external_context(inspector.clone())
            .append_handler_register(inspector_handle_register)
            .modify_tx_env(|tx: &mut TxEnv| {
                tx.caller = config.caller_address;
                tx.transact_to = TransactTo::Call(config.contract_address);
                tx.data = input.clone();
                tx.value = U256::ZERO;
                tx.gas_limit = config.gas_limit;
            })
            .build();

        evm.transact().map_err(|e| ProxyError::DetectionFailed(
            format!("EVM execution failed: {}", e)
        ))?;
        
        Ok(inspector)
    }

    fn analyze_traces(&self, traces: Vec<ProxyInspector>) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        if traces.is_empty() {
            return Ok(None);
        }

        let consistent_execution = self.check_all_are_equal(&traces);
        let first_trace = &traces[0];

        if consistent_execution {
            self.analyze_consistent_trace(first_trace)
        } else {
            self.analyze_diamond_proxy(first_trace)
        }
    }

    fn analyze_consistent_trace(&self, trace: &ProxyInspector) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        if trace.delegatecall_unknown.len() == 1 {
            let static_address = trace.delegatecall_unknown[0];
            Ok(Some((ProxyType::StaticAddress, ProxyDispatch::Static(static_address))))
        } else if trace.delegatecall_storage.len() == 1 {
            let storage_slot = trace.delegatecall_storage[0];
            Ok(Some((
                self.identify_proxy_by_storage(&storage_slot),
                ProxyDispatch::Storage(storage_slot)
            )))
        } else if trace.external_calls.len() == 1 {
            let (address, fun) = trace.external_calls[0];
            if FUN_TO_PROXY.contains_key(&fun) {
                Ok(Some((ProxyType::External, ProxyDispatch::External(address, fun))))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn analyze_diamond_proxy(&self, trace: &ProxyInspector) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        if self.has_diamond_selector(&trace.code) {
            Ok(Some((ProxyType::EIP_2535, ProxyDispatch::Facet_EIP_2535)))
        } else if self.has_diamond_storage_pattern(&trace.code) {
            Ok(Some((ProxyType::EIP_2535, ProxyDispatch::FacetStorageSlot)))
        } else {
            Ok(Some((ProxyType::DiamondOther, ProxyDispatch::Unknown)))
        }
    }
}

impl DetectionStrategy for DynamicDetector {
    fn detect(&self, code: &Bytes) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        if code.is_empty() {
            return Ok(None);
        }

        let config = TraceConfig::default();
        let mut traces = Vec::new();
        
        for input in &self.test_inputs {
            let inspector = self.execute_trace(code, input, &config)
                .map_err(|e| ProxyError::DetectionFailed(
                    format!("Trace execution failed: {}", e)
                ))?;
            traces.push(inspector);
        }

        self.analyze_traces(traces)
    }

    fn name(&self) -> &'static str {
        "DynamicDetector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;

    #[test]
    fn test_empty_code() {
        let detector = DynamicDetector::default();
        assert!(detector.detect(&Bytes::new()).unwrap().is_none());
    }

    #[test]
    fn test_custom_test_inputs() {
        let inputs = vec![
            Bytes::from(vec![0x12, 0x34]),
            Bytes::from(vec![0x56, 0x78]),
        ];
        let detector = DynamicDetector::with_test_inputs(inputs.clone());
        assert_eq!(detector.test_inputs, inputs);
    }

    #[test]
    fn test_trace_config() {
        let config = TraceConfig::default();
        assert_eq!(config.gas_limit, 30_000_000);
        assert_ne!(config.contract_address, config.caller_address);
    }
}
