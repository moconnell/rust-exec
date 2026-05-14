# rust-exec

A proof-of-concept trade execution layer in Rust, exploring exchange connectivity and trade workflow with [Hyperliquid](hyperliquid.xyz) using [tokio](https://github.com/tokio-rs/tokio) and [hyperliquid_rust_sdk](https://github.com/hyperliquid-dex/hyperliquid-rust-sdk).

## Features

- Rust/Tokio async exchange connectivity
- Hyperliquid L2 market data ingestion
- exchange abstraction via trait
- shared state propagation using mpsc & watch channels
- dummy weights collection
- dry-run execution loop
- pre-trade risk check framework
- testnet integration tests
