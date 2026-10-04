---
title: Runtime74 dependency cascade target capacity
category: zircon_runtime
report_id: Runtime829-dependency-cascade-target-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime829 · dependency cascade target capacity

## Scope

`UiAssetDependencyIndex::cascade_invalidation_targets` publishes the reverse-dependency
closure in BFS order. The target vector previously started at zero capacity, so a high-fanout
first wave paid geometric growth even though the current `BTreeSet` length was known. Empty
lookups and self-cycles must remain allocation-free.

## Implementation

- Keep the borrowed `HashSet` visited index, `VecDeque` traversal, dependent order, and
  self-cycle exclusion unchanged.
- After the first genuinely admitted dependent, reserve the current reverse-edge fanout in
  `targets`; later branches append to the same output without an eager allocation on empty
  paths.
- Add a lower semantic/capacity regression and the ignored
  `RUNTIME829_DEPENDENCY_CASCADE_TARGET_CAPACITY_BENCH_V1` Release marker.

The reservation is a lower bound for the first published wave, not a new graph authority. It
does not alter the returned target set, BFS order, duplicate suppression, or query/report DTOs.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the old zero-capacity collector
and GREEN after the post-admission reservation was added (`4/4`). For a dense 4,096-target first
wave, the deterministic zero-capacity model changes `11` geometric growth events to `0`; an empty
or self-cycle traversal still has zero target capacity. These are allocation-shape counts only,
not allocator, CPU, RSS, or product percentile evidence.

## Local evidence

- `tools/tests/test_runtime_dependency_cascade_target_capacity_performance_contract.py`:
  `4/4`.
- Lower Rust module
  `zircon_runtime/src/ui/template/asset/dependency_index/cascade_target_capacity_tests.rs`
  covers fanout order/capacity, empty and self-cycle behavior, and the ignored Release marker.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and lower Rust files;
  Python compilation passes for the source contract.
- The current one-process non-tooling Runtime/Editor contract batch loads `627` modules and
  passes `2239/2239` tests in `5.505s` with zero failures, errors, or skips; the nine-slice
  focused loader passes `33/33` in `0.017s`.
- Managed Cargo/Windows Release compilation, lower Rust execution, allocator evidence, and
  dependency-cascade product p50/p95/p99 remain pending behind the external dirty-worktree
  admission gate. Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/dependency_index.rs` | `B1F21B6A20576EF62069C51C955018B8A41C2F950E7F31437B7BCF6B10B89F46` |
| `zircon_runtime/src/ui/template/asset/dependency_index/cascade_target_capacity_tests.rs` | `C4BBFA17F2AA1109014710E77244332B78DD9B6CA6B791BB9A884FD4B4622DB1` |
| `tools/tests/test_runtime_dependency_cascade_target_capacity_performance_contract.py` | `D12E38F64D30474195E1D1C4ACA8831B1DBEA43ACAF65E76EFA17F0948722EE4` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until the owner-
attributed Windows Release batch compiles the current Runtime/Editor tree, executes the lower
regression and ignored marker, and supplies allocator plus dependency-cascade product p50/p95/p99
evidence. Local source/model evidence does not claim final performance acceptance.
