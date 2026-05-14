# rust-exec

A proof-of-concept to explore exchange connectivity with [Hyperliquid](hyperliquid.xyz) and trade workflow using the Rust language, [tokio](https://github.com/tokio-rs/tokio) and [hyperliquid_rust_sdk](https://github.com/hyperliquid-dex/hyperliquid-rust-sdk).

## Features

- Rust/Tokio async exchange connectivity
- Hyperliquid L2 market data ingestion
- exchange abstraction via trait
- shared state propagation using watch channels
- dry-run execution loop
- pre-trade risk check framework
- testnet integration tests
