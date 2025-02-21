# evm-proxy-tools

[![Crates.io](https://img.shields.io/crates/v/evm-proxy-tools.svg)](https://crates.io/crates/evm-proxy-tools)
[![Documentation](https://docs.rs/evm-proxy-tools/badge.svg)](https://docs.rs/evm-proxy-tools)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://github.com/bitfinding/evm-proxy-tools/actions/workflows/rust.yml/badge.svg)](https://github.com/bitfinding/evm-proxy-tools/actions/workflows/rust.yml)

A comprehensive Rust library for detecting and analyzing Ethereum proxy contracts. This tool helps developers and security researchers understand proxy patterns in smart contracts, with support for:

- [EIP-1167](https://eips.ethereum.org/EIPS/eip-1167) Minimal Proxy detection
- [EIP-1967](https://eips.ethereum.org/EIPS/eip-1967) Storage-based proxy detection  
- [EIP-2535](https://eips.ethereum.org/EIPS/eip-2535) Diamond proxy pattern analysis
- Custom proxy pattern detection

## Features

- Static analysis of contract bytecode
- Detection of common proxy patterns
- Implementation contract resolution
- Support for custom proxy patterns
- Async-first API design

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
evm-proxy-tools = "0.1.0"
```

## Quick Start

```rust
use evm_proxy_tools::{get_proxy_type, ProxyType};

async fn example() {
    let contract_code = vec![/* contract bytecode */];
    if let Some((proxy_type, dispatch)) = get_proxy_type(&contract_code) {
        println!("Detected proxy type: {:?}", proxy_type);
    }
}
```

## Building

For optimal performance, build with:

```bash
RUSTFLAGS='-C target-cpu=native' cargo build --profile maxperf --target x86_64-unknown-linux-gnu
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
