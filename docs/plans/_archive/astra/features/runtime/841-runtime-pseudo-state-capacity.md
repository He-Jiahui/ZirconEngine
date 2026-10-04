---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73/2026-09-19-runtime-pseudo-state-capacity.md
---

# Runtime841 · runtime-tree pseudo-state collector capacity

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime841 | Reserve the bounded runtime-tree pseudo-state collector capacity before alias collection | implemented_pending_validation | TDD source/model contract `4/4`; Runtime810+Runtime841 source batch `7/7`; the combined Runtime/Editor repair/capacity batch passes `30/30`, and the non-tooling performance-contract loader passes `2340/2340` across `640` modules; lower order/capacity regression and ignored `RUNTIME841_RUNTIME_PSEUDO_STATE_CAPACITY_BENCH_V1` marker are wired; exact Rustfmt and Python compilation pass. Managed Cargo/Windows Release, allocator, and Runtime73 style p50/p95/p99 evidence remain pending. |

## Implementation

`collect_runtime_pseudo_states` now derives a saturating upper bound from
authored attributes, enabled component flags, node flags, and the two-slot
resolved-painter alias allowance before collecting. Retained-state filtering,
alias order, sorting/deduplication, and empty-node semantics are unchanged.
The alias fanout values remain private named module constants because they are
helper-local control bounds, not a shared Runtime Interface contract.

For a dense 4,096-entry model the bounded collector removes `12→0` modeled
geometric growth events. This deterministic model is not a product latency or
allocator receipt.

## Source fingerprints

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/ui/v2/style/runtime_state.rs` | `4EC5919AEBB62FCBABE5168CFB47F94769D58D0214ECBC504F7E20C7D19D3712` |
| `zircon_runtime/src/ui/v2/style/runtime_state/capacity_tests.rs` | `2CF1504E6360BA6F14AB361D6F9FD41DEA0C97119338BC4F761E3BAE529AF9EB` |
| `tools/tests/test_runtime_runtime_state_capacity_performance_contract.py` | `D9FCBD5A9DAC4ED092C395A1711D63752E97166026B96DA25BA67C5375FD88D4` |

## Acceptance boundary

This record remains `implemented_pending_validation` until the owner-attributed
batched Windows Release lane executes the lower Rust regression and ignored
marker against current source and reports the Runtime73 style product
p50/p95/p99 gates. Tooling production is outside this slice.
