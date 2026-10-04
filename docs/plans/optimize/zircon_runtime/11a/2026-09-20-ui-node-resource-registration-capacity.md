---
title: Runtime843 UI node resource registration output capacity
category: zircon_runtime
report_id: Runtime843-ui-node-resource-registration-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime843 · UI node resource registration output capacity

## Scope

`UiAssetSurfaceIndex::record_tree_node_resources` scans retained node metadata and publishes
resource-free node IDs plus per-node resource URIs. The report vector previously grew from zero
for a large tree, and each metadata-bearing node also started its URI projection vector without a
known lower bound.

## Implementation

- Keep the allocation-free all-resource path by reserving
  `nodes_without_resources` lazily on the first empty projection, using the retained tree node
  count as its bounded upper bound.
- Construct `NodeResourceCollector` with the saturating sum of the three authored metadata-map
  lengths before traversing attributes, slot attributes, and style overrides.
- Preserve resource scheme filtering, fallback policy, URI first-seen order, per-node ownership,
  stale-edge removal, and the tolerant recursive path projection exactly as before.

## TDD and regression contract

The Python source/model contract is
`tools/tests/test_runtime_ui_node_resource_registration_capacity_performance_contract.py`.
TDD RED observed the missing lower file and reservation/constructor assertions; GREEN passes
`4/4`. The lower Rust module
`zircon_runtime/src/ui/template/asset/surface_index/capacity_tests.rs` checks the bounded URI
collector constructor and wires the ignored marker
`RUNTIME843_NODE_RESOURCE_REGISTRATION_CAPACITY_BENCH_V1`.

## Deterministic performance boundary

For a dense 4,096-node report in which every node has no resource URI, the output-capacity model
changes geometric `Vec` growth from `11` events to `0`. Metadata URI projection receives the
bounded top-level map-key lower bound without a second metadata walk. This is deterministic
allocation-shape evidence, not allocator, CPU/RSS, or product p50/p95/p99 evidence.

## Local validation boundary

The focused source/model contract passes `4/4`; the combined Runtime/Editor repair/capacity batch
passes `40/40` in `0.046s`; the batched non-tooling performance/pressure loader covers `647`
modules and passes `2366/2366` in `5.524s`, with zero load errors, failures, errors, or skips.
Exact-file Rustfmt and Python compilation are clean. The lower Rust regression and ignored Release
benchmark are wired but were not run through Cargo in this session. Managed Windows Release
compilation, allocator evidence, and UI asset registration product percentile gates remain pending
because the shared external `E:\Git\zr_vm` dirty-worktree admission gate is unresolved. Tooling
production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/surface_index/node_resource_registration.rs` | `363F6F5F00928FB9DAB8902DBE37B43317C4B115DA44C0917B6D5C113E50CF65` |
| `zircon_runtime/src/ui/template/asset/surface_index/capacity_tests.rs` | `DCCB9BFA9E9212330A2CBEFA0637110275737ED94063C3849CF3AFCAFA1CA88B` |
| `tools/tests/test_runtime_ui_node_resource_registration_capacity_performance_contract.py` | `684D4CA6F7933DB457D60E4AC20B360738E7DAAB272B37CCAFCE1E74BBF46D03` |

## Acceptance boundary

Keep this record at `implementation_complete` / `managed_validation_pending`.
The source/model and deterministic capacity receipts do not replace the
owner-attributed Windows Release compile, lower Rust regression, allocator
measurement, or UI asset-registration product p50/p95/p99 evidence.
