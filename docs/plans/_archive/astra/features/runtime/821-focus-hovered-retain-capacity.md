---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-19-focus-hovered-retain-capacity.md
related_code:
  - zircon_runtime/src/ui/surface/focus.rs
tests:
  - zircon_runtime/src/ui/surface/focus.rs
  - tools/tests/test_runtime_focus_hovered_retain_performance_contract.py
---

# Runtime821 Focus Hovered-Path Retention

`UiSurface` now filters its invalidated hovered path in place after a tree
change. The existing `Vec` capacity is retained, while input-owner validation,
order, duplicates, and empty behavior remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime821 | Replace the hovered-path replacement `filter().collect()` with `mem::take` plus in-place `retain` | implemented_pending_validation | TDD source/model contract `4/4`; lower order/capacity regression and ignored `RUNTIME821_FOCUS_HOVERED_RETAIN_BENCH_V1` marker are wired; the one-process Runtime/Editor batch passes `1416/1416` across `393` modules and the broader non-tooling batch passes `2210/2210` across `619` modules, all with zero failures/errors/skips; the 4,096-entry deterministic model changes one replacement allocation per reconciliation to zero new allocations; managed Cargo/Release and input product p50/p95/p99 evidence remain pending. |

## Complexity boundary

This is a local retained-buffer change. It does not alter focus ownership,
pointer capture, drag state, input-method lifecycle, validation order, or the
published UI input contract. The retained capacity is bounded by the prior hit
path and is intentionally kept for the next reconciliation.

## Managed gate

The source/model contract is locally green and joins the next batched
Runtime/Editor validation. No standalone Cargo command was started. Managed
Windows Release compilation and product allocation/latency evidence remain
pending behind the external dirty `E:\Git\zr_vm` admission gate; no coordinator
status was polled.
