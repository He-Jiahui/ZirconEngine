---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/invalidation.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/model/counters.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/overlay_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/timing.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/model/counters.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/overlay_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/timing.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract diagnostics invalidation/overlay/refresh/test ownership scan
  - host_contract diagnostics refresh model/timing/overlay-text ownership scan
  - host_contract diagnostics refresh counter method ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Host Contract Diagnostics

`diagnostics.rs` is the retained-host diagnostics boundary. It now stays as a structural entry that re-exports the public crate-facing diagnostics records and attaches the module-local regression tests.

`diagnostics/invalidation.rs` owns `HostInvalidationDiagnostics`, the small counter snapshot passed from invalidation scheduling into presenter diagnostics. It tracks slow-path rebuilds, render rebuilds, and paint-only requests without depending on presenter or window types.

`diagnostics/overlay.rs` owns `STARTUP_REFRESH_DIAGNOSTICS_OVERLAY`, the zero-present baseline text shown before live refresh counters exist. Keeping the string in one owner prevents presenter/window startup surfaces from drifting apart.

`diagnostics/refresh.rs` is now the structural refresh diagnostics entry. `refresh/model.rs` owns the `HostRefreshDiagnostics` record and private timing fields, while `refresh/model/counters.rs` owns present-count accumulation, full/region paint counters, painted-pixel totals, invalidation counter merging, FPS forwarding, and overlay text forwarding. `refresh/timing.rs` owns first/last present timestamp mutation and FPS calculation, while `refresh/overlay_text.rs` owns startup/live overlay text formatting. The first/last present timestamps remain private to the refresh subtree so callers can read counters but cannot mutate timing state directly.

`diagnostics_tests.rs` owns the module-local regressions for overlay text changes, full-vs-region counter increments, invalidation counter text, and startup overlay parity. These tests remain module-local because they validate private retained-host formatting semantics rather than a public workspace API.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.

The 2026-06-21 refresh model/timing/overlay-text split reduced `diagnostics/refresh.rs` from 104 lines to a 5-line structural re-export entry. `refresh/model.rs` owns the diagnostics record and counter mutations, `refresh/timing.rs` owns present timestamp/FPS helpers, and `refresh/overlay_text.rs` owns overlay string formatting. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a diagnostics refresh model/timing/overlay-text ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 refresh counter method split reduced `diagnostics/refresh/model.rs` from 80 lines to a 30-line diagnostics record owner. `refresh/model/counters.rs` owns present counter mutation, invalidation counter merging, FPS forwarding, and overlay text forwarding. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a diagnostics refresh counter method ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
