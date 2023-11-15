use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use ethers_contract::{abigen, EthAbiType};
use ethers_core::types::H160;
// use ethers_core::types::H256;
use ethers_providers::Middleware;
use futures::future::join_all;
use revm_primitives::{Address, U256};
use thiserror::Error;
use tracing::debug;

use crate::{types::{ProxyDispatch, ProxyType}, consts::{DIAMOND_STANDARD_STORAGE_SLOT, ADDR_MASK_H256}, utils::{u256_to_h256_be, ru256_to_h256_be, raddress_to_h160, h256_to_u256_be, h256_to_raddress_unchecked, as_u32_le, h160_to_b160}};

#[derive(Clone, Debug, Error)]
pub enum ProxyReadError {
    #[error("unknown proxy")]
    UnknownProxy,
    #[error("RPC error: `{0}`")]
    RPCError(String),
    #[error("the storage doesn't contain an address")]
    StorageNotAddress,
    #[error("proxy is implemented in a different address")]
    ExternalProxy,
    #[error("unknown data store error")]
    Unknown,
}

#[derive(Clone, Debug)]
pub enum ProxyImplementation {
    Single(Address),
    Multiple(Vec<Address>),
    Facets(HashMap<Address, u32>)
}

// #[derive(EthAbiType)]
// struct Facet {
//     facetAddress: H160,
//     functionSelectors: Vec<u32>
// }

abigen!(
    IDiamondLoupe, r"[
    struct Facet {address facetAddress; bytes4[] functionSelectors;}

    function facets() external view returns (Facet[])
]",
);

pub async fn read_single_storage_implementation<M>(rpc: &M, address: &Address, storage: &U256) -> Result<Address, ProxyReadError>
    where M: Middleware
{
    let h256_storage = ru256_to_h256_be(storage);
    let h256_value = rpc.get_storage_at(raddress_to_h160(address), h256_storage, None).await.map_err(|e| ProxyReadError::RPCError(e.to_string()))?;
    // let value = h256_to_u256_be(h256_value);

    debug!("stored value:: {:?}", h256_value);
    if (h256_value & *ADDR_MASK_H256) == h256_value {
	let stored_address = h256_to_raddress_unchecked(&h256_value);
	Ok(stored_address)
    } else {
	Err(ProxyReadError::StorageNotAddress)
    }
}

pub async fn read_facet_list_from_function<M>(rpc: Arc<M>, address: &Address) -> Result<ProxyImplementation, ProxyReadError>
where M: Middleware + 'static
{
    let address = raddress_to_h160(address);
    let contract = IDiamondLoupe::new(address, rpc);
    let facets = contract.facets().await.map_err(|e| ProxyReadError::RPCError(e.to_string()))?;
    let facets_hashmap: HashMap<Address, u32> = facets.iter().map(|v| {
	v.1.iter().map(|v1| (h160_to_b160(&v.0), as_u32_le(v1)))
    }).flatten().collect();
    Ok(ProxyImplementation::Facets(facets_hashmap))
}

pub async fn read_diamond_implementation<M>(rpc: &M, address: &Address, diamond_base: &U256) -> Result<ProxyImplementation, ProxyReadError>
    where M: Middleware
{
    // Scan storage to find the first array (should have its size)


    // Go to the base of the array and get the structs


    // For each struct read the arrays of function signatures
    todo!()
}

#[async_recursion]
pub async fn get_proxy_implementation<M>(rpc: Arc<M>, address: &Address, proxy_dispatch: &ProxyDispatch) -> Result<ProxyImplementation, ProxyReadError>
    where M: Middleware + 'static
{
    match proxy_dispatch {
        ProxyDispatch::Unknown => Err(ProxyReadError::UnknownProxy),
        ProxyDispatch::Storage(slot) => Ok(ProxyImplementation::Single(read_single_storage_implementation(&rpc, address, slot).await?)),
        ProxyDispatch::MultipleStorage(slots) => {
	    let addrs: Result<Vec<Address>, ProxyReadError> = join_all(slots.iter().map(|s| async { read_single_storage_implementation(&rpc, address, s).await })).await.into_iter().collect();
	    Ok(ProxyImplementation::Multiple(addrs?))
	},
        ProxyDispatch::Static(address) => Ok(ProxyImplementation::Single(address.clone())),
        ProxyDispatch::Facet_EIP_2535 => { Ok(read_facet_list_from_function(rpc, address).await?) },
        ProxyDispatch::FacetStorageSlot => Ok(read_diamond_implementation(&rpc, address, &DIAMOND_STANDARD_STORAGE_SLOT).await?),
        ProxyDispatch::External(_, _) => Err(ProxyReadError::ExternalProxy)
        // ProxyDispatch::External(address, dispatch) => Ok(get_proxy_implementation(rpc, address, dispatch).await?),
    }
}
