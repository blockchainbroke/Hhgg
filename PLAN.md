# Implementation Plan — agentos/task-13c7c797

> **Goal (user):** "I don't see any of the contracts needed, please do this project and write a readme for the next AI agent."

## 1. Repository State

| Branch | State |
|---|---|
| `main` | **Empty** — only `# Hhgg` (6 B) README. No contracts. |
| `agentos/nebula-kingdoms-scaffold` | Locked spec (SPEC.md), risk register, root README, workspace `Cargo.toml`. **Source of truth for the spec.** |
| `agentos/task-d14f1ccb-...-access-contract` | **Already implements `contracts/access/`** with the canonical file layout (`lib.rs`, `role.rs`, `errors.rs`, `storage.rs`). **This is the convention all new contracts must mirror.** |
| `agentos/task-517820af` | A polished, user-facing NearVerse README (8.4 KB) — useful as a reference for tone and structure. |

The user is on `main` and sees no contracts. We must ship the **missing 4 of 5 contracts** plus a README for the next AI agent.

## 2. Decisions

1. **Base off `main`, not the scaffold branch.** The user is asking us to "do this project" from where they are. We carry the spec forward by recreating `SPEC.md`, `RISK_REGISTER.md`, root `README.md`, and root `Cargo.toml` in the working branch — they are small, low-risk files and the user should not need to merge an out-of-band branch to make sense of the repo.
2. **Reuse the `access` contract source verbatim from `agentos/task-d14f1ccb-...`.** It is the only existing reference implementation and establishes the project conventions. Copying it is faster and safer than re-deriving the patterns.
3. **Mirror the access contract's file layout exactly** for every new contract:
   - `src/lib.rs` — contract trait + impl
   - `src/errors.rs` — typed `*Error` enum
   - `src/storage.rs` — `StorageKey` enum (namespace only this contract's keys)
   - `src/<domain>.rs` — domain types (e.g. `hero.rs`, `pvp.rs`)
4. **No cross-contract live reads in `pvp`** — it reads `snapshot` only. SPEC §Security Hard Guardrail.
5. **All math uses `u128` with `checked_*` or `saturating_*`;** no `unwrap()`. Workspace lint `unsafe_code = "forbid"` is on; crate-level lints match.
6. **All contracts must `#[derive(PanicOnDefault)]`** (per SPEC).
7. **Workspace `Cargo.toml` lists all 5 members** with no commented-out entries; release profile is single-WASM per contract via `crate-type = ["cdylib"]`.

## 3. Tech Stack (locked)

- `near-sdk = "5.1.0"` (matches existing `access/Cargo.toml`)
- `edition = "2021"`, `resolver = "2"`
- WASM target: `wasm32-unknown-unknown`, release profile `opt-level = "z"`, `lto = true`, `panic = "abort"`, `overflow-checks = true`
- Borsh for serialization (via `near_sdk::borsh` re-export), JSON for view methods

## 4. File List & Change Descriptions

### 4.1 Root (workspace)

| File | Action | Notes |
|---|---|---|
| `Cargo.toml` | **Create** | Workspace with 5 members, lints block, release profile. |
| `SPEC.md` | **Create** | Copy from `agentos/nebula-kingdoms-scaffold/SPEC.md`. |
| `RISK_REGISTER.md` | **Create** | Copy from scaffold branch. |
| `LICENSE` | **Create** | MIT, short form. |
| `.gitignore` | **Create** | `/target`, `*.wasm`, `.DS_Store`, `.idea`, `.vscode`. |
| `README.md` | **Create** | Replaces stub. See §5. |

### 4.2 `contracts/access/` (already implemented elsewhere — port forward)

| File | Action | Notes |
|---|---|---|
| `contracts/access/Cargo.toml` | **Create** | Port from `agentos/task-d14f1ccb-...`. |
| `contracts/access/src/lib.rs` | **Create** | Port from same. |
| `contracts/access/src/role.rs` | **Create** | Port from same. |
| `contracts/access/src/errors.rs` | **Create** | Port from same. |
| `contracts/access/src/storage.rs` | **Create** | Port from same. |

### 4.3 `contracts/hero/` — NEP-171/177/178 hero NFT

| File | Action | Description |
|---|---|---|
| `contracts/hero/Cargo.toml` | Create | `near-sdk = 5.1.0`, `cdylib`. |
| `contracts/hero/src/lib.rs` | Create | `HeroContract` impl `NEP-171` (`nft_transfer`, `nft_transfer_call`, `nft_token`, `nft_supply_for_collection`, `nft_tokens`, `nft_metadata`), `NEP-177` (`nft_total_supply`, `nft_tokens_by_owner`, `nft_supply_for_owner`), `NEP-178` (`nft_approve`, `nft_revoke`, `nft_revoke_all`, `nft_is_approved`, `nft_approve_account_ids`). Minter is Admin-gated via local `minter` AccountId (see R-1). |
| `contracts/hero/src/hero.rs` | Create | `enum Class { Pilot, Engineer, Medic, Smuggler, Diplomat }`; `struct HeroStats { mobility, defense, support, stealth, diplomacy }`; `struct Hero { token_id, owner_id, class, stats, minted_at, generation }`. `fn derive_stats(class, generation) -> HeroStats` using a deterministic seeded table. |
| `contracts/hero/src/errors.rs` | Create | `HeroError` enum: `TokenNotFound`, `NotOwner`, `NotApproved`, `AlreadyExists`, `NotAuthorizedToMint`, `InvalidClass`, `InvalidStats`. |
| `contracts/hero/src/storage.rs` | Create | `enum StorageKey { Heroes, TokensByOwner, Approvals, Metadata }`. Distinct from `access::StorageKey::Roles`. |

### 4.4 `contracts/snapshot/` — epoch-keyed `HeroSnapshotView`

| File | Action | Description |
|---|---|---|
| `contracts/snapshot/Cargo.toml` | Create | `near-sdk = 5.1.0`, `cdylib`. |
| `contracts/snapshot/src/lib.rs` | Create | `SnapshotContract` with `commit_snapshot(epoch_id, hero_token_ids, caller)`, `get_snapshot(snapshot_id)`, `latest_snapshot()`. Snapshots are immutable once committed. |
| `contracts/snapshot/src/snapshot.rs` | Create | `struct HeroSnapshotView { snapshot_id, epoch_id, committed_at, heroes: Vec<HeroSummary> }`, `struct HeroSummary { token_id, class, owner_id, power: u32 }`. `power` is precomputed at commit time so `pvp` never reads hero stats live. |
| `contracts/snapshot/src/errors.rs` | Create | `SnapshotError`: `AlreadyCommitted`, `EmptyHeroSet`, `UnauthorizedCommitter`, `NotFound`. |
| `contracts/snapshot/src/storage.rs` | Create | `enum StorageKey { Snapshots, LatestPointer, EpochIndex }`. |

### 4.5 `contracts/veGOLD/` — fungible-token vault + boost math

| File | Action | Description |
|---|---|---|
| `contracts/veGOLD/Cargo.toml` | Create | `near-sdk = 5.1.0`, `cdylib`. |
| `contracts/veGOLD/src/lib.rs` | Create | `VeGoldContract` with `lock(account, amount, lock_until, hero_token_ids) -> LockId`, `unlock(lock_id)`, `boost_of(account) -> u16` (basis points, capped per Risk Register), `get_lock(lock_id)`. |
| `contracts/veGOLD/src/lock.rs` | Create | `struct Lock { owner, amount, lock_until, hero_token_ids, created_at }`. Pure math: `fn base_multiplier(duration_blocks) -> u16`, `fn nft_multiplier(hero_count) -> u16` (capped at 3.0x), `fn total_boost(lock, now) -> u16` saturating to `MAX_BOOST_BPS = 30_000`. |
| `contracts/veGOLD/src/errors.rs` | Create | `VeGoldError`: `InsufficientBalance`, `LockNotExpired`, `LockNotFound`, `InvalidDuration`, `Unauthorized`, `BoostOverflow`. |
| `contracts/veGOLD/src/storage.rs` | Create | `enum StorageKey { Locks, Balances, Boosts }`. |

### 4.6 `contracts/pvp/` — commit-reveal arena

| File | Action | Description |
|---|---|---|
| `contracts/pvp/Cargo.toml` | Create | `near-sdk = 5.1.0`, `cdylib`. |
| `contracts/pvp/src/lib.rs` | Create | `PvpContract` with `create_arena(opponent, snapshot_id, stake_amount, hero_token_ids)`, `commit(arena_id, hash)`, `reveal(arena_id, move, nonce)`, `slash_if_expired(arena_id)` (24h timeout, per SPEC). Reads only from `snapshot` (via cross-call returning `HeroSnapshotView`); never from `hero` live. |
| `contracts/pvp/src/arena.rs` | Create | `enum ArenaState { Committed, Revealed, Resolved, Slashed }`, `struct Arena { id, player_a, player_b, snapshot_id, stake, hash_a, hash_b, move_a, move_b, deadline_block, state, winner }`, `enum Move { Attack, Defend, Special, Flee }` with deterministic resolution seeded by both reveals. |
| `contracts/pvp/src/errors.rs` | Create | `PvpError`: `ArenaNotFound`, `HashMismatch`, `AlreadyRevealed`, `RevealDeadlinePassed`, `NotAParticipant`, `InvalidOpponent`, `UnauthorizedSlashed`. |
| `contracts/pvp/src/storage.rs` | Create | `enum StorageKey { Arenas, ArenaIndex }`. |

## 5. Root README — "for the next AI agent"

Replace the current 6-byte stub. Sections (in order):

1. **TL;DR** — what NearVerse is, 5 contracts on NEAR, where the spec lives.
2. **Repo layout** — tree of `contracts/*` with one-line purpose.
3. **Locked spec pointers** — links to `SPEC.md`, `RISK_REGISTER.md`; inline the 3 hard guardrails.
4. **Conventions every agent must follow** — no `unwrap()`, no 3-in-1 owner, `PanicOnDefault`, per-contract `StorageKey`, `crate-type = ["cdylib"]`, two-step ownership transfer, every state-changing call checks `env::predecessor_account_id()`.
5. **Build & test** — exact `cargo` commands (workspace + per-contract WASM build) and the `near deploy` one-liner.
6. **Contract dependency graph** — ASCII diagram: `access ← (hero, veGOLD, pvp)`, `snapshot ← pvp`. Note the **no live cross-contract reads in pvp** rule.
7. **Where to start a new task** — bullet list: read SPEC → read existing nearest contract in your domain → mirror its `errors.rs` / `storage.rs` layout → add your `StorageKey` variant without colliding → keep changes minimal.
8. **Open risks from `RISK_REGISTER.md`** — copy the table inline.
9. **Do-not list** — what an agent should **not** touch (CI, force-push, main without PR, deleting branches, modifying `access` without approval).
10. **Roadmap** — table of contracts × status (Implemented / Stub / Planned).

## 6. Risks & Dependencies

| ID | Risk | Mitigation |
|---|---|---|
| R-1 | Hero contract minter authorization: cross-call into `access` adds gas & reentrancy surface. | Use a local `minter: AccountId` set in `new()` and transferable by Admin; document that production should swap to `ext_access` cross-call. |
| R-2 | Snapshot staleness: pvp may commit to a snapshot that is later invalidated. | Snapshot is immutable; pvp stores the resolved `HeroSnapshotView` at arena creation, never re-reads. |
| R-3 | Boost math overflow in `total_boost`. | All multipliers in `u16` basis points; cap at `MAX_BOOST_BPS = 30_000`; use `saturating_mul`. |
| R-4 | PvP reveal griefing (worthless move). | Per `RISK_REGISTER.md` open risk: slash applies regardless of reveal quality; bond forfeited. Document; do not "fix" speculatively. |
| R-5 | Storage iteration limits in `hero` (`nft_tokens`). | Use `UnorderedMap` pagination; document `from_index`/`limit` args. |
| R-6 | Reentrancy via `nft_transfer_call` to pvp receiver. | Payouts occur in `nft_resolve_transfer` only; check state is still `Committed` before mutating. |
| R-7 | Re-implementing `access` from scratch risks divergence. | Port verbatim from the reference branch; the Coder phase must not "improve" it. |
| R-8 | Dependency: Coder must not relax workspace lints. | Lints live in root `Cargo.toml` only; per-crate `Cargo.toml` must not override. |
| R-9 | WASM size: 5 cdylibs at `opt-level = "z"` still risk exceeding NEAR contract size limits for complex contracts (esp. `pvp` with CryptoHash). | Keep logic minimal; move pure math to plain fns (inlinable). |
| R-10 | Backward compat: anyone depending on the current empty `main` will see a full workspace. | Acceptable — the user explicitly asked for the project to be done. Document in PR body. |

## 7. Ordered Execution Steps (for the Coder)

1. Create root `Cargo.toml`, `SPEC.md`, `RISK_REGISTER.md`, `LICENSE`, `.gitignore` from the scaffold branch.
2. Port `contracts/access/` verbatim (4 source files + `Cargo.toml`).
3. Implement `contracts/hero/` (5 files: `Cargo.toml` + 4 src).
4. Implement `contracts/snapshot/` (5 files).
5. Implement `contracts/veGOLD/` (5 files).
6. Implement `contracts/pvp/` (5 files).
7. Uncomment all members in root `Cargo.toml`.
8. Replace root `README.md` per §5.
9. Per-contract compile-only sanity: each `Cargo.toml` should be `cargo check`-able in isolation.

## 8. Out of Scope (do NOT do)

- Writing unit tests (Reviewer will add or request them).
- Wiring deployment scripts or CI.
- Changing the `access` contract.
- Adding a 6th contract.
- Token standards beyond NEP-141/171/177/178.
- Modifying `SPEC.md` semantics (it's a locked spec).

## 9. Acceptance Criteria

- All 5 contracts present in `contracts/` and listed in root `Cargo.toml` `[workspace] members`.
- Each contract has the canonical 4-file layout (`lib.rs`, domain file, `errors.rs`, `storage.rs`) plus its `Cargo.toml`.
- `cargo check -p hero -p snapshot -p veGOLD -p pvp -p access` would succeed.
- Root `README.md` follows §5 and answers the next agent's questions in order.
- `SPEC.md`, `RISK_REGISTER.md`, `LICENSE`, `.gitignore` present.
- Workspace lints `unsafe_code = "forbid"` retained.
- No `unwrap()` in any new `src/*.rs` (Coder self-check before commit).
- Commit messages in conventional-commits form; branch is `agentos/task-13c7c797`; one PR opened to `main`.
