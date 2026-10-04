---
title: Runtime11A AccessKit tree projection capacity
category: zircon_runtime
report_id: Runtime809-accesskit-tree-projection-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime809 · AccessKit tree projection capacity

## Scope

The AccessKit bridge projected each accessibility snapshot node and each child
list through iterator `collect` calls. The collectors had exact source bounds,
but started with an empty vector, so large accessibility snapshots could incur
geometric growth and copy work. The projection now reserves the snapshot node
bound (including the optional synthetic root) and each direct root/child bound,
then appends in the existing source order.

## Implementation

- Reserve `snapshot.nodes.len()` plus the optional synthetic-root slot with a
  saturating bound before node projection.
- Reserve `snapshot.roots.len()` and `source.children.len()` before direct
  child-ID projection.
- Preserve root selection, synthetic-root insertion order, child order, focus
  fallback, and all AccessKit node properties.
- Add a lower source regression and the ignored
  `RUNTIME809_ACCESSKIT_TREE_PROJECTION_CAPACITY_BENCH_V1` marker for the
  managed Release lane.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the three empty
collectors and GREEN after the bounded-buffer implementation. A deterministic
4,096-node model records twelve legacy geometric growth events versus zero
growth events after exact-bound reservation. This is allocation-shape evidence,
not allocator, CPU, RSS, screen-reader latency, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_runtime_accesskit_tree_projection_capacity_performance_contract.py`
  (`4/4`).
- Lower Rust source regression and ignored Release marker are wired in
  `accesskit/performance_tests.rs`.
- Exact-file Rustfmt and Python AST checks pass. The twelve-slice merged
  Runtime/Editor focused batch passes `48/48`; the strict non-tooling
  performance/pressure batch passes `2643/2643` across `682` files in
  `51.429s` with zero failures, errors, or skips. The pre-existing Runtime78
  AccessKit source contract was repaired to accept the bounded projection
  shape without weakening its focus-membership assertions. These are local
  source/model receipts; managed Cargo/Release and product percentile gates
  remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/accessibility/accesskit.rs` | `FD3194C2D730D8672F23BCC36BF8C7AD6D72E1BC3B65D541B618677F36185BFE` |
| `zircon_runtime/src/ui/accessibility/accesskit/performance_tests.rs` | `F22ACCC674947F3A1071C66ADA4F582E6AB02C3AE165CF2E825C7996B415A11C` |
| `tools/tests/test_runtime_accesskit_tree_projection_capacity_performance_contract.py` | `EE51DD1D7D7A2EE00B5B33506B7F684A5424552A31CEA20DF1419B6D5E072C80` |
| `tools/tests/test_runtime78_accesskit_focus_membership_performance_contract.py` | `9505CDF1C46C49B03EAD8293BEAF420BD4D4452823D3CCD8B1326BC2CD6BF29E` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Runtime UI tree, runs the lower regression and ignored
marker, and supplies AccessKit allocation and product p50/p95/p99 evidence.
Tooling production remains deferred for the later Rust migration.
