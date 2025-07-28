# IBC-Mini: Rust Light-Client + zk-Proof Bridge

A minimal demonstration of IBC light client verification with zero-knowledge proofs.

## Overview

This project demonstrates:
1. Tendermint header verification using a Rust light client
2. Zero-knowledge proof generation using Groth16 (arkworks)
3. On-chain proof verification via CosmWasm contract

## Project Structure

- `ibc-light-client/` - Core light client logic
- `zk-circuit/` - Zero-knowledge circuit implementation  
- `cli/` - Command-line interface for proof generation
- `cosmwasm-verifier/` - CosmWasm contract for proof verification

## Quick Start

```bash
# Build the project
cargo build --release

# Run demo
cargo run --bin ibc-mini demo

# Generate proof for specific height
cargo run --bin ibc-mini prove --height 1000
```

## Architecture

```
┌────────────┐      ┌─────────────┐      ┌───────────┐
│ Tendermint │ RPC  │ Rust Light  │ π    │ CosmWasm  │
│  Full Node │───► │  Client     │───► │ Verifier  │
└────────────┘      └─────────────┘      └───────────┘
```

**Note**: This is a portfolio demonstration project, not production-ready code.