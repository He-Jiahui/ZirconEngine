---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor535-unchanged-status-line-borrow.md
implementation_files:
  - zircon_editor/src/ui/host/editor_event_runtime_access/status.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/status.rs
tests:
  - zircon_editor/src/ui/host/editor_event_runtime_access/status.rs
---

# Editor963 Editor535 unchanged status-line borrow

Status publication now compares a borrowed `&str` under the existing shell
lock and allocates only after the value changes. Invalidation and bridge
refresh behavior are preserved. Marker
`EDITOR535_UNCHANGED_STATUS_LINE_BORROW_BENCH_V1` models 32,768 unchanged
updates with zero optimized pre-comparison clones. Managed Editor Release
validation remains pending.
