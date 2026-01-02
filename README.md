# evm-proxy-tools

[![Crate](https://img.shields.io/crates/v/evm-proxy-tools.svg)](https://crates.io/crates/evm-proxy-tools)
[![Docs](https://docs.rs/evm-proxy-tools/badge.svg)](https://docs.rs/evm-proxy-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Detect and read EVM proxy contract implementations.

## Features

- Detect proxy patterns from EVM bytecode
- Resolve implementation addresses for upgradeable contracts
- Support for all major proxy standards

## Supported Proxy Types

| Standard | Description |
|----------|-------------|
| [EIP-1167](https://eips.ethereum.org/EIPS/eip-1167) | Minimal Proxy Contract (clone factory) |
| [EIP-1967](https://eips.ethereum.org/EIPS/eip-1967) | Standard Proxy Storage Slots |
| [EIP-1822](https://eips.ethereum.org/EIPS/eip-1822) | Universal Upgradeable Proxy (UUPS) |
| [EIP-2535](https://eips.ethereum.org/EIPS/eip-2535) | Diamond Standard (multi-facet) |
| [EIP-3448](https://eips.ethereum.org/EIPS/eip-3448) | MetaProxy Standard |
| [EIP-7511](https://eips.ethereum.org/EIPS/eip-7511) | Minimal Proxy with PUSH0 |
| EIP-897 | DelegateProxy interface |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
evm-proxy-tools = "0.1"
```

## Usage

### Detect Proxy Type

```rust
use evm_proxy_tools::{get_proxy_type, ProxyType, Dispatch};

let bytecode = hex::decode("363d3d373d3d3d363d73...").unwrap();
if let Some((proxy_type, dispatch)) = get_proxy_type(&bytecode) {
    println!("Proxy type: {:?}", proxy_type);
    println!("Dispatch: {:?}", dispatch);
}
```

### Read Implementation Address

```rust
use evm_proxy_tools::{get_proxy_implementation, Dispatch};
use alloy::providers::ProviderBuilder;

let provider = ProviderBuilder::new()
    .connect_http("https://eth.llamarpc.com".parse().unwrap());

let dispatch = Dispatch::Storage(slot);
let implementation = get_proxy_implementation(
    provider,
    &proxy_address,
    &dispatch,
    None, // latest block
).await?;
```

## CLI Tools

### proxy_tools

Analyze a proxy contract on-chain:

```bash
cargo run --bin proxy_tools -- 0x1234... -r https://eth.llamarpc.com
```

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Maximum performance build
RUSTFLAGS='-C target-cpu=native' cargo build --profile maxperf
```

## License

MIT
