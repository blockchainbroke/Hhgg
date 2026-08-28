# NearVerse

> Sci-fi DeFi kingdom on NEAR Protocol — 5 hero classes, async PvP, NFT-gated veGOLD staking.

**Status:** Pre-launch · Working repo: Hhgg · Brand: NearVerse

## Stack
- NEAR Protocol (Rust smart contracts, NEP-171/177/178 NFTs)
- Async PvP (commit-reveal, dual-stake, 24h timeout slash)
- Single-token veGOLD with NFT-gated boost multipliers

## Architecture
- `contracts/access/` — Role enum (Admin | Council | Player), no 3-in-1 owner pattern
- `contracts/hero/` — NEP-171 hero NFT (5 classes: Pilot, Engineer, Medic, Smuggler, Diplomat)
- `contracts/snapshot/` — epoch-keyed HeroSnapshotView, no live cross-contract reads
- `contracts/veGOLD/` — lock + boost math, NFT-gated
- `contracts/pvp/` — commit-reveal arena

## Quickstart
```bash
cargo build --target wasm32-unknown-unknown --release
near deploy --accountId blockchainbloke.testnet --wasmFile target/wasm32-unknown-unknown/release/*.wasm
```

See SPEC.md for full locked spec.