use std::collections::HashMap;

use once_cell::sync::Lazy;
use revm::{
    interpreter::{
        CallInputs, CallOutcome, CallScheme, Gas, InstructionResult, 
        Interpreter, InterpreterResult, InterpreterTypes,
        interpreter_types::{Jumps, StackTr},
    },
    state::{AccountInfo, Bytecode},
    database_interface::DBErrorMarker,
    Database, Inspector, Context,
    bytecode::opcode,
    context::JournalTr,
};

use alloy_primitives::{
    Bytes,
    Address, U256, B256, FixedBytes,
};

use thiserror::Error;
use tracing::{debug, trace};

use crate::utils::slice_as_u32_be;

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
    #[error("Custom error: {0}")]
    Custom(String),
}

impl DBErrorMarker for ProxyDetectError {}

pub struct ProxyDetectDB {
    pub contract_address: Address,
    code: HashMap<Address, Bytes>,
    pub values_to_storage: HashMap<Address, U256>,
    pub delegatecalls: Vec<Address>
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

    pub fn insert_delegatecall(&mut self, contract: Address) {
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

    fn code_by_hash(&mut self, _code_hash: B256) -> Result<Bytecode,Self::Error>  {
        // println!("code_by_hash(): {}", code_hash);
	todo!()
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256,Self::Error>  {
        use std::ops::{BitAnd, BitXor};
        let magic_value = index.bitand(*ADDR_MASK).bitxor(*ADDR_XOR);
	let magic_address = Address::from_word(FixedBytes::from_slice(&magic_value.to_be_bytes::<32>()));
	debug!("storage(): {:x} -> {:x} = {:x}", address, index, magic_value);

        self.values_to_storage.insert(magic_address, index);
	Ok(magic_value)
    }

    fn block_hash(&mut self, _number: u64) -> Result<B256,Self::Error>  {
	// println!("block_hash(): {}", number);
        todo!()
    }
}

impl<CTX, INTR> Inspector<CTX, INTR> for ProxyInspector 
where 
    CTX: ProxyDetectDBAccess,
    INTR: InterpreterTypes,
    INTR::Bytecode: Jumps,
    INTR::Stack: StackTr,
{
    #[inline(always)]
    fn step(
        &mut self,
        interp: &mut Interpreter<INTR>,
        _context: &mut CTX,
    ) {
        // debug!("addr: {}", interpreter.contract.address);
        let op = interp.bytecode.opcode();
        trace!("opcode: {}", revm::bytecode::opcode::OpCode::new(op).unwrap());
        for mem in interp.stack.data() {
            trace!("STACK: {:x}", mem);
        }
        trace!("--");
        match op {
            opcode::SLOAD => {
                // Try to get stack value at position 0
                let stack_data = interp.stack.data();
                if !stack_data.is_empty() {
                    let memory = stack_data[stack_data.len() - 1];
		    self.storage_access.push(memory);
                    trace!("SLOAD detected {}", memory);
                }
            },
            _ => ()
        };
    }

    #[inline(always)]
    fn call(
        &mut self,
        context: &mut CTX,
        call: &mut CallInputs,
    ) -> Option<CallOutcome> {
        // println!("call!!! {:?} {}", call.scheme, call.target_address);
        let db = context.get_proxy_detect_db();
        if call.scheme == CallScheme::Call && call.target_address == db.contract_address {
            return None;
        }
        
        // Get the input bytes for function selector extraction
        let input_bytes: Bytes = match &call.input {
            revm::interpreter::CallInput::Bytes(bytes) => bytes.clone(),
            revm::interpreter::CallInput::SharedBuffer(_) => {
                // For shared buffer, we can't easily access the bytes without context
                // Just use empty bytes as fallback
                Bytes::new()
            }
        };
        
	match call.scheme {
	    CallScheme::DelegateCall => {
		db.delegatecalls.push(call.bytecode_address);
		if let Some(storage) = db.values_to_storage.get(&call.bytecode_address) {
                    self.delegatecall_storage.push(*storage);
		} else {
                    self.delegatecall_unknown.push(call.bytecode_address);
		}
		db.insert_delegatecall(call.bytecode_address);
            },
	    CallScheme::Call | CallScheme::CallCode | CallScheme::StaticCall => {
		if input_bytes.len() >= 4 {
		    let fun = slice_as_u32_be(&input_bytes);
		    self.external_calls.push((call.target_address, fun));
		    debug!("external call detected {:x}: {:x}", call.target_address, fun);
		}
	    }
	};
        Some(CallOutcome::new(
            InterpreterResult { 
                result: InstructionResult::Return, 
                output: Bytes::new(), 
                gas: Gas::new(call.gas_limit) 
            }, 
            0..0
        ))
    }
}

/// Trait to access the ProxyDetectDB from the context.
/// This is needed because we need access to the DB in the Inspector implementation.
pub trait ProxyDetectDBAccess {
    fn get_proxy_detect_db(&mut self) -> &mut ProxyDetectDB;
}

// Implement for the Context type that revm uses
impl<BLOCK, TX, CFG, JOURNAL> ProxyDetectDBAccess for Context<BLOCK, TX, CFG, ProxyDetectDB, JOURNAL> 
where 
    JOURNAL: JournalTr<Database = ProxyDetectDB>,
{
    fn get_proxy_detect_db(&mut self) -> &mut ProxyDetectDB {
        self.journaled_state.db_mut()
    }
}
