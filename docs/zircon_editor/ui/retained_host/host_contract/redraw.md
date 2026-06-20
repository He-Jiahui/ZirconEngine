---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw/request.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw/dispatch_result.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/redraw_result.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw/request.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw/dispatch_result.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract redraw request/dispatch/test ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Host Contract Redraw

`redraw.rs` is the retained-host redraw contract entry. It exports the two shared redraw types used by native window, native pointer, native keyboard, popup dismiss, text input, and event-loop present paths while keeping their concrete behavior in child owners.

## Request Ownership

`redraw/request.rs` owns `HostRedrawRequest`. It defines the `None`, `Full`, and `Region` request shapes, full-frame and region constructors, visible-frame filtering, performance-counter recording, frame-update flags, damage-region access, scenario access, and coalescing rules.

The merge rule intentionally keeps damage unioning and frame-update preservation in one place. When two frame-update requests coalesce, the later frame-update scenario wins so presenter counters remain attributed to the interaction that produced the final retained frame.

## Dispatch Ownership

`redraw/dispatch_result.rs` owns `NativePointerDispatchResult`, the thin wrapper that pointer, keyboard, popup, and text paths return to the event loop. It converts idle, full-frame, paint-only region, and frame-update region outcomes into `HostRedrawRequest`, while preserving the public query helpers used by tests and event-loop scheduling.

The native pointer subtree still owns route-specific damage calculation in `native_pointer/redraw_result.rs`; this module owns only the shared result container and conversion to the queued redraw request.

## Root Boundary

The root `redraw.rs` now only declares the children, re-exports `HostRedrawRequest` and `NativePointerDispatchResult`, and attaches the external test module. It should not regain merge logic, performance counters, pointer result constructors, or inline tests.

## Test Ownership

`redraw_tests.rs` owns module-local regressions for damage unioning, full-frame override behavior, frame-update region requests, preserved frame-update bits, and latest-scenario attribution. The tests stay outside production files so the redraw contract root remains a narrow type-export boundary.

## Validation Notes

This slice used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `redraw.rs` no longer owns request/result bodies or inline tests, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.
