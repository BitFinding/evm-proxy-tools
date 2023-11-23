use std::collections::HashMap;

use ethers_core::types::H256;
use once_cell::sync::Lazy;
use revm_primitives::U256;

use crate::ProxyType;

pub static ADDR_MASK_H256: Lazy<H256> = Lazy::new(|| {
    H256::from(hex_literal::hex!("000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"))
});

pub static EIP_1967_DEFAULT_STORAGE: Lazy<HashMap<U256, ProxyType>> = Lazy::new(|| {
    [
            (U256::from_be_bytes(hex_literal::hex!("7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3")), ProxyType::EIP_1967_ZOS),
            (U256::from_be_bytes(hex_literal::hex!("360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc")), ProxyType::EIP_1967),
            (U256::from_be_bytes(hex_literal::hex!("a3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50")), ProxyType::EIP_1967_BEACON),
            (U256::from_be_bytes(hex_literal::hex!("c5f16f0fcc639fa48a6947836d9850f504798523bf8c9a3a87d5876cf622bcf7")), ProxyType::EIP_1822),
    ].into_iter().collect()
});

pub static DIAMOND_STANDARD_STORAGE_SLOT_LESSBYTES: Lazy<Vec<u8>> = Lazy::new(|| hex_literal::hex!("c8fcad8db84d3cc18b4c41d551ea0ee66dd599cde068d998e57d5e09332c13").to_vec());

// pub static DIAMOND_STANDARD_STORAGE_SLOT: Lazy<Vec<u8>> = Lazy::new(|| hex_literal::hex!("c8fcad8db84d3cc18b4c41d551ea0ee66dd599cde068d998e57d5e09332c13").to_vec());
pub static DIAMOND_STANDARD_STORAGE_SLOT: Lazy<U256> = Lazy::new(|| U256::from_be_bytes(hex_literal::hex!("c8fcad8db84d3cc18b4c41d551ea0ee66dd599cde068d998e57d5e09332c131b")));

pub static FUN_TO_PROXY: Lazy<HashMap<u32, ProxyType>> = Lazy::new(|| {
    [
	// facetAddress(bytes4)
	(0xcdffacc6, ProxyType::EIP_2535)
     ].into_iter().collect()
});
