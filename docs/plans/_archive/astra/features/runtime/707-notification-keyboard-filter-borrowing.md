---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
tests:
  - zircon_runtime/src/ui/component/state_reducer/notification_center.rs
---

# Runtime Notification Keyboard Filter Borrowing

The notification reducer already bounded keyboard navigation to the visible
notification collector, but then copied every enabled entry reference into a
second temporary `Vec`. Navigation now searches the bounded owned entries
directly through one helper. First/last selection, disabled filtering, current
index fallback, logical ordering, and unread-count updates are unchanged; the
keyboard path loses one allocation and one `O(L)` reference materialization per
action, where `L` is the visible limit.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / Runtime11A notification interaction | Borrow enabled-entry selection directly from the bounded visible list. | Notification visibility/materialization contracts pass; a Rust regression covers disabled rows and First/Last/Next/Previous semantics; scoped Rustfmt and diff checks pass. | implemented_pending_validation |

## Batched local evidence

- Notification visible-materialization and popup-render contracts pass (`5/5` in
  the focused batch).
- `keyboard_navigation_filters_disabled_entries_without_materializing_a_second_list`
  covers disabled rows, first/last navigation, and relative next/previous
  selection without constructing a second enabled-entry vector.
- Current source snapshot SHA-256:
  `3017412F826CB56E6A956ACAC39AEFFE9D4592D5CEAFA50DA04EE6F80AC7A9BC`.
- Scoped Rustfmt and `git diff --check` pass; only the existing LF/CRLF notice
  is emitted.

This is source/contract evidence, not managed Cargo execution or product
notification-storm CPU/allocation/RSS/latency p50/p95/p99 evidence.

## Managed acceptance gate

The owner-attributed Windows Release gate remains deferred by the external
`E:\\Git\\zr_vm` dirty-worktree admission recorded in `696`. No coordinator
state was polled in this slice. Keep this row at
`implemented_pending_validation` until batched managed Runtime tests and
Release measurements validate the current source. Tooling changes remain
deferred by request.
