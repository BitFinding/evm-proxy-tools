use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use ethers_contract::abigen;
use ethers_core::types::{BlockId, BlockNumber};
// use ethers_core::types::H256;
use ethers_providers::Middleware;
use futures::future::join_all;
use alloy_primitives::{Address, U256};
use thiserror::Error;
use tracing::debug;

use crate::{types::ProxyDispatch, consts::{DIAMOND_STANDARD_STORAGE_SLOT, ADDR_MASK_H256}, utils::{ru256_to_h256_be, raddress_to_h160, h256_to_raddress_unchecked, as_u32_le, h160_to_b160}};

// Remove this enum as we're using the centralized ProxyError now

#[derive(Clone, Debug)]
pub enum ProxyImplementation {
    Single(Address),
    Multiple(Vec<Address>),
    Facets(HashMap<Address, u32>)
}

impl ProxyImplementation {
    pub fn to_vec(&self) -> Vec<Address> {
        match self {
            ProxyImplementation::Single(addr) => vec![addr.clone()],
            ProxyImplementation::Multiple(addrs) => addrs.to_owned(),
            ProxyImplementation::Facets(addrs) => addrs.iter().map(|(k, _v)| k.clone()).collect(),
        }
    }
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

pub async fn read_single_storage_implementation<M>(rpc: &M, address: &Address, storage: &U256, block_number: Option<u64>) -> Result<Address, ProxyReadError>
    where M: Middleware
{
    let h256_storage = ru256_to_h256_be(storage);
    let block = block_number.map(|b| b.into());
    let h256_value = rpc.get_storage_at(raddress_to_h160(address), h256_storage, block).await.map_err(|e| ProxyReadError::RPCError(e.to_string()))?;
    // let value = h256_to_u256_be(h256_value);

    debug!("stored value:: {:?}", h256_value);
    if (h256_value & *ADDR_MASK_H256) == h256_value {
	let stored_address = h256_to_raddress_unchecked(&h256_value);
	Ok(stored_address)
    } else {
	Err(ProxyReadError::StorageNotAddress)
    }
}

pub async fn read_facet_list_from_function<M>(rpc: Arc<M>, address: &Address, block_number: Option<u64>) -> Result<ProxyImplementation, ProxyReadError>
where M: Middleware + 'static
{
    let address = raddress_to_h160(address);
    let contract = IDiamondLoupe::new(address, rpc);
    let block: BlockId = BlockId::Number(block_number.map(|b| b.into()).unwrap_or(BlockNumber::Latest));
    let facets = contract.facets().block(block).await.map_err(|e| ProxyReadError::RPCError(e.to_string()))?;
    let facets_hashmap: HashMap<Address, u32> = facets.iter().map(|v| {
	v.1.iter().map(|v1| (h160_to_b160(&v.0), as_u32_le(v1)))
    }).flatten().collect();
    Ok(ProxyImplementation::Facets(facets_hashmap))
}

pub async fn read_diamond_implementation<M>(_rpc: &M, _address: &Address, _diamond_base: &U256, _block_number: Option<u64>) -> Result<ProxyImplementation, ProxyReadError>
    where M: Middleware
{
    // TODO: implement properly
    return Ok(ProxyImplementation::Multiple(Vec::new()))
    // Scan storage to find the first array (should have its size)


    // Go to the base of the array and get the structs


    // For each struct read the arrays of function signatures
}

#[async_recursion]
/// Retrieves the implementation address(es) for a proxy contract
///
/// This function resolves the actual implementation contract(s) for a proxy based on its
/// dispatch mechanism. It supports various proxy patterns including:
/// - Static address proxies
/// - Storage-based proxies (EIP-1967)
/// - Diamond proxies (EIP-2535)
///
/// # Arguments
///
/// * `rpc` - The RPC client for interacting with the blockchain
/// * `address` - The proxy contract's address
/// * `proxy_dispatch` - The proxy's dispatch mechanism
/// * `block_number` - Optional block number for historical queries
///
/// # Returns
///
/// Returns a `ProxyImplementation` containing the implementation address(es)
///
/// # Example
///
/// ```no_run
/// use evm_proxy_tools::{get_proxy_implementation, ProxyDispatch, Result};
/// use std::sync::Arc;
///
/// async fn example<M>(client: Arc<M>, address: Address, dispatch: ProxyDispatch) -> Result<()> {
///     let implementation = get_proxy_implementation(client, &address, &dispatch, None).await?;
///     println!("Implementation: {:?}", implementation);
///     Ok(())
/// }
/// ```
pub async fn get_proxy_implementation<M>(
    rpc: Arc<M>,
    address: &Address,
    proxy_dispatch: &ProxyDispatch,
    block_number: Option<u64>
) -> Result<ProxyImplementation>
    where M: Middleware + 'static
{
    match proxy_dispatch {
        ProxyDispatch::Unknown => Err(ProxyReadError::UnknownProxy),
        ProxyDispatch::Storage(slot) => Ok(ProxyImplementation::Single(read_single_storage_implementation(&rpc, address, slot, block_number).await?)),
        ProxyDispatch::MultipleStorage(slots) => {
	    let addrs: Result<Vec<Address>, ProxyReadError> = join_all(slots.iter().map(|s| async { read_single_storage_implementation(&rpc, address, s, block_number).await })).await.into_iter().collect();
	    Ok(ProxyImplementation::Multiple(addrs?))
	},
        ProxyDispatch::Static(address) => Ok(ProxyImplementation::Single(address.clone())),
        ProxyDispatch::Facet_EIP_2535 => { Ok(read_facet_list_from_function(rpc, address, block_number).await?) },
        ProxyDispatch::FacetStorageSlot => Ok(read_diamond_implementation(&rpc, address, &DIAMOND_STANDARD_STORAGE_SLOT, block_number).await?),
        ProxyDispatch::External(_, _) => Err(ProxyReadError::ExternalProxy)
        // ProxyDispatch::External(address, dispatch) => Ok(get_proxy_implementation(rpc, address, dispatch).await?),
    }
}
