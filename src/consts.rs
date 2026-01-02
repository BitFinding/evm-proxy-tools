use std::collections::HashMap;

use once_cell::sync::Lazy;
use alloy_primitives::U256;

use crate::ProxyType;

pub static ADDR_MASK_U256: Lazy<U256> = Lazy::new(|| {
    U256::from_be_bytes(hex_literal::hex!("000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"))
});

pub static EIP_1967_DEFAULT_STORAGE: Lazy<HashMap<U256, ProxyType>> = Lazy::new(|| {
    [
            (U256::from_be_bytes(hex_literal::hex!("7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3")), ProxyType::Eip1967Zos),
            (U256::from_be_bytes(hex_literal::hex!("360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc")), ProxyType::Eip1967),
            (U256::from_be_bytes(hex_literal::hex!("a3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50")), ProxyType::Eip1967Beacon),
            (U256::from_be_bytes(hex_literal::hex!("c5f16f0fcc639fa48a6947836d9850f504798523bf8c9a3a87d5876cf622bcf7")), ProxyType::Eip1822),
    ].into_iter().collect()
});

pub static DIAMOND_STANDARD_STORAGE_SLOT_LESSBYTES: Lazy<Vec<u8>> = Lazy::new(|| hex_literal::hex!("c8fcad8db84d3cc18b4c41d551ea0ee66dd599cde068d998e57d5e09332c13").to_vec());

pub static DIAMOND_STANDARD_STORAGE_SLOT: Lazy<U256> = Lazy::new(|| U256::from_be_bytes(hex_literal::hex!("c8fcad8db84d3cc18b4c41d551ea0ee66dd599cde068d998e57d5e09332c131b")));

pub static FUN_TO_PROXY: Lazy<HashMap<u32, ProxyType>> = Lazy::new(|| {
    [
	(0xcdffacc6, ProxyType::Eip2535)
     ].into_iter().collect()
});

pub static GNOSIS_SAFE_STORAGE_SLOT: Lazy<U256> = Lazy::new(|| U256::ZERO);

pub static COMPOUND_UNITROLLER_STORAGE_SLOT: Lazy<U256> = Lazy::new(|| U256::from(2));

pub const GNOSIS_SAFE_MASTERCOPY_SELECTOR: [u8; 4] = hex_literal::hex!("a619486e");

pub const ZERO_AGE_FIRST: &[u8] = &hex_literal::hex!("3d3d3d3d363d3d37363d73");
pub const ZERO_AGE_SECOND: &[u8] = &hex_literal::hex!("5af43d3d93803e602a57fd5bf3");

pub const SOLADY_PUSH0_FIRST: &[u8] = &hex_literal::hex!("5f5f365f5f37365f73");
pub const SOLADY_PUSH0_SECOND: &[u8] = &hex_literal::hex!("5af43d5f5f3e6029573d5ffd5b3d5ff3");

pub const VYPER_BETA_FIRST: &[u8] = &hex_literal::hex!("366000600037611000600036600073");
pub const VYPER_BETA_SECOND: &[u8] = &hex_literal::hex!("5af41558576110006000f3");

pub const SEQUENCE_WALLET_BYTECODE: &[u8] = &hex_literal::hex!("363d3d373d3d3d363d30545af43d82803e903d91601857fd5bf3");

pub const ZERO_X_SPLITS_FIRST: &[u8] = &hex_literal::hex!("36603057343d52307f830d2d700a97af574b186c80d40429385d24241565b08a7c559ba283a964d9b160203da23d3df35b3d3d3d3d363d3d37363d73");
pub const ZERO_X_SPLITS_SECOND: &[u8] = &hex_literal::hex!("5af43d3d93803e605b57fd5bf3");

pub const CWIA_FIRST: &[u8] = &hex_literal::hex!("3d3d3d3d363d3d3761");
pub const CWIA_SECOND: &[u8] = &hex_literal::hex!("5af43d3d93803e6057fd5bf3");

pub const EIP_6551_SIZE: usize = 173;
pub const EIP_1167_SIZE: usize = 45;

