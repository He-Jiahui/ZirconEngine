---
title: Runtime200 Primary Pointer Source Counter
category: zircon_runtime
report_id: Runtime200-primary-pointer-source-counter-2026-09-14
date: 2026-09-14
related_to:
  - docs/plans/optimize/zircon_runtime/200/2026-09-13-active-pointer-index.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime200 Primary Pointer Source Counter

## Scope

`UiInputManager::active_pointer_is_primary` runs for every newly observed
pointer id. The active-pointer table had already acquired an exact pointer-id
index, but the new touch/pen admission path still scanned the ordered entry
vector to determine whether a primary pointer of the same source existed. This
slice keeps the ordered public entry view and removes only that residual scan.

## Implementation

- `UiActivePointerTable` now retains source-specific primary membership counters
  for Touch and Pen beside the pointer-id index.
- `upsert`, source/primary transitions, `remove`, and `clear` update the
  counters in the same mutation boundary as the ordered entries and index.
- `has_primary_for_source` gives the manager an expected-constant-time probe;
  Mouse and Unknown retain the existing unconditional-primary behavior.
- `active_pointer_is_primary` preserves the existing pointer-id hit behavior,
  touch-like source policy, and lack of multi-seat identity; only the fallback
  `entries().iter().any(...)` scan is removed.
- The lower Rust lifecycle regression compares each Touch/Pen counter result
  with the legacy ordered-entry scan across multiple-primary promotion,
  demotion, source transition, removal, and clear. The ignored Release marker
  `RUNTIME200_PRIMARY_POINTER_SOURCE_COUNTER_BENCH_V1` compares the old full
  vector scan (the primary entry is placed at the tail to prevent early
  short-circuiting) with the counter probe over 4,096 active pointers.

## Complexity and deterministic pressure model

For `P` active pointers, a new Touch/Pen pointer previously performed an
`O(P)` membership scan. The retained counter changes the probe to `O(1)` while
leaving insertion, ordered entry publication, and middle removal semantics
unchanged. A 100,000-pointer pressure model therefore records `100,000`
legacy membership probes versus `1` counter probe per admission decision. This
is structural work evidence, not a product CPU/RSS or input-to-present timing
claim.

## TDD and local evidence

- The source contract was intentionally RED before implementation: the table
  had no primary counters and the manager still used `entries().iter().any`.
- The source contract plus the adjacent active-pointer, pointer-state, and
  input-metadata contracts pass `15/15` in one invocation.
- The final all-current non-tooling Runtime/Editor discovery loaded `870`
  modules and passed `3562/3562` tests in `396.106s` in one process. This
  includes the new source contract and is local static/model evidence only.
- A post-regression-edit confirmation reran the same one-process scope at
  `870` modules and `3562/3562` tests with `FAILURES=0`, `ERRORS=0`, and
  `SKIPPED=0` in `517.932s`; the wall-clock variation is loader/system noise,
  not a product latency measurement.
- The benchmark-marker guard was then tightened to report explicit legacy vs
  counter membership steps; its focused source contract rerun passes `5/5`.
- Scoped Rustfmt passes for the pointer table, its lower test module, and the
  manager. No Cargo process was started; the lower Rust regression and Release
  benchmark remain for the existing managed batch.
- The comprehensive non-tooling Runtime/Editor batch remains the batched
  validation mechanism. Tooling production work is intentionally deferred.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs` | `29B309CE2808DCF61657BF34C0381E32B0DA27184133B7138964CB4AB6683CA3` |
| `zircon_runtime/src/ui/dispatch/input_manager/pointer_table/index_tests.rs` | `B439776E196F6B3E45F2055A9043B8BBEFB9DE8C6BB650AB3BFDC76444545182` |
| `zircon_runtime/src/ui/dispatch/input_manager/manager.rs` | `F9A51F5FAC3694DACE32E90FFA0491D5AD9DA4AC8531E24B5A5B87263D0CF654` |
| `tools/tests/test_runtime_ui_primary_pointer_source_counter_performance_contract.py` | `5C3081F781A0DB07AADBCF5026120042A49C1EF04507DB40CB27DD5DEEA865C1` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves compilation, counter parity under source
transitions, allocation behavior, and pointer-input p50/p95/p99. The existing
external `E:\\Git\\zr_vm` dirty-worktree admission blocker remains recorded in
the shared async log; no coordinator status query or retry is performed here.
