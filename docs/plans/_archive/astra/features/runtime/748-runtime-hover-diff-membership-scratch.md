---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-14-hover-diff-membership-scratch.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
tests:
  - tools/tests/test_runtime_pointer_hover_membership_scratch_performance_contract.py
  - zircon_runtime/src/ui/surface/surface/event_routing.rs (hot_path_tests)
---

# Runtime200 hover-diff membership scratch reuse

The large-path pointer hover diff now reuses a surface-owned membership table.
The route keeps the existing equal-path and bounded-linear fast paths, clears
and grows only the retained table when necessary, then drops an over-sized
table at the configured safety ceiling. Entered/left ordering and pointer
capture/focus behavior are unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime200 / pointer hover | Replace the per-event large-path membership `HashSet` with non-serialized surface scratch and retain warm capacity. | New source contract `4/4`; combined hover-diff/pressure/source invocation `13/13`; post-change Runtime/Editor performance-plus-pressure loader covers `557` modules and passes `2072/2072` in `13.644s`; a subsequent all-current Runtime/Editor contract invocation covers `869` modules and passes `3557/3557` in `337.155s`; Wiki validation reports `272/272` pages with zero errors; Rust large-path capacity regression and ignored `RUNTIME200_HOVER_DIFF_MEMBERSHIP_SCRATCH_BENCH_V1` marker authored. | implemented_pending_validation |

## 性能边界

The one-million-event model reduces large-path membership-table allocations
from one per event to one warm allocation. This deterministic model and the
local source contracts do not substitute for managed Windows Release
allocation or pointer-input p50/p95/p99 evidence.

## Local validation notes

Scoped `rustfmt --check` for the two Runtime production files passes. A
workspace-wide `cargo fmt --all -- --check` only reports five unrelated
formatting diffs in pre-existing/foreign Runtime test and host-output files;
those files were not rewritten.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/surface.rs` | `26FA9866478B63BE830181ED3F126B946935DBBE86A550C25385ADC804650CC7` |
| `zircon_runtime/src/ui/surface/surface/event_routing.rs` | `DA85539C6EEED66D65EB991F3C8078CB1F8AECE8F2A99C0FD703C95A6D9CDB36` |
| `tools/tests/test_runtime_pointer_hover_membership_scratch_performance_contract.py` | `4A960805A0816DAC172B7C1891699ADE5B46EB60AB7CB10C5632E3AE27E546BD` |

## 受管验证

The slice is admitted to the existing batched Runtime/Editor validation lane;
no per-task Cargo run, coordinator request, or status query was issued.
Managed admission remains blocked by the already recorded external
`E:\Git\zr_vm` dirty-worktree and ownership/overlay guards. Until a managed
Release receipt and benchmark gate are available, this row stays
`implemented_pending_validation`.
