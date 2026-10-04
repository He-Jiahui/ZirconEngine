---
title: Runtime11A reflection node-index hash lookup
category: zircon_runtime
report_id: Runtime11A-reflection-node-index-hash-lookup-2026-09-13
date: 2026-09-13
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A reflection node-index hash lookup

## Scope

`UiEventManager` resolves reflected nodes by `UiNodePath`; the index is a lookup-only
structure and has no production iteration consumer. Keeping it as an ordered `BTreeMap`
paid tree comparisons on every query even though the ordered `trees`/node traversal already
defines deterministic duplicate-path precedence.

## Implementation

- `node_index` now uses `HashMap<UiNodePath, (UiTreeId, UiNodeId)>`, while route and tree
  ownership maps remain ordered `BTreeMap`s.
- `rebuild_node_index` counts the incoming nodes once and reserves only the missing capacity
  after clearing the retained map. The existing ordered `trees` then node traversal is unchanged,
  so the last duplicate path continues to win exactly as before.
- Added lower regressions for retained capacity, path lookup, and duplicate-path precedence.
  No public API or reflection payload semantics changed.

## Validation

- TDD RED: the new Rust regression requires the `HashMap::capacity` lookup contract while the
  source still declared a `BTreeMap`.
- GREEN: the focused Runtime UI node-pool/reflection/layout contract batch passed `30/30`.
- The direct Runtime UI module batch (`91` modules) passed `466/466` tests in `15.490s`.
- Scoped Rustfmt (`--edition 2021 --check`) passed for the manager, reflection store, and
  optimization tests; scoped diff checks remain clean apart from normal line-ending notices.
- The broader batched Runtime/Editor static result collected immediately before this narrow
  slice remains the reference baseline at `548` modules / `2057/2057` tests in `15.174s`; this
  slice was not represented as a second broad rerun and no coordinator status was queried.

## Performance boundary

Expected lookup complexity changes from ordered `O(log N)` key search to expected `O(1)` hash
lookup. Capacity is retained across rebuilds to avoid warm reallocation. The managed Windows
Cargo/Release allocation and reflection-query latency p50/p95/p99 gates remain pending; this
record does not turn source-contract evidence into a product-performance acceptance.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs` | `BC047C7BCBBFADD1185276A1B93F3CCB7C86784D4BA144BE04CC0E4DB9B70B2E` |
| `zircon_runtime/src/ui/event_ui/manager/reflection_store.rs` | `1C666926BAE88B98E1BF6DBFA9D8433F45DEB268C78D204AB281CCDD27764646` |
| `zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs` | `4E2E6E53B8E32805E2B4AA2D6D47234156E1FB8CDFA1CE58A5CF8059D05FAA8F` |
| `tools/tests/test_runtime_ui_reflection_node_index_hash_lookup_performance_contract.py` | `032E1FB3290FC4DF18505E4B6AB1FA87E9E58521D451CF0A20A919B98268AAC5` |

## Acceptance

Implementation is complete and locally source-validated. The row stays
`managed_validation_pending` until an owner-attributed Windows Release batch proves compile,
behavior, allocation, and query-latency thresholds. Tooling production changes remain deferred.
