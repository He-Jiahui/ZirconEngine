---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/invalidation.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/invalidation.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract diagnostics invalidation/overlay/refresh/test ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Host Contract Diagnostics

`diagnostics.rs` is the retained-host diagnostics boundary. It now stays as a structural entry that re-exports the public crate-facing diagnostics records and attaches the module-local regression tests.

`diagnostics/invalidation.rs` owns `HostInvalidationDiagnostics`, the small counter snapshot passed from invalidation scheduling into presenter diagnostics. It tracks slow-path rebuilds, render rebuilds, and paint-only requests without depending on presenter or window types.

`diagnostics/overlay.rs` owns `STARTUP_REFRESH_DIAGNOSTICS_OVERLAY`, the zero-present baseline text shown before live refresh counters exist. Keeping the string in one owner prevents presenter/window startup surfaces from drifting apart.

`diagnostics/refresh.rs` owns `HostRefreshDiagnostics`, present-count accumulation, full/region paint counters, painted-pixel totals, invalidation counter merging, FPS calculation, and overlay text formatting. The first/last present timestamps remain private to the refresh owner so callers can read counters but cannot mutate timing state directly.

`diagnostics_tests.rs` owns the module-local regressions for overlay text changes, full-vs-region counter increments, invalidation counter text, and startup overlay parity. These tests remain module-local because they validate private retained-host formatting semantics rather than a public workspace API.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
