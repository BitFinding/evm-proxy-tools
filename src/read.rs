use std::collections::HashMap;

use alloy::providers::Provider;
use alloy::rpc::types::BlockId;
use alloy::sol;
use alloy_primitives::{Address, B256, U256};
use futures::future::join_all;
use thiserror::Error;
use tracing::debug;

use crate::{Dispatch, consts::{DIAMOND_STANDARD_STORAGE_SLOT, ADDR_MASK_U256}, utils::as_u32_le};

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

impl ProxyImplementation {
    pub fn to_vec(&self) -> Vec<Address> {
        match self {
            ProxyImplementation::Single(addr) => vec![*addr],
            ProxyImplementation::Multiple(addrs) => addrs.to_owned(),
            ProxyImplementation::Facets(addrs) => addrs.keys().copied().collect(),
        }
    }
}

sol! {
    #[sol(rpc)]
    interface IDiamondLoupe {
        struct Facet { address facetAddress; bytes4[] functionSelectors; }
        function facets() external view returns (Facet[]);
    }
}

pub async fn read_single_storage_implementation<P>(
    provider: &P,
    address: &Address,
    storage: &U256,
    block_number: Option<u64>
) -> Result<Address, ProxyReadError>
where
    P: Provider
{
    let value = if let Some(block) = block_number {
        provider.get_storage_at(*address, *storage)
            .block_id(BlockId::number(block))
            .await
    } else {
        provider.get_storage_at(*address, *storage).await
    }.map_err(|e| ProxyReadError::RPCError(e.to_string()))?;

    debug!("stored value:: {:?}", value);
    
    if (value & *ADDR_MASK_U256) == value {
        Ok(Address::from_word(B256::from(value)))
    } else {
        Err(ProxyReadError::StorageNotAddress)
    }
}

pub async fn read_facet_list_from_function<P>(
    provider: P,
    address: &Address,
    block_number: Option<u64>
) -> Result<ProxyImplementation, ProxyReadError>
where
    P: Provider + Clone
{
    let contract = IDiamondLoupe::new(*address, provider);
    
    let call = contract.facets();
    let facets_result = if let Some(block) = block_number {
        call.block(BlockId::number(block)).call().await
    } else {
        call.call().await
    }.map_err(|e| ProxyReadError::RPCError(e.to_string()))?;
    
    let facets_hashmap: HashMap<Address, u32> = facets_result
        .iter()
        .flat_map(|facet| {
            facet.functionSelectors.iter().map(move |selector| {
                (facet.facetAddress, as_u32_le(&selector.0))
            })
        })
        .collect();
    
    Ok(ProxyImplementation::Facets(facets_hashmap))
}

pub async fn read_diamond_implementation<P>(
    _provider: &P,
    _address: &Address,
    _diamond_base: &U256,
    _block_number: Option<u64>
) -> Result<ProxyImplementation, ProxyReadError>
where
    P: Provider
{
    // TODO: implement properly
    Ok(ProxyImplementation::Multiple(Vec::new()))
    // Scan storage to find the first array (should have its size)
    // Go to the base of the array and get the structs
    // For each struct read the arrays of function signatures
}

pub async fn get_proxy_implementation<P>(
    provider: P,
    address: &Address,
    dispatch: &Dispatch,
    block_number: Option<u64>
) -> Result<ProxyImplementation, ProxyReadError>
where
    P: Provider + Clone + 'static
{
    match dispatch {
        Dispatch::Unknown => Err(ProxyReadError::UnknownProxy),
        Dispatch::Storage(slot) => {
            Ok(ProxyImplementation::Single(
                read_single_storage_implementation(&provider, address, slot, block_number).await?
            ))
        },
        Dispatch::MultipleStorage(slots) => {
            let futures = slots.iter().map(|s| {
                let provider = provider.clone();
                async move {
                    read_single_storage_implementation(&provider, address, s, block_number).await
                }
            });
            let addrs: Result<Vec<Address>, ProxyReadError> = join_all(futures).await.into_iter().collect();
            Ok(ProxyImplementation::Multiple(addrs?))
        },
        Dispatch::Static(static_address) => Ok(ProxyImplementation::Single(*static_address)),
        Dispatch::DiamondFacets => {
            Ok(read_facet_list_from_function(provider, address, block_number).await?)
        },
        Dispatch::DiamondStorage => {
            Ok(read_diamond_implementation(&provider, address, &DIAMOND_STANDARD_STORAGE_SLOT, block_number).await?)
        },
        Dispatch::External(_, _) => Err(ProxyReadError::ExternalProxy),
        Dispatch::Static6551 { implementation, .. } => {
            Ok(ProxyImplementation::Single(*implementation))
        },
        Dispatch::SelfAddressSlot => {
            let slot = U256::from_be_bytes(address.into_word().0);
            Ok(ProxyImplementation::Single(
                read_single_storage_implementation(&provider, address, &slot, block_number).await?
            ))
        },
    }
}
