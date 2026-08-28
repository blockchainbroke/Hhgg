# Risk Register

## Resolved Disagreements

| Disagreement | Resolution | Batch |
|---|---|---|
| Admin-only vs multi-role init | Multi-role with role registry deferred to Batch 1; foundation stub only | 0 |
| veGOLD as NFT vs fungible | Fungible token with NFT-gated boost (simpler, gas-efficient) | 0 |
| Live cross-contract reads in PvP | Epoch-keyed snapshots; no live reads during combat | 0 |
| Owner role vs Admin role | Separated: Admin can pause/set roles; Owner is initial Admin but transferable | 0 |

## Open Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Role registry DOS via registration spam | Medium | High | Batch 1: require deposit or bonding for role registration |
| veGOLD boost manipulation via NFT rarity sniping | Low | Medium | Batch 2: cap boost multipliers, use time-weighted averages |
| PvP reveal griefing (reveal with worthless move) | Medium | Low | Slash applies regardless of reveal quality; bond forfeited |
| Storage iteration limits as contract scales | Low | High | Batch 1+2: use paginated iterators, avoid unbounded queries |
| Snapshot staleness vs fairness | Low | Medium | Snapshot epoch bound to block height; immutable after commit |
