
use crate::consts::{EIP_1967_DEFAULT_STORAGE, DIAMOND_STANDARD_STORAGE_SLOT_LESSBYTES, FUN_TO_PROXY};
// use hardfork::Hardfork;
use crate::proxy_inspector::{ProxyInspector, ProxyDetectDB, InspectorData};
use once_cell::sync::Lazy;
use revm::{Context, context::TxEnv, primitives::TxKind, MainContext, MainBuilder, InspectEvm};
use alloy_primitives::{Address, Bytes, U256};
use tracing::debug;
use twoway::find_bytes;

use crate::{ProxyType, Dispatch};

pub trait ProxyDetector {
    fn try_match(code: &[u8]) -> Option<(ProxyType, Dispatch)>;
}

pub struct MinimalProxy {}

#[inline(always)]
pub fn extract_minimal_contract<const ADDR_SIZE: usize>(code: &[u8], min_size: usize, first_part: &[u8], second_part: &[u8]) -> Option<Address> {
    let second_start = first_part.len() + ADDR_SIZE;
    if code.len() >= min_size && &code[0..first_part.len()] == first_part && &code[second_start..second_start + second_part.len()] == second_part {
	let addr = &code[first_part.len()..second_start];
	if ADDR_SIZE == 16 {
	    let mut addr_vec = vec![0; 20];
	    addr_vec[4..].clone_from_slice(addr);
	    Some(Address::from_slice(&addr_vec))
	    // Some
	} else {
	    Some(Address::from_slice(addr))
	}
    } else {
	None
    }
}

impl MinimalProxy {
    fn is_eip_1667_long(code: &[u8]) -> Option<Address> {
	const EIP_1667_FIRST_BYTES: &[u8] = &hex_literal::hex!("363d3d373d3d3d363d73");
	const EIP_1667_SECOND_BYTES: &[u8] = &hex_literal::hex!("5af43d82803e903d91602b57fd5bf3");

	extract_minimal_contract::<20>(code, 45, EIP_1667_FIRST_BYTES, EIP_1667_SECOND_BYTES)
    }

    fn is_eip_1667_short(code: &[u8]) -> Option<Address> {
	const EIP_1667_FIRST_BYTES: &[u8] = &hex_literal::hex!("363d3d373d3d3d363d6f");
	const EIP_1667_SECOND_BYTES: &[u8] = &hex_literal::hex!("5af43d82803e903d91602b57fd5bf3");

	extract_minimal_contract::<16>(code, 41, EIP_1667_FIRST_BYTES, EIP_1667_SECOND_BYTES)
    }

    fn is_eip_7511_long(code: &[u8]) -> Option<Address> {
	const EIP_7511_FIRST_BYTES: &[u8] = &hex_literal::hex!("365f5f375f5f365f73");
	const EIP_7511_SECOND_BYTES: &[u8] = &hex_literal::hex!("5af43d5f5f3e5f3d91602a57fd5bf3");

	extract_minimal_contract::<20>(code, 44, EIP_7511_FIRST_BYTES, EIP_7511_SECOND_BYTES)
    }

    fn is_eip_7511_short(code: &[u8]) -> Option<Address> {
	const EIP_7511_FIRST_BYTES: &[u8] = &hex_literal::hex!("365f5f375f5f365f6f");
	const EIP_7511_SECOND_BYTES: &[u8] = &hex_literal::hex!("5af43d5f5f3e5f3d91602a57fd5bf3");

	extract_minimal_contract::<16>(code, 40, EIP_7511_FIRST_BYTES, EIP_7511_SECOND_BYTES)
    }

    fn is_eip_3448_long(code: &[u8]) -> Option<Address> {
	const EIP_3448_FIRST_BYTES: &[u8] = &hex_literal::hex!("363d3d373d3d3d3d60368038038091363936013d73");
	const EIP_3448_SECOND_BYTES: &[u8] = &hex_literal::hex!("5af43d3d93803e603457fd5bf3");

	extract_minimal_contract::<20>(code, 44, EIP_3448_FIRST_BYTES, EIP_3448_SECOND_BYTES)
    }

    fn is_eip_3448_short(code: &[u8]) -> Option<Address> {
	const EIP_3448_FIRST_BYTES: &[u8] = &hex_literal::hex!("363d3d373d3d3d3d60368038038091363936013d6f");
	const EIP_3448_SECOND_BYTES: &[u8] = &hex_literal::hex!("5af43d3d93803e603457fd5bf3");

	extract_minimal_contract::<16>(code, 44, EIP_3448_FIRST_BYTES, EIP_3448_SECOND_BYTES)
    }

    fn is_eip_3448(code: &[u8]) -> Option<Address> {
	Self::is_eip_3448_long(code).or_else(|| Self::is_eip_3448_short(code))
    }

    fn is_eip_7511(code: &[u8]) -> Option<Address> {
	Self::is_eip_7511_long(code).or_else(|| Self::is_eip_7511_short(code))
    }

    fn is_eip_1667(code: &[u8]) -> Option<Address> {
	Self::is_eip_1667_long(code).or_else(|| Self::is_eip_1667_short(code))
    }

}

impl ProxyDetector for  MinimalProxy {
    fn try_match(code: &[u8]) -> Option<(ProxyType, Dispatch)> {
	if let Some(address) = Self::is_eip_1667(code) {
	    Some((ProxyType::Eip1167, Dispatch::Static(address)))
	} else if let Some(address) = Self::is_eip_7511(code) {
	    Some((ProxyType::Eip7511, Dispatch::Static(address)))
	} else {
	    Self::is_eip_3448(code).map(|address| (ProxyType::Eip3448, Dispatch::Static(address)))
	}
    }
}

struct StorageSlotProxy {}

impl StorageSlotProxy {

}


struct StorageCallTaint {
    code: Bytes,
    address: Address
}

static DEFAULT_CALLER_ADDRESS: Lazy<Address> = Lazy::new(|| hex_literal::hex!("11ff0000ff0000ff0000ff0000ff0000ff0000ff").into());
static DEFAULT_CONTRACT_ADDRESS: Lazy<Address> = Lazy::new(|| hex_literal::hex!("00ff0000ff0000ff0000ff0000ff0000ff0000ff").into());


impl StorageCallTaint {

    pub fn new_with_info(code: &[u8], address: Address, _caller: Address) -> Self {
	Self {
	    code: Bytes::copy_from_slice(code),
	    address
	}
    }

    pub fn new(code: &[u8]) -> Self {
	Self::new_with_info(code, *DEFAULT_CONTRACT_ADDRESS, *DEFAULT_CALLER_ADDRESS)
    }

    pub fn trace_calldata(&self, calldata: Bytes) -> InspectorData {

	// init revm
	let mut db = ProxyDetectDB::new(self.address);
	db.install_contract(self.address, &self.code);

	let inspector = ProxyInspector::new();

        // Build EVM with the new revm 33 API
        let tx = TxEnv::builder()
            .kind(TxKind::Call(self.address))
            .data(calldata)
            .value(U256::ZERO)
            .gas_limit(30_000_000)
            .build()
            .expect("Failed to build TxEnv");

        let mut evm = Context::mainnet()
            .with_db(db)
            .build_mainnet_with_inspector(inspector);

        let _res = evm.inspect_one_tx(tx);
        
        // Get the inspector from the EVM and collect results
        evm.inspector.collect()
    }

    fn identify_proxy_by_storage(storage: &U256) -> ProxyType {
	if let Some(proxy) = EIP_1967_DEFAULT_STORAGE.get(storage) {
	    *proxy
	} else if *storage > U256::from(0x100) {
	    ProxyType::Eip1967Custom
	} else {
	    ProxyType::Eip897
	}
    }

    fn check_all_are_equal(data: &[InspectorData]) -> bool {
	let first = &data[0];
	data.iter().all(|e| e == first)
    }

    fn detect_proxy_from_data(&self, data: &[InspectorData]) -> Option<(ProxyType, Dispatch)> {
	debug!("inspector_data: {:#?}", data);

	let consistent_execution = Self::check_all_are_equal(data);
	if consistent_execution {
	    if data[0].delegatecall_unknown.len() == 1 {
		let static_address = data[0].delegatecall_unknown[0];
		Some((ProxyType::StaticAddress, Dispatch::Static(static_address)))
	    }  else if data[0].delegatecall_storage.len() == 1{
		let storage_slot = data[0].delegatecall_storage[0];
		Some((Self::identify_proxy_by_storage(&storage_slot), Dispatch::Storage(storage_slot)))
	    } else if data[0].external_calls.len() ==1 {
		let address = data[0].external_calls[0].0;
		let fun = data[0].external_calls[0].1;
		if FUN_TO_PROXY.contains_key(&fun) {
		    Some((ProxyType::External, Dispatch::External(address, fun)))
		} else {
		    None
		}
	    } else {
		None
	    }
	} else if find_bytes(&self.code, &hex_literal::hex!("637a0ed627")).is_some() {
	    Some((ProxyType::Eip2535, Dispatch::DiamondFacets))
	} else if find_bytes(&self.code, &DIAMOND_STANDARD_STORAGE_SLOT_LESSBYTES).is_some() {
	    Some((ProxyType::Eip2535, Dispatch::DiamondStorage))
	} else {
	    Some((ProxyType::DiamondOther, Dispatch::Unknown))
	}
    }

    fn get_proxy(&self) -> Option<(ProxyType, Dispatch)> {
	// Run with 3 different call data to check if we get different DelegateCall
	let mut runs = Vec::new();

	let calldata_detectors = vec![
	    vec![0xaa, 0xcc, 0xbb, 0xdd],
	    vec![0xcc, 0xbb, 0xdd, 0xf1, 0xf1, 0xf1, 0xf1, 0xf1, 0xf1, 0xf1],
	    vec![0x01, 0x02, 0x04, 0x11]
	];
	for calldata in calldata_detectors {
	    let ret = self.trace_calldata(calldata.into());
	    runs.push(ret);
	}
	self.detect_proxy_from_data(&runs)
    }
}


impl ProxyDetector for StorageSlotProxy {
    fn try_match(code: &[u8]) -> Option<(ProxyType, Dispatch)> {
        let tainter = StorageCallTaint::new(code);
	tainter.get_proxy()
    }
}

pub fn get_proxy_type(code: &[u8]) -> Option<(ProxyType, Dispatch)> {
    MinimalProxy::try_match(code).or_else(|| StorageSlotProxy::try_match(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_proxy() {
        assert_eq!(MinimalProxy::is_eip_1667(&hex_literal::hex!("363d3d373d3d3d363d73bebebebebebebebebebebebebebebebebebebebe5af43d82803e903d91602b57fd5bf3")), Some(Address::from(hex_literal::hex!("bebebebebebebebebebebebebebebebebebebebe"))));
        assert_eq!(MinimalProxy::is_eip_1667(&hex_literal::hex!("363d3d373d3d3d363d6fbebebebebebebebebebebebebebebebe5af43d82803e903d91602b57fd5bf3")), Some(Address::from(hex_literal::hex!("00000000bebebebebebebebebebebebebebebebe"))));
        assert_eq!(MinimalProxy::is_eip_1667(&hex_literal::hex!("9999999999")), None);
        assert_eq!(MinimalProxy::is_eip_1667(&hex_literal::hex!("9999999999aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")), None);

	assert_eq!(MinimalProxy::is_eip_7511(&hex_literal::hex!("365f5f375f5f365f73bebebebebebebebebebebebebebebebebebebebe5af43d5f5f3e5f3d91602a57fd5bf3")), Some(Address::from(hex_literal::hex!("bebebebebebebebebebebebebebebebebebebebe"))));
        assert_eq!(MinimalProxy::is_eip_7511(&hex_literal::hex!("365f5f375f5f365f6fbebebebebebebebebebebebebebebebe5af43d5f5f3e5f3d91602a57fd5bf3")), Some(Address::from(hex_literal::hex!("00000000bebebebebebebebebebebebebebebebe"))));
        assert_eq!(MinimalProxy::is_eip_7511(&hex_literal::hex!("9999999999")), None);
        assert_eq!(MinimalProxy::is_eip_7511(&hex_literal::hex!("9999999999aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")), None);

	assert_eq!(MinimalProxy::is_eip_3448(&hex_literal::hex!("363d3d373d3d3d3d60368038038091363936013d73bebebebebebebebebebebebebebebebebebebebe5af43d3d93803e603457fd5bf3")), Some(Address::from(hex_literal::hex!("bebebebebebebebebebebebebebebebebebebebe"))));
        assert_eq!(MinimalProxy::is_eip_3448(&hex_literal::hex!("363d3d373d3d3d3d60368038038091363936013d6fbebebebebebebebebebebebebebebebe5af43d3d93803e603457fd5bf3")), Some(Address::from(hex_literal::hex!("00000000bebebebebebebebebebebebebebebebe"))));
        assert_eq!(MinimalProxy::is_eip_3448(&hex_literal::hex!("9999999999")), None);
        assert_eq!(MinimalProxy::is_eip_3448(&hex_literal::hex!("9999999999aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")), None);
    }
}
