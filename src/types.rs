use alloy_primitives::{U256, Address};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProxyType {
    NoProxy,

    // Some kind of proxy but didn't find which one
    Unknown,

    // Minimal statically defined ones
    EIP_1167,
    EIP_3448,
    EIP_7511,
    // Another type of static dispatch
    StaticAddress,

    // Storage slot
    EIP_897,
    EIP_1967,
    EIP_1967_CUSTOM,
    EIP_1967_ZOS,
    EIP_1967_BEACON,
    EIP_1822,

    // Diamond
    EIP_2535,
    DiamondOther,

    External
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum ProxyDispatch {
    Unknown,
    Storage(U256),
    MultipleStorage(Vec<U256>),
    Static(Address),
    Facet_EIP_2535,
    FacetStorageSlot,
    // Needs to be analysed
    External(Address, u32)
}
