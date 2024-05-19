use ethers_core::types::{H160 as eH160, U256 as eU256, H256 as eH256, NameOrAddress as eNameOrAddress};
use ethers_core::types::transaction::eip2930::AccessListItem;

use alloy_primitives::{Address as rAddress, U256 as rU256};

/// Ethers/Alloy/REVM trait to convert for types from one to another
pub trait EARGlue<To> {
    fn convert(&self) -> To;
}

impl EARGlue<eH256> for eU256 {
    #[inline(always)]
    fn convert(&self) -> eH256 {
        let mut h = eH256::default();
        self.to_big_endian(h.as_mut());
        h
    }
}

impl EARGlue<eU256> for eH256 {
    #[inline(always)]
    fn convert(&self) -> eU256 {
        eU256::from_big_endian(self.as_bytes())
    }
}

impl EARGlue<eH160> for rAddress {
    #[inline(always)]
    fn convert(&self) -> eH160 {
        eH160::from_slice(self.as_slice())
    }
}

impl EARGlue<rAddress> for eH160 {
    #[inline(always)]
    fn convert(&self) -> rAddress {
        rAddress::from_slice(self.as_fixed_bytes())
    }
}

// impl EARGlue<rAddress> for eNameOrAddress {
//     #[inline(always)]
//     fn convert(&self) -> rAddress {
//         rAddress::from_slice(self.as_address().unwrap().as_fixed_bytes())
//     }
// }

impl EARGlue<eNameOrAddress> for rAddress {
    #[inline(always)]
    fn convert(&self) -> eNameOrAddress {
        eNameOrAddress::Address(self.convert())
    }
}

/// Small helper function to convert [eU256] into [eH256].
#[inline(always)]
pub fn u256_to_h256_be(u: eU256) -> eH256 {
    let mut h = eH256::default();
    u.to_big_endian(h.as_mut());
    h
}

/// Small helper function to convert [eU256] into [eH256].
#[inline(always)]
pub fn ru256_to_h256_be(u: &rU256) -> eH256 {
    eH256::from(u.to_be_bytes())
}

/// Small helper function to convert [eH256] into [eU256].
#[inline(always)]
pub fn h256_to_u256_be(storage: eH256) -> eU256 {
    eU256::from_big_endian(storage.as_bytes())
}

/// Small helper function to convert ether's [eH256] into revm's [B256].
#[inline(always)]
pub fn h256_to_b256(h: eH256) -> alloy_primitives::B256 {
    alloy_primitives::B256::from_slice(h.as_bytes())
}

/// Small helper function to convert ether's [eU256] into revm's [eU256].
#[inline(always)]
pub fn u256_to_ru256(u: eU256) -> rU256 {
    let mut buffer = [0u8; 32];
    u.to_little_endian(buffer.as_mut_slice());
    rU256::from_le_bytes(buffer)
}

/// Small helper function to convert ethers's [H160] into revm's [B160].
#[inline(always)]
pub fn h160_to_b160(h: &eH160) -> alloy_primitives::Address {
    alloy_primitives::Address::from_slice(h.as_bytes())
}

#[inline(always)]
pub fn raddress_to_h160(ra: &rAddress) -> eH160 {
    eH160::from_slice(ra.as_slice())
}

pub fn to_revm_access_list(list: Vec<AccessListItem>) -> Vec<(rAddress, Vec<rU256>)> {
    list.into_iter()
        .map(|item| {
            (
                h160_to_b160(&item.address),
                item.storage_keys.into_iter().map(h256_to_u256_be).map(u256_to_ru256).collect(),
            )
        })
        .collect()
}

#[inline(always)]
pub fn h256_to_raddress_unchecked(h256: &eH256) -> rAddress {
    rAddress::from_slice(&h256.as_fixed_bytes()[12..])
}

#[inline(always)]
pub fn slice_as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}

#[inline(always)]
pub fn as_u32_be(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}

#[inline(always)]
pub fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}
