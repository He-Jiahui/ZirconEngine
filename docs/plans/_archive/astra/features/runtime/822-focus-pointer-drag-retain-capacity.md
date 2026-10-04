---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-19-focus-pointer-drag-retain-capacity.md
related_code:
  - zircon_runtime/src/ui/surface/focus.rs
tests:
  - zircon_runtime/src/ui/surface/focus.rs
  - tools/tests/test_runtime_focus_pointer_drag_retain_performance_contract.py
---

# Runtime822 Focus Pointer-Drag Owner Retention

`UiSurface` now removes invalid pointer-drag owners by retaining the existing
`BTreeMap` after a tree change. The old temporary owner vector and per-owner
second-pass removals are gone, while valid drag state and input-owner
semantics remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime822 | Replace the invalid pointer-drag owner `keys().filter().collect()` plus removal loop with `mem::take` and `BTreeMap::retain` | implemented_pending_validation | TDD source/model contract `4/4`; lower semantic regression and ignored `RUNTIME822_FOCUS_POINTER_DRAG_RETAIN_BENCH_V1` marker are wired; the current one-process non-tooling Runtime/Editor batch passes `2218/2218` across `621` modules with zero failures/errors/skips, and the latest shared batch passes `2227/2227` across `624` modules; the 4,096-entry deterministic model removes one temporary vector per reconciliation and changes repeated removal lookups to one map traversal; managed Cargo/Release and input product p50/p95/p99 evidence remain pending. |

## Complexity boundary

This is a local transient-input cleanup change. It does not alter focus
ownership, pointer capture, drag metrics, pointer-drag payloads, validation
order, or the published UI input contract. `BTreeMap::retain` keeps surviving
entries and their ordering while dropping only invalid owners.

## Managed gate

The source/model contract is locally green and joins the next batched
Runtime/Editor validation. No standalone Cargo command was started. Managed
Windows Release compilation and product allocation/latency evidence remain
pending behind the external dirty `E:\Git\zr_vm` admission gate; no coordinator
status was polled.
