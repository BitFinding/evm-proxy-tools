# Open Source Readiness Plan for evm-proxy-tools

This document outlines the necessary work to prepare `evm-proxy-tools` for open source release, inspired by burntsushi's idiomatic Rust style (ripgrep, regex, walkdir).

---

## Executive Summary

The crate detects and reads EVM proxy contract implementations. Core functionality works (9 tests pass), but the API design, documentation, and code organization need significant improvements to meet open source quality standards.

**Current State:**
- Compiles with 1 dead code warning
- 8 clippy warnings (minor)
- No public documentation
- Mixed abstraction levels
- Inconsistent naming conventions

---

## Phase 1: API Consistency & Trait Design

### 1.1 Establish Core Traits (HIGH PRIORITY)

**Problem:** `ProxyDetector` trait exists but is underutilized. Detection and reading are separate concepts but not well abstracted.

**Solution:** Create a clean trait hierarchy inspired by burntsushi's approach:

```rust
/// A detector that can identify proxy patterns from bytecode.
pub trait Detector {
    /// The type of result this detector produces.
    type Match;
    
    /// Attempts to detect a proxy pattern in the given bytecode.
    /// Returns `None` if this detector doesn't match.
    fn detect(&self, bytecode: &[u8]) -> Option<Self::Match>;
}

/// A reader that can resolve proxy implementation addresses.
pub trait Reader {
    /// Read the implementation address(es) for a detected proxy.
    fn read<P: Provider>(
        &self,
        provider: &P,
        address: Address,
        block: Option<u64>,
    ) -> impl Future<Output = Result<Implementation, ReadError>>;
}
```

**Tasks:**
- [ ] Rename `ProxyDetector` trait to `Detector`
- [ ] Create `Reader` trait for implementation resolution
- [ ] Make `MinimalProxy` and `StorageSlotProxy` public with trait impls
- [ ] Add `DetectorChain` for composing multiple detectors

### 1.2 Unify Result Types

**Problem:** Functions return `Option<(ProxyType, ProxyDispatch)>` - tuple is not self-documenting.

**Solution:**
```rust
/// The result of proxy detection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detection {
    /// The type of proxy detected.
    pub proxy_type: ProxyType,
    /// How to dispatch to find the implementation.
    pub dispatch: Dispatch,
}
```

**Tasks:**
- [ ] Create `Detection` struct
- [ ] Rename `ProxyDispatch` to `Dispatch` (shorter, still clear)
- [ ] Update all detection functions to return `Option<Detection>`

### 1.3 Fix Naming Conventions

**Problem:** Several naming issues violate Rust conventions:
- `ProxyType::EIP_1167` uses underscores (should be `Eip1167`)
- `ProxyDispatch::Facet_EIP_2535` mixes conventions
- `FacetStorageSlot` is ambiguous
- `#[allow(non_camel_case_types)]` is a code smell

**Tasks:**
- [ ] Rename all `EIP_*` variants to `Eip*` (e.g., `Eip1167`, `Eip1967`)
- [ ] Rename `Facet_EIP_2535` to `DiamondFacets`
- [ ] Rename `FacetStorageSlot` to `DiamondStorage`
- [ ] Remove `#[allow(non_camel_case_types)]`
- [ ] Add `#[non_exhaustive]` to enums for future compatibility

---

## Phase 2: Error Handling (burntsushi style)

### 2.1 Unified Error Type

**Problem:** `ProxyReadError` exists but `ProxyDetectError` is unused. No unified error story.

**Solution:** Single, comprehensive error type:

```rust
/// Errors that can occur during proxy detection or reading.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    /// No proxy pattern detected.
    NotAProxy,
    /// RPC communication failed.
    Rpc(String),
    /// Storage value is not a valid address.
    InvalidStorageValue,
    /// Proxy delegates to external contract (cannot resolve directly).
    ExternalProxy { address: Address, selector: u32 },
    /// EVM execution failed during detection.
    Execution(String),
}

impl Error {
    /// Returns true if this error indicates no proxy was found.
    pub fn is_not_proxy(&self) -> bool { ... }
    
    /// If this is an external proxy error, returns the external address.
    pub fn external_address(&self) -> Option<Address> { ... }
}
```

**Tasks:**
- [ ] Create unified `Error` type with `ErrorKind` inner enum
- [ ] Add helper methods for common error queries
- [ ] Implement `std::error::Error` properly
- [ ] Remove unused `ProxyDetectError::Custom`
- [ ] Consider adding `#[non_exhaustive]` for future error variants

---

## Phase 3: Module Organization

### 3.1 Current Structure (Problematic)

```
src/
  lib.rs           # Minimal re-exports
  detect.rs        # Detection logic + EVM tracing (too much)
  read.rs          # Implementation reading
  types.rs         # ProxyType, ProxyDispatch
  proxy_inspector.rs  # REVM inspector + DB
  consts.rs        # Magic constants
  utils.rs         # Byte conversion utilities
```

### 3.2 Proposed Structure

```
src/
  lib.rs              # Public API, re-exports, top-level docs
  error.rs            # Unified error types
  types.rs            # Detection, ProxyType, Dispatch, Implementation
  
  detect/
    mod.rs            # Detector trait, DetectorChain, detect()
    minimal.rs        # EIP-1167, EIP-7511, EIP-3448 (static proxies)
    storage.rs        # EIP-897, EIP-1967, EIP-1822 (storage slot proxies)
    diamond.rs        # EIP-2535 diamond detection
    
  read/
    mod.rs            # Reader trait, read()
    storage.rs        # Read from storage slots
    diamond.rs        # Read diamond facets
    
  evm/
    mod.rs            # EVM execution helpers
    inspector.rs      # ProxyInspector
    db.rs             # ProxyDetectDB
    
  constants.rs        # Storage slots, byte patterns
  util.rs             # Internal utilities (not pub)
```

**Tasks:**
- [ ] Split `detect.rs` into `detect/` module with submodules
- [ ] Move `ProxyInspector` and `ProxyDetectDB` to `evm/` module
- [ ] Create `read/` module structure
- [ ] Keep internal utilities private (`pub(crate)`)

---

## Phase 4: Documentation (CRITICAL for Open Source)

### 4.1 Crate-Level Documentation

**Problem:** No crate documentation. README is minimal.

**Tasks:**
- [ ] Add comprehensive `//!` doc at top of `lib.rs`:
  - What the crate does
  - Quick start example
  - Supported proxy types with links to EIPs
  - Feature flags (if any)
  
- [ ] Expand README.md:
  - Badge section (crates.io, docs.rs, CI)
  - Installation instructions
  - Usage examples
  - Supported proxy standards table
  - Contributing guidelines link

### 4.2 Public API Documentation

**Problem:** Zero doc comments on public items.

**Tasks:** Add `///` docs to ALL public items:

- [ ] `ProxyType` - document each variant with EIP links
- [ ] `Dispatch` - explain each dispatch mechanism
- [ ] `Detection` - usage examples
- [ ] `Implementation` - explain Single vs Multiple vs Facets
- [ ] `get_proxy_type()` - main entry point, needs examples
- [ ] `get_proxy_implementation()` - async usage example
- [ ] All error types and variants

### 4.3 Internal Documentation

**Tasks:**
- [ ] Add comments explaining magic hex constants in `consts.rs`
- [ ] Document the EVM tracing strategy in `proxy_inspector.rs`
- [ ] Explain the "taint tracking" approach for storage slot detection
- [ ] Add `# Safety` sections if any unsafe code is added

---

## Phase 5: Code Quality Improvements

### 5.1 Fix Clippy Warnings

**Current warnings (8):**
1. `needless_borrow` in `extract_minimal_contract`
2. `manual_map` in detection chain (2 instances)
3. `identity_op` in utils (3 instances with `<< 0`)
4. `single_match` in inspector (match vs if)
5. `uninlined_format_args` in binaries (4 instances)

**Tasks:**
- [ ] Run `cargo clippy --fix` for auto-fixable issues
- [ ] Manually fix remaining warnings
- [ ] Add `#![warn(clippy::all, clippy::pedantic)]` to lib.rs
- [ ] Address or explicitly allow pedantic warnings

### 5.2 Remove Dead Code

- [ ] Remove unused `ProxyDetectError::Custom` variant
- [ ] Remove or implement commented `Tainter` structs in `proxy_inspector.rs`
- [ ] Clean up commented code throughout

### 5.3 Improve Type Safety

**Problem:** Using `u32` for function selectors is error-prone.

**Tasks:**
- [ ] Create `Selector` newtype: `pub struct Selector([u8; 4]);`
- [ ] Implement `From<[u8; 4]>`, `From<u32>`, `Display`, `Debug`
- [ ] Replace `u32` with `Selector` in `ProxyDispatch::External`
- [ ] Replace `u32` in `ProxyImplementation::Facets`

### 5.4 Simplify Utils

**Problem:** Three similar byte-to-u32 functions with manual bit shifting.

**Tasks:**
- [ ] Use `u32::from_be_bytes` / `u32::from_le_bytes`
- [ ] Reduce to single generic function or remove if standard lib suffices

---

## Phase 6: Missing Functionality

### 6.1 Complete Diamond Implementation

**Problem:** `read_diamond_implementation()` returns empty vec with TODO.

**Tasks:**
- [ ] Implement storage-based diamond facet reading
- [ ] Parse diamond storage layout (facet array structure)
- [ ] Add tests with real diamond contract bytecode

### 6.2 Add Builder Pattern for Detection

```rust
let detector = Detector::builder()
    .with_minimal_proxies(true)
    .with_storage_proxies(true)
    .with_diamonds(true)
    .build();

let result = detector.detect(&bytecode)?;
```

### 6.3 Add Sync API Option

**Problem:** `get_proxy_implementation` is async-only.

**Tasks:**
- [ ] Consider `blocking` feature flag for sync API
- [ ] Or document how to use with `tokio::runtime::Runtime::block_on`

### 6.4 Improve Test Coverage

**Current:** 9 tests covering happy paths.

**Tasks:**
- [ ] Add tests for error cases
- [ ] Add tests for edge cases (empty bytecode, malformed proxies)
- [ ] Add property-based tests for byte pattern matching
- [ ] Add integration tests with real RPC (behind feature flag)

---

## Phase 7: Project Infrastructure

### 7.1 Cargo.toml Improvements

**Tasks:**
- [ ] Fix `author` → `authors = ["snf"]`
- [ ] Add `description`, `license`, `repository`, `keywords`, `categories`
- [ ] Add `rust-version` MSRV
- [ ] Review and minimize dependencies
- [ ] Add feature flags for optional functionality

### 7.2 CI/CD Setup

**Tasks:**
- [ ] Add GitHub Actions workflow:
  - `cargo check`
  - `cargo test`
  - `cargo clippy -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo doc`
- [ ] Add Dependabot for dependency updates
- [ ] Add CHANGELOG.md
- [ ] Add CONTRIBUTING.md

### 7.3 Licensing

**Tasks:**
- [ ] Verify LICENSE file is complete
- [ ] Add SPDX headers to source files (optional)
- [ ] Add license badge to README

---

## Implementation Order

### Week 1: Foundation
1. Fix Cargo.toml metadata
2. Fix all clippy warnings
3. Remove dead code
4. Add crate-level documentation

### Week 2: Types & Traits
1. Create `Detection` struct
2. Rename enum variants (remove underscores)
3. Create unified `Error` type
4. Refactor `Detector` trait

### Week 3: Module Reorganization
1. Split into `detect/`, `read/`, `evm/` modules
2. Create `Reader` trait
3. Add public API documentation

### Week 4: Polish
1. Complete diamond implementation
2. Add `Selector` newtype
3. Expand test coverage
4. Set up CI/CD
5. Final README and CHANGELOG

---

## Success Criteria

- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes with >80% coverage
- [ ] `cargo doc` generates without warnings
- [ ] All public items have documentation
- [ ] README includes working examples
- [ ] CI pipeline is green
- [ ] Crate compiles on stable Rust (document MSRV)

---

## References

- [EIP-1167: Minimal Proxy Contract](https://eips.ethereum.org/EIPS/eip-1167)
- [EIP-1967: Proxy Storage Slots](https://eips.ethereum.org/EIPS/eip-1967)
- [EIP-2535: Diamond Standard](https://eips.ethereum.org/EIPS/eip-2535)
- [ripgrep](https://github.com/BurntSushi/ripgrep) - API design inspiration
- [walkdir](https://github.com/BurntSushi/walkdir) - Trait design inspiration
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
