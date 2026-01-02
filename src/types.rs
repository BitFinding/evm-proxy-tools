use alloy_primitives::{Address, U256};

/// Identifies the type of proxy pattern detected in a smart contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProxyType {
    /// Not a proxy contract.
    NoProxy,

    /// Proxy detected but specific type could not be determined.
    Unknown,

    /// EIP-1167: Minimal Proxy Contract (clone factory pattern).
    Eip1167,

    /// EIP-3448: MetaProxy Standard.
    Eip3448,

    /// EIP-7511: Minimal Proxy Contract with PUSH0.
    Eip7511,

    /// Static address embedded in bytecode (non-standard).
    StaticAddress,

    /// EIP-897: DelegateProxy interface.
    Eip897,

    /// EIP-1967: Standard Proxy Storage Slots.
    Eip1967,

    /// EIP-1967 with custom storage slot.
    Eip1967Custom,

    /// EIP-1967 ZeppelinOS variant.
    Eip1967Zos,

    /// EIP-1967 Beacon proxy variant.
    Eip1967Beacon,

    /// EIP-1822: Universal Upgradeable Proxy Standard (UUPS).
    Eip1822,

    /// EIP-2535: Diamond Standard (multi-facet proxy).
    Eip2535,

    /// Diamond-like proxy with non-standard implementation.
    DiamondOther,

    /// Proxy that delegates to an external contract for resolution.
    External,
}

/// Describes how to locate the implementation address for a proxy.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Dispatch {
    /// Implementation location is unknown.
    Unknown,

    /// Implementation address stored at a specific storage slot.
    Storage(U256),

    /// Multiple implementation addresses at different storage slots.
    MultipleStorage(Vec<U256>),

    /// Implementation address is statically embedded in bytecode.
    Static(Address),

    /// Diamond proxy using EIP-2535 facets() function.
    DiamondFacets,

    /// Diamond proxy using storage-based facet mapping.
    DiamondStorage,

    /// Implementation resolved via external contract call.
    External(Address, u32),
}

/// The result of detecting a proxy pattern in bytecode.
#[derive(Clone, Debug, PartialEq)]
pub struct Detection {
    /// The type of proxy detected.
    pub proxy_type: ProxyType,
    /// How to dispatch/resolve the implementation address.
    pub dispatch: Dispatch,
}

impl Detection {
    /// Creates a new Detection result.
    pub fn new(proxy_type: ProxyType, dispatch: Dispatch) -> Self {
        Self { proxy_type, dispatch }
    }
}

// Type alias for backward compatibility during migration
pub type ProxyDispatch = Dispatch;
