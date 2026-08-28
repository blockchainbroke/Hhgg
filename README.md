# NearVerse

> **Sci-fi DeFi kingdom on NEAR Protocol** — Collect hero NFTs, stake veGOLD, and battle in async PvP arenas. Governed by a multi-role access system with no single-owner pattern.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![NEAR](https://img.shields.io/badge/NEAR-Protocol-000000?logo=near)](https://near.org)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org)
[![Contracts](https://img.shields.io/badge/Contracts-5-blueviolet)](#-architecture)
[![Status](https://img.shields.io/badge/Status-Pre--launch-blue)](#-roadmap)

**Working repo:** `blockchainbroke/Hhgg` · **Brand:** NearVerse · **Status:** Pre-launch

---

## 🌌 Vision

NearVerse is a sci-fi DeFi kingdom built on NEAR Protocol. Players recruit heroes from one of five classes, lock **veGOLD** to earn boost multipliers, and engage in **asynchronous PvP** with a commit-reveal scheme. All protocol governance flows through a multi-role access system — Admin, Council, and Player are distinct, with no 3-in-1 owner.

The protocol is designed around three hard principles:

1. **No single-owner risk** — roles are split, transfer is explicit
2. **No live cross-contract reads** — combat uses epoch-keyed snapshots
3. **No `unwrap()`** — every fallible path uses `require!()` or explicit `Result`

---

## 🏛 Architecture

The protocol is a Cargo workspace of **5 Rust smart contracts**, all targeting `wasm32-unknown-unknown`:

| Contract | Path | Purpose |
|---|---|---|
| `access` | `contracts/access/` | Role registry (Admin \| Council \| Player); pause/unpause; owner transfer |
| `hero` | `contracts/hero/` | NEP-171 / NEP-177 / NEP-178 hero NFT (5 classes) |
| `snapshot` | `contracts/snapshot/` | Epoch-keyed `HeroSnapshotView`; breaks cross-contract live reads |
| `veGOLD` | `contracts/veGOLD/` | Lock + boost math; NFT-gated; single fungible token |
| `pvp` | `contracts/pvp/` | Commit-reveal arena; dual stake; 24h timeout slash |

### Storage Namespacing

Every contract isolates state using a `BorshStorageKey` enum:

- `StorageKey::Roles` — Role registry
- `StorageKey::Heroes` — Hero NFT data
- `StorageKey::Snapshots` — Epoch-keyed snapshots
- `StorageKey::VeGOLD` — Lock and boost state
- `StorageKey::PvP` — Arena state

### Cross-Contract Communication

- All inter-contract calls go through **snapshot views** populated at epoch boundaries
- The `pvp` contract never calls `hero` or `veGOLD` during combat resolution
- Players commit a hash of their move + nonce; reveals are checked against the snapshot

---

## 🦸 Hero Classes

Each hero is a NEP-171 NFT with class-specific stat profiles:

| Class | Role | Profile |
|---|---|---|
| **Pilot** | Recon | High mobility, low defense |
| **Engineer** | Crafter | Balanced stats, crafting bonus |
| **Medic** | Support | Healing boost, support role |
| **Smuggler** | Rogue | Stealth bonus, trade discount |
| **Diplomat** | Governor | Council voting power, alliance bonuses |

Stat distribution is deterministically derived from a class seed and token id; minting is Admin-gated.

---

## 💰 Tokenomics — veGOLD

- **Single fungible token** with NFT-gated boost multipliers
- Lock duration determines boost (cliff-based, no rebasing)
- All arithmetic is explicit; no compounding
- Players lock veGOLD → receive a `Lock` record → boost is computed from `principal × duration × NFT_multiplier`
- Unlock is delayed until lock expiry; early-withdraw forfeits a configurable penalty

**Boost formula (simplified):**
```
boost_bps = base_bps + duration_bps * weeks_locked + nft_bonus_bps * hero_count
```

---

## ⚔ PvP — Async Combat

1. **Commit** — Player A and Player B both submit `sha256(move || salt || epoch)` plus a stake
2. **Reveal** — Within 24h, both reveal their `move` and `salt`; the contract verifies the hash
3. **Resolution** — Winner takes the combined stake minus protocol fee
4. **Timeout slash** — If either side fails to reveal within 24h, the other side claims the stake

There are no live cross-contract reads during combat. Heroes are read from the **most recent finalized snapshot** in `snapshot` contract.

---

## 🚀 Quickstart

### Prerequisites
- Rust 1.74+ with target `wasm32-unknown-unknown`
- `near-cli` for deployment
- Node 18+ (for off-chain tooling, optional)

### Build all contracts
```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

### Deploy
```bash
near deploy --accountId nearverse.testnet --wasmFile target/wasm32-unknown-unknown/release/access.wasm
near deploy --accountId hero.nearverse.testnet --wasmFile target/wasm32-unknown-unknown/release/hero.wasm
near deploy --accountId snapshot.nearverse.testnet --wasmFile target/wasm32-unknown-unknown/release/snapshot.wasm
near deploy --accountId vegold.nearverse.testnet --wasmFile target/wasm32-unknown-unknown/release/vegold.wasm
near deploy --accountId pvp.nearverse.testnet --wasmFile target/wasm32-unknown-unknown/release/pvp.wasm
```

### Initialize
```bash
near call nearverse.testnet new '{"admin": "you.testnet", "council": ["council.testnet"]}' --accountId you.testnet
```

See [`SPEC.md`](SPEC.md) for the full locked spec and [`RISK_REGISTER.md`](RISK_REGISTER.md) for design tradeoffs.

---

## 🛠 Development

### Layout
```
.
├── Cargo.toml              # workspace
├── contracts/
│   ├── access/             # role registry
│   ├── hero/               # NEP-171 NFT
│   ├── snapshot/           # epoch-keyed views
│   ├── veGOLD/             # lock + boost
│   └── pvp/                # commit-reveal arena
├── scripts/                # build & deploy helpers
├── .github/workflows/      # CI
├── docs/                   # deep-dives
├── LICENSE                 # MIT
├── README.md
├── RISK_REGISTER.md
└── SPEC.md
```

### Per-contract layout
Each contract is its own crate:
```
contracts/<name>/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs              # entrypoint
    ├── storage.rs          # StorageKey + state
    ├── errors.rs           # custom errors
    ├── role.rs             # (access only)
    ├── nft_core.rs         # (hero only) NEP-171
    ├── nft_metadata.rs     # (hero only) NEP-177
    ├── nft_approval.rs     # (hero only) NEP-178
    ├── hero.rs             # (hero only) class + stats
    ├── snapshot.rs         # (snapshot only)
    ├── lock.rs             # (veGOLD only)
    ├── boost.rs            # (veGOLD only)
    ├── arena.rs            # (pvp only)
    ├── commit.rs           # (pvp only) commit-reveal
    └── slash.rs            # (pvp only) timeout
```

### Test
```bash
cargo test --workspace
cargo test --workspace --target wasm32-unknown-unknown
```

### Lint
```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

---

## 🛡 Security Hard Guardrails

These are enforced in code, not just docs:

- ✅ **No `unwrap()`** in any state-modifying path (CI grep enforced)
- ✅ **No 3-in-1 owner** — Admin, Council, Player are distinct roles
- ✅ **Pausable** — only Admin can pause/unpause
- ✅ **Owner transfer** — restricted to Admin role, with 2-step transfer
- ✅ **Predecessor checks** — every mutator verifies `env::predecessor_account_id()`
- ✅ **`PanicOnDefault`** — prevents silent failures on deserialization
- ✅ **Storage isolation** — every contract uses its own `BorshStorageKey` namespace
- ✅ **Overflow checks** — `overflow-checks = true` in release profile

---

## 📚 Documentation

- [`SPEC.md`](SPEC.md) — Locked product spec
- [`RISK_REGISTER.md`](RISK_REGISTER.md) — Resolved disagreements and open risks
- [`docs/access.md`](docs/access.md) — Role registry deep-dive
- [`docs/hero.md`](docs/hero.md) — Hero NFT class system
- [`docs/pvp.md`](docs/pvp.md) — Commit-reveal arena mechanics
- [`docs/vegold.md`](docs/vegold.md) — Lock and boost math
- [`docs/snapshot.md`](docs/snapshot.md) — Epoch snapshotting
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — How to contribute

---

## 🗺 Roadmap

- [x] Batch 0 — Workspace scaffold, spec, all 5 contracts implemented
- [ ] Batch 1 — Hero NFT (NEP-171/177/178), snapshot contract
- [ ] Batch 2 — veGOLD lock/boost, PvP arena
- [ ] Batch 3 — Testnet deployment, audit prep
- [ ] Batch 4 — Mainnet launch

---

## 📄 License

MIT — see [`LICENSE`](LICENSE).
