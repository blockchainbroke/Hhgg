# NearVerse — Locked Spec

## Vision

NearVerse is a sci-fi DeFi kingdom on NEAR Protocol where players collect hero NFTs, stake veGOLD, and compete in async PvP arenas. The protocol is governed by a multi-role access control system (Admin | Council | Player) with no single 3-in-1 owner pattern.

## Tokenomics

- **veGOLD**: Single-token vault with NFT-gated boost multipliers
- Lock duration determines boost multiplier
- NFT holders receive additional yield multipliers
- No rebasing; all arithmetic is explicit and cliff-based

## Hero Classes

Five hero classes, each as a NEP-171 NFT:

1. **Pilot** — High mobility, low defense
2. **Engineer** — Balanced stats, crafting bonus
3. **Medic** — Healing boost, support role
4. **Smuggler** — Stealth bonus, trade discount
5. **Diplomat** — Council voting power, alliance bonuses

## PvP

- Async PvP with commit-reveal scheme
- Dual-stake mechanism
- 24-hour timeout slash for non-revealing players
- No live cross-contract reads during combat resolution

## Storage Namespacing

All storage keys are namespaced using `BorshStorageKey` enums:

- `StorageKey::Roles` — Role registry
- `StorageKey::Heroes` — Hero NFT data
- `StorageKey::Snapshots` — Epoch-keyed snapshots
- `StorageKey::VeGOLD` — Lock and boost state
- `StorageKey::PvP` — Arena state

## Security Hard Guardrails

- **No `unwrap()`**: All fallible operations use `require!()` or explicit `Result` handling
- **No 3-in-1 owner pattern**: Admin, Council, and Player roles are separate
- **Pausable**: Only Admin can pause/unpause the contract
- **Owner transfer**: Restricted to Admin role
- **Predecessor checks**: All state-modifying calls verify `env::predecessor_account_id()`
- **PanicOnDefault**: Prevents silent failures on deserialization
- **Storage isolation**: Each module uses its own `StorageKey` namespace