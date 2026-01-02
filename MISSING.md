# Missing Proxy Patterns

This document catalogs proxy patterns NOT currently supported by `evm-proxy-tools`, their importance, usage across EVM networks, and implementation plans.

## Executive Summary

The library currently supports 7 core standards (EIP-1167, EIP-1967, EIP-1822, EIP-2535, EIP-3448, EIP-7511, EIP-897). This document identifies **15+ additional patterns** found in production, ranging from EIP standards to protocol-specific implementations.

**Priority Tiers:**
- **P0 (Critical)**: Widely deployed, billions in TVL - Safe Proxy, ERC-6551, Compound Unitroller
- **P1 (High)**: Significant usage, growing adoption - CWIA variants, 0age proxy, Vyper Beta
- **P2 (Medium)**: Niche but standardized - ERC-1538, ERC-3561, Sequence Wallet
- **P3 (Low)**: Emerging/Draft standards - ERC-7760, ERC-7702, ERC-7546

---

## P0: Critical Priority

### 1. Safe (Gnosis Safe) Proxy

**Status:** Production (billions in TVL)

**Where Used:**
- Safe multisig wallets (9.7M+ deployments, $100B+ in assets)
- Ethereum, Polygon, Arbitrum, Optimism, BSC, and all major EVM chains
- Most DAO treasuries and protocol admin contracts

**How It Works:**
The Safe proxy uses a simple storage-based delegation pattern. The implementation address (called `singleton` or `masterCopy`) is stored at **slot 0**. The fallback function intercepts calls to `masterCopy()` (selector `0xa619486e`) to return the implementation directly without delegatecall.

```solidity
// Storage layout
address internal singleton;  // slot 0

// Fallback intercepts masterCopy() selector
fallback() external payable {
    if (msg.sig == 0xa619486e) {
        // Return singleton directly
        assembly { mstore(0, sload(0)) return(0, 0x20) }
    }
    // Otherwise delegatecall to singleton
}
```

**Detection:**
- Bytecode contains selector `a619486e` (masterCopy)
- OR: Storage slot 0 contains a valid contract address
- Bytecode pattern: Look for the specific Safe assembly fallback

**Implementation Read:**
```rust
// Method 1: Call masterCopy()
let impl = provider.call(proxy, "masterCopy()").await?;

// Method 2: Read storage slot 0
let impl = provider.get_storage_at(proxy, U256::ZERO).await?;
```

**Why Important:**
- Single most widely used smart contract wallet
- Critical infrastructure for DAOs, protocols, and institutions
- Many tools already special-case Safe detection

**Implementation Plan:**
1. Add `ProxyType::GnosisSafe` variant
2. Detect via bytecode pattern (selector `a619486e`) OR storage slot 0 heuristic
3. Read implementation from slot 0
4. Add support for both v1.3.0 and v1.4.x patterns

---

### 2. ERC-6551: Token Bound Accounts (TBA)

**Status:** Final (ERC), rapidly growing adoption

**Where Used:**
- NFT ecosystems (NFTs that own assets)
- Gaming (character inventories)
- Ethereum, Polygon, Base, Optimism
- 5,700+ deployments on mainnet (as of 2023)

**How It Works:**
ERC-6551 creates smart contract accounts owned by NFTs. Each account is an EIP-1167-style minimal proxy with additional immutable data appended:

```
[EIP-1167 proxy bytecode][salt][chainId][tokenContract][tokenId]
```

The registry deploys accounts deterministically based on the NFT's identity.

**Detection:**
- Bytecode starts with EIP-1167 pattern
- Bytecode length is 173 bytes (45 + 128 bytes of packed data)
- Data section contains: `uint256 salt, uint256 chainId, address tokenContract, uint256 tokenId`

```rust
// Bytecode structure
const EIP_6551_SIZE: usize = 173;
const EIP_1167_SIZE: usize = 45;

fn is_eip_6551(code: &[u8]) -> bool {
    code.len() == EIP_6551_SIZE && is_eip_1167(&code[..EIP_1167_SIZE])
}
```

**Implementation Read:**
```rust
// Implementation is in the EIP-1167 portion
let impl_addr = extract_eip_1167_address(&code[..45]);

// Token info is in the data section
let (salt, chain_id, token_contract, token_id) = decode_6551_data(&code[45..]);
```

**Why Important:**
- Enables NFTs to own assets and interact with DeFi
- Growing ecosystem (games, social, identity)
- Already partially detected as EIP-1167 but metadata is lost

**Implementation Plan:**
1. Add `ProxyType::Eip6551` variant
2. Detect via bytecode length (173 bytes) + EIP-1167 prefix
3. Add `Dispatch::Static6551 { address, token_contract, token_id, chain_id }` variant
4. Parse immutable data section

---

### 3. Compound Unitroller Pattern

**Status:** Production (billions in TVL)

**Where Used:**
- Compound V2 and all forks (Venus, Cream, Benqi, etc.)
- Most lending protocol comptrollers
- Ethereum, BSC, Avalanche, Polygon, Fantom

**How It Works:**
The Unitroller is a transparent proxy with a simple storage layout. The implementation is stored in a named variable (`comptrollerImplementation`) rather than an EIP-1967 slot.

```solidity
contract UnitrollerAdminStorage {
    address public admin;                        // slot 0
    address public pendingAdmin;                 // slot 1
    address public comptrollerImplementation;   // slot 2
    address public pendingComptrollerImplementation; // slot 3
}
```

**Detection:**
- Contract has function selector for `comptrollerImplementation()` (`0xbb82aa5e`)
- OR: Storage slot 2 contains implementation address
- Often has `_setPendingImplementation` function

**Implementation Read:**
```rust
// Method 1: Call the getter
let impl = provider.call(proxy, "comptrollerImplementation()").await?;

// Method 2: Read storage slot 2
let impl = provider.get_storage_at(proxy, U256::from(2)).await?;
```

**Why Important:**
- Foundation of DeFi lending (Compound, Aave v2 uses similar)
- Billions in TVL across forks
- Pattern copied by many protocols

**Implementation Plan:**
1. Add `ProxyType::CompoundUnitroller` variant
2. Detect via bytecode function selector scan OR EVM trace
3. Add `Dispatch::Storage` with slot 2
4. Consider generic "named storage" detection

---

## P1: High Priority

### 4. Clones With Immutable Args (CWIA)

**Status:** Production (multiple variants)

**Where Used:**
- Sudoswap (AMM pools)
- 0xSplits (payment splitting)
- Ajna (lending)
- Astaria (lending)
- Buttonwood (bonds)

**How It Works:**
CWIA proxies append immutable configuration data to the bytecode. Before delegating, the proxy reads this data and appends it to calldata. The logic contract reads it via `_getArgXxx()` helpers.

```
[proxy bytecode][immutable args][2-byte length]
```

**Variants Detected in Production:**

| Variant | Instances | Bytecode Signature |
|---------|-----------|-------------------|
| ClonesWithImmutableArgs | 264 | `3d3d3d3d363d3d3761...5af43d3d93803e6057fd5bf3` |
| ClonesWithCallData | 336 | `363d3d3761...5af43d82803e903d91603657fd5bf3` |
| Sudoswap CWIA | 318 | `3d3d3d3d363d3d37605160353639...` |
| Solady CWIA | 78 | `36602c57343d527f9e4ac34f21c619...` |
| 0xSplits CWIA | 3 | `36602f57343d527f9e4ac34f21c619...` |

**Detection:**
Each variant has a distinct bytecode pattern. The implementation address is embedded, and immutable args follow.

**Implementation Read:**
```rust
// Extract implementation from bytecode
let impl = extract_cwia_implementation(code, variant);

// Extract immutable args (optional, for advanced use)
let args = extract_cwia_args(code, variant);
```

**Why Important:**
- Gas-efficient for factory patterns
- Used by major DeFi protocols
- Multiple incompatible variants in production

**Implementation Plan:**
1. Add `ProxyType::ClonesWithImmutableArgs` with variant enum
2. Add patterns for all 5+ variants
3. Extract implementation address from each pattern
4. Optionally expose immutable args length

---

### 5. 0age More-Minimal Proxy

**Status:** Production (9,928 instances)

**Where Used:**
- Various factory contracts
- Gas-optimized deployments

**How It Works:**
A 44-byte variant of EIP-1167 that saves 1 byte and 4 gas per call by reordering instructions.

**Bytecode Pattern:**
```
3d3d3d3d363d3d37363d73<ADDR>5af43d3d93803e602a57fd5bf3
```

**Detection:**
```rust
const ZERO_AGE_FIRST: &[u8] = &hex!("3d3d3d3d363d3d37363d73");
const ZERO_AGE_SECOND: &[u8] = &hex!("5af43d3d93803e602a57fd5bf3");
```

**Implementation Plan:**
1. Add `ProxyType::ZeroAgeMinimal` variant
2. Add bytecode pattern detection
3. Extract static address from bytecode

---

### 6. Vyper Beta Proxy

**Status:** Legacy but still in use (4,270 instances)

**Where Used:**
- Uniswap V1 (all exchange contracts)
- Early Curve pools
- Legacy Vyper deployments

**How It Works:**
Pre-EIP-1167 proxy from early Vyper. Notable quirk: always returns 4096 bytes regardless of actual response size.

**Bytecode Pattern:**
```
366000600037611000600036600073<ADDR>5af41558576110006000f3
```

**Detection:**
The `611000` (PUSH2 0x1000) is distinctive - it's the 4096-byte return buffer.

**Implementation Plan:**
1. Add `ProxyType::VyperBeta` variant (already partially supported as noted in detect.rs)
2. Ensure proper bytecode pattern matching
3. Document the 4096-byte return quirk

---

### 7. Solady PUSH0 Proxy

**Status:** Production (growing post-Shanghai)

**Where Used:**
- New deployments on post-Shanghai networks
- Gas-optimized minimal proxies
- Ethereum mainnet, L2s

**How It Works:**
Uses the `PUSH0` opcode (introduced in Shanghai) to push zero more efficiently. 45 bytes but 8 gas cheaper per call than EIP-1167.

**Bytecode Pattern:**
```
5f5f365f5f37365f73<ADDR>5af43d5f5f3e6029573d5ffd5b3d5ff3
```

**Detection:**
Starts with `5f5f` (PUSH0 PUSH0).

**Implementation Plan:**
1. Already have `ProxyType::Eip7511` - verify this covers Solady variant
2. Add any missing PUSH0-based patterns
3. Note: Only works on Shanghai+ networks

---

## P2: Medium Priority

### 8. ERC-1538: Transparent Contract Standard

**Status:** Withdrawn (superseded by EIP-2535)

**Where Used:**
- Legacy multi-facet contracts
- Some older upgradeable systems

**How It Works:**
Predecessor to Diamond. Uses a manager contract to route function calls to different logic contracts based on selector.

**Detection:**
- Has `updateContract(address,string,string)` function
- Has `delegateAddress(bytes4)` to query routing

**Implementation Read:**
```rust
// For each function selector, query the delegate
let impl = provider.call(proxy, "delegateAddress(bytes4)", selector).await?;
```

**Implementation Plan:**
1. Add `ProxyType::Eip1538` variant
2. Detect via function selector presence
3. Add `Dispatch::FunctionRouter` that requires per-selector queries

---

### 9. ERC-3561: Trust Minimized Upgradeability

**Status:** Stagnant

**Where Used:**
- Security-focused upgradeable contracts
- Timelock-protected upgrades

**How It Works:**
Adds a mandatory time delay before upgrades take effect. Uses a "next implementation" slot.

**Storage Slots:**
```
next_implementation: keccak256("eip3561.proxy.next.implementation") - 1
upgrade_time: keccak256("eip3561.proxy.upgrade.time") - 1
```

**Implementation Plan:**
1. Add `ProxyType::Eip3561` variant
2. Check for next_implementation slot
3. Return both current (EIP-1967) and pending implementation

---

### 10. Sequence Wallet Proxy

**Status:** Production (1,888 instances)

**Where Used:**
- Sequence smart wallet
- Modular wallet infrastructure

**How It Works:**
Uses the proxy's own address as a storage key for the implementation. Most expensive proxy pattern due to storage read.

**Detection:**
```
363d3d373d3d3d363d30545af43d82803e903d91601857fd5bf3
```

The `30` (ADDRESS) `54` (SLOAD) sequence is distinctive - it loads from `storage[address(this)]`.

**Implementation Read:**
```rust
// Implementation stored at slot = address(proxy)
let slot = U256::from_be_bytes(proxy.into());
let impl = provider.get_storage_at(proxy, slot).await?;
```

**Implementation Plan:**
1. Add `ProxyType::SequenceWallet` variant
2. Detect via bytecode pattern
3. Add `Dispatch::SelfAddressSlot` variant

---

## P3: Lower Priority / Emerging

### 11. ERC-7702: EOA Code Delegation

**Status:** Draft (Pectra upgrade target)

**Where Used:**
- Future: EOA smart account upgrades
- Account abstraction integration

**How It Works:**
Allows EOAs to temporarily set their code to delegate to a contract. The EOA signs an authorization that sets `code = 0xef0100 || address`.

**Detection:**
- Code starts with `0xef0100` followed by 20-byte address
- Only valid after Pectra upgrade

**Implementation Plan:**
1. Add `ProxyType::Eip7702` variant (post-Pectra)
2. Detect magic prefix `0xef0100`
3. Extract delegated address

---

### 12. ERC-7760: Minimal Upgradeable Proxies

**Status:** Draft

**Where Used:**
- Proposed standard for upgradeable clones
- Not yet widely deployed

**How It Works:**
Combines minimal proxy with upgradeability and immutable args support. Exposes `implementation()` on-chain.

**Implementation Plan:**
1. Monitor adoption
2. Add support when finalized

---

### 13. ERC-7546: Upgradeable Clone

**Status:** Draft

**Where Used:**
- Hybrid clone/diamond pattern
- Experimental deployments

**How It Works:**
Combines EIP-1967 upgradeability with horizontal extensibility (facets).

**Implementation Plan:**
1. Monitor adoption
2. Implement when patterns stabilize

---

### 14. Beacon Proxy Variations

**Status:** Production (via EIP-1967)

**Where Used:**
- Yearn V3 Vaults
- NFT collections
- Any "upgrade all clones at once" pattern

**How It Works:**
Proxy stores beacon address (not implementation). Beacon provides implementation to all attached proxies.

**Current Support:**
Already supported via `ProxyType::Eip1967Beacon` and slot `0xa3f0ad74...`

**Gap:**
Need to ensure two-step resolution works:
1. Read beacon address from proxy
2. Call `implementation()` on beacon

---

### 15. 0xSplits Clones

**Status:** Production (2,890 instances)

**Where Used:**
- 0xSplits payment splitting
- Revenue share contracts

**How It Works:**
Custom minimal proxy with embedded split configuration.

**Bytecode Pattern:**
```
36603057343d52307f830d2d700a97af574b186c80d40429385d24241565b08a7c559ba283a964d9b160203da23d3df35b3d3d3d3d363d3d37363d73<ADDR>5af43d3d93803e605b57fd5bf3
```

**Implementation Plan:**
1. Add `ProxyType::ZeroXSplitsClones` variant
2. Add bytecode pattern
3. Extract implementation address

---

## Implementation Roadmap

### Phase 1: Critical (Week 1-2)
- [ ] Safe Proxy detection and read
- [ ] ERC-6551 Token Bound Accounts
- [ ] Compound Unitroller pattern

### Phase 2: High Priority (Week 3-4)
- [ ] CWIA variants (all 5+)
- [ ] 0age More-Minimal Proxy
- [ ] Vyper Beta verification
- [ ] Solady PUSH0 variants

### Phase 3: Medium Priority (Week 5-6)
- [ ] Sequence Wallet
- [ ] 0xSplits Clones
- [ ] ERC-1538 (if demand exists)
- [ ] ERC-3561 (if demand exists)

### Phase 4: Emerging (Ongoing)
- [ ] ERC-7702 (post-Pectra)
- [ ] ERC-7760 (when finalized)
- [ ] ERC-7546 (when finalized)

---

## Detection Strategy Summary

| Pattern | Detection Method | Implementation Location |
|---------|-----------------|------------------------|
| Safe Proxy | Selector `0xa619486e` in bytecode | Storage slot 0 |
| ERC-6551 | EIP-1167 + 173 byte length | Bytecode offset 10-30 |
| Compound | Selector `0xbb82aa5e` | Storage slot 2 |
| CWIA | Variant-specific bytecode | Bytecode (varies) |
| 0age | `3d3d3d3d363d3d37363d73` prefix | Bytecode offset 10-30 |
| Vyper Beta | `611000` in bytecode | Bytecode offset 23-43 |
| Solady PUSH0 | `5f5f365f5f37` prefix | Bytecode offset 9-29 |
| Sequence | `30545af4` in bytecode | Storage[address(this)] |
| 0xSplits | Long prefix with `307f830d2d70` | Bytecode (late offset) |

---

## References

- [Banteg's Minimal Proxy Compendium](https://banteg.xyz/posts/minimal-proxies/) - Comprehensive proxy archaeology
- [OpenZeppelin Proxy Docs](https://docs.openzeppelin.com/contracts/5.x/api/proxy)
- [EIP-1167](https://eips.ethereum.org/EIPS/eip-1167) - Minimal Proxy
- [EIP-1967](https://eips.ethereum.org/EIPS/eip-1967) - Proxy Storage Slots
- [EIP-2535](https://eips.ethereum.org/EIPS/eip-2535) - Diamond Standard
- [ERC-6551](https://eips.ethereum.org/EIPS/eip-6551) - Token Bound Accounts
- [Safe Contracts](https://github.com/safe-global/safe-smart-account)
- [Compound Protocol](https://github.com/compound-finance/compound-protocol)
- [Solady LibClone](https://github.com/Vectorized/solady/blob/main/src/utils/LibClone.sol)
- [Clones With Immutable Args](https://github.com/wighawag/clones-with-immutable-args)
