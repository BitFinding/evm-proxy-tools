use std::{collections::{HashSet, HashMap}, rc::Rc, ops::{BitAnd, BitXor}};
use num_traits::cast::ToPrimitive;

use once_cell::sync::Lazy;
use revm::{
    interpreter::{opcode, CallInputs, CallScheme, CreateInputs, Gas, InstructionResult, Interpreter}, primitives::{AccountInfo, Bytecode}, Database, EvmContext, Inspector
};

use alloy_primitives::{
    Bytes,
    Address, U256, B256, FixedBytes,
};

use revm_interpreter::{CallOutcome, InterpreterResult, OpCode};
use thiserror::Error;
use tracing::debug;

use crate::utils::{as_u32_be, slice_as_u32_be};

type StorageCall = HashSet<Bytes>;

/// The collected results of [`InspectorStack`].
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorData {
    pub storage_access: Vec<U256>,
    pub delegatecall_storage: Vec<U256>,
    pub delegatecall_unknown: Vec<Address>,
    pub external_calls: Vec<(Address, u32)>
}

/// An inspector that calls multiple inspectors in sequence.
///
/// If a call to an inspector returns a value other than [InstructionResult::Continue] (or
/// equivalent) the remaining inspectors are not called.
#[derive(Debug, Default)]
pub struct ProxyInspector {
    storage_access: Vec<U256>,
    delegatecall_storage: Vec<U256>,
    delegatecall_unknown: Vec<Address>,
    external_calls: Vec<(Address, u32)>
}

impl ProxyInspector {
    /// Creates a new inspector stack.
    ///
    /// Note that the stack is empty by default, and you must add inspectors to it.
    /// This is done by calling the `set_*` methods on the stack directly, or by building the stack
    /// with [`InspectorStack`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Collects all the data gathered during inspection into a single struct.
    #[inline]
    pub fn collect(self) -> InspectorData {
        InspectorData {
	    storage_access: self.storage_access,
            delegatecall_storage: self.delegatecall_storage,
            delegatecall_unknown: self.delegatecall_unknown,
            external_calls: self.external_calls,
        }
    }

}

// enum TaintDetail {
//     // Variables embedded in the code, minimal proxies and others
//     CodeData(u16, u16),
//     CallData(u16, u16),
//     Storage(Rc<TaintInfo>),
//     Static
// }

// struct TaintInfo {
//     taint_detail: TaintDetail,
//     clean_taint: bool
// }

// struct Tainter {
//     memory: Vec<(U256, TaintInfo)>,
//     stack: Vec<(U256, TaintInfo)>
// }

static ADDR_MASK: Lazy<U256> = Lazy::new(|| U256::from_be_bytes(hex_literal::hex!("000000000000000000000000ffffffffffffffffffffffffffffffffffffffff")));
static ADDR_XOR: Lazy<U256> = Lazy::new(|| U256::from_be_bytes(hex_literal::hex!("000000000000000000000000c1d50e94dbe44a2e3595f7d5311d788076ac6188")));

#[derive(Clone, Debug, Error)]
pub enum ProxyDetectError {

}

pub struct ProxyDetectDB {
    contract_address: Address,
    code: HashMap<Address, Bytes>,
    values_to_storage: HashMap<Address, U256>,
    delegatecalls: Vec<Address>
}


impl ProxyDetectDB {
    pub fn new(contract_address: Address) -> Self {
	Self {
            contract_address,
	    code: HashMap::new(),
	    values_to_storage: HashMap::new(),
            delegatecalls: Vec::new()
	}
    }

    pub fn install_contract(&mut self, address: Address, code: &Bytes) {
	self.code.insert(address, code.clone());
    }

    fn insert_delegatecall(&mut self, contract: Address) {
        self.delegatecalls.push(contract);
    }
}

impl Database for ProxyDetectDB {
    type Error = ProxyDetectError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo> ,Self::Error>  {
        debug!("basic(): addr: {:?}", address);
	if let Some(code) = self.code.get(&address) {
	    Ok(Some(
		AccountInfo {
		    balance: U256::ZERO,
		    nonce: 0,
		    code_hash: B256::ZERO,
		    code: Some(Bytecode::new_raw(code.clone())),
		}
	    ))
	} else if address == Address::ZERO {
	    // Return empty account for null, revm asks for it
	    Ok(None)
	} else {
	    Ok(Some(
		AccountInfo {
		    balance: U256::ZERO,
		    nonce: 0,
		    code_hash: B256::ZERO,
		    // Let's give it some code
		    code: Some(Bytecode::new_raw(Bytes::copy_from_slice(&[0xcc, 0xaa, 0xdd, 0xbb]))),
		}
	    ))
	}
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode,Self::Error>  {
        // println!("code_by_hash(): {}", code_hash);
	todo!()
    }

    fn storage(&mut self, address: Address,index: U256) -> Result<U256,Self::Error>  {
        let magic_value = index.bitand(*ADDR_MASK).bitxor(*ADDR_XOR);
	let magic_address = Address::from_word(FixedBytes::from_slice(&magic_value.to_be_bytes::<32>()));
	debug!("storage(): {:x} -> {:x} = {:x}", address, index, magic_value);

        self.values_to_storage.insert(magic_address, index);
	Ok(magic_value)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256,Self::Error>  {
	// println!("block_hash(): {}", number);
        todo!()
    }
}


impl Inspector<ProxyDetectDB> for ProxyInspector {

    #[inline(always)]
    fn step(
        &mut self,
        interpreter: &mut Interpreter,
        context: &mut EvmContext<ProxyDetectDB>,
    ) {
        // debug!("addr: {}", interpreter.contract.address);
        // debug!("opcode: {}", interpreter.current_opcode());
        let opcode = OpCode::new(interpreter.current_opcode()).unwrap();
        debug!("opcode: {}", opcode);
        for mem in interpreter.stack().data() {
            debug!("STACK: {:x}", mem);
        }
        debug!("--");
        match interpreter.current_opcode() {
            opcode::SLOAD => {
                if let Ok(memory) = interpreter.stack.peek(0) {
		    self.storage_access.push(memory);
                    debug!("SLOAD detected {}", memory);
                }
            },
            _ => ()
        };
    }

    #[inline(always)]
    fn call(
        &mut self,
        context: &mut EvmContext<ProxyDetectDB>,
        call: &mut CallInputs,
    ) -> Option<CallOutcome> {
        // println!("call!!! {:?} {}", call.scheme, call.target_address);
        // return (InstructionResult::Continue, Gas::new(call.gas_limit), Bytes::new());
        if call.scheme == CallScheme::Call && call.target_address == context.db.contract_address {
            return None;
        }
	match call.scheme {
	    CallScheme::DelegateCall => {
		context.db.delegatecalls.push(call.bytecode_address);
		if let Some(storage) = context.db.values_to_storage.get(&call.bytecode_address) {
                    self.delegatecall_storage.push(*storage);
		} else {
                    self.delegatecall_unknown.push(call.bytecode_address);
		}
		context.db.insert_delegatecall(call.bytecode_address);
            },
	    CallScheme::Call | CallScheme::CallCode | CallScheme::StaticCall => {
		if call.input.len() >= 4 {
		    let fun = slice_as_u32_be(&call.input);
		    self.external_calls.push((call.target_address, fun));
		    debug!("external call detected {:x}: {:x}", call.target_address, fun);
		}

	    }
	};
        Some(CallOutcome { result: InterpreterResult { result: InstructionResult::Return, output: Bytes::new(), gas: Gas::new(call.gas_limit) }, memory_offset: 0..0 })
    }
}
