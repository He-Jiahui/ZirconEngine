---
title: Runtime75 TreeView metadata collection capacity
category: zircon_runtime
report_id: Runtime842-tree-view-metadata-collection-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime842 · TreeView metadata collection capacity

## Scope

The default-interaction TreeView bridge reduces TOML metadata into ordered node IDs,
borrowed/owned option IDs, and disabled-option membership. Each recursive collector previously
started every direct array with zero-capacity output (and, where applicable, zero-capacity
deduplication storage), so a large authored list paid geometric growth before the first input
event reached the reducer.

## Implementation

- `collect_tree_node_ids` now reserves the direct TOML array bound for both its ordered output and
  borrowed deduplication set before recursion.
- `collect_borrowed_string_ids` and `collect_owned_string_ids` apply the same direct-array bound
  to their output and deduplication set.
- `collect_disabled_option_ids` reserves the direct array bound before inserting IDs.
- Nested traversal, table alias precedence, first-occurrence order, duplicate suppression, empty
  values, and the existing `range_selected_ids` exact range capacity remain unchanged. The
  reservation is a local lower bound; no second tree walk or new authority is introduced.

## TDD and regression contract

The Python source/model contract is
`tools/tests/test_runtime_tree_view_metadata_capacity_performance_contract.py`. TDD RED was
observed before the reservation lines and lower module existed; GREEN now passes `4/4`.
The lower Rust module
`zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_support/capacity_tests.rs`
checks nested ordering, duplicate behavior, borrowed/owned parity, disabled membership, and the
ignored managed marker
`RUNTIME842_TREE_METADATA_COLLECTION_CAPACITY_BENCH_V1`.

## Deterministic performance boundary

For a direct array of `4,096` metadata values, the allocation-shape model changes geometric
output growth from `11` events to `0`; the optimized collector reserves only the admitted direct
container bound. This is deterministic capacity evidence, not allocator, CPU/RSS, or product
p50/p95/p99 evidence.

## Local validation boundary

The focused source contract passes `4/4`; the combined adjacent Runtime/Editor source batch passes
`40/40` in `0.046s`; the latest batched non-tooling performance/pressure loader covers `647`
modules and passes `2366/2366` in `5.524s`, with zero load errors, failures, errors, or skips.
Scoped Rustfmt and Python compilation are clean. The lower Rust regression and Release benchmark
are wired but were not run through Cargo in this session. Managed Windows Release compilation,
allocator evidence, and TreeView product percentile gates remain pending because the shared external
`E:\Git\zr_vm` dirty-worktree admission gate is unresolved. Tooling production remains deferred
for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_support.rs` | `B7C95698AB252E61E13EE0D292228608371F0D6B91D0E3ADB9665C89A41EF928` |
| `zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_support/capacity_tests.rs` | `E927978F8D1B61A596BA7F7F6CA063CF70226493FD3ABDC69D8E099212334F0F` |
| `tools/tests/test_runtime_tree_view_metadata_capacity_performance_contract.py` | `ECBB04169513C55C99902EFEB63C90766B05FDD2913337BD1779019866C6CC5B` |

## Acceptance boundary

Keep this record at `implementation_complete` / `managed_validation_pending`.
The source/model and deterministic capacity receipts do not replace the
owner-attributed Windows Release compile, lower Rust regression, allocator
measurement, or TreeView product p50/p95/p99 evidence.
