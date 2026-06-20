---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/hit_samples.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/mod.rs
  - tools/ui-profile-capture.ps1
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/hit_samples.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - profiling artifact export/environment/schema ownership scan
  - profiling artifact geometry ownership scan
  - profiling artifact geometry subtree ownership scan
  - profiling hit-route subtree ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Profiling Artifacts

`profiling_artifacts.rs` is the retained-host profiling artifact boundary. It is now a structural entry file that wires export, environment, schema, geometry, and module-local tests while preserving the single outward entry `export_present_artifacts(...)`.

`profiling_artifacts/export.rs` owns the export workflow: check the capture switches, create the session directory, serialize `ui_profile_geometry.json`, and optionally write `screenshot_reference.png` from the presenter snapshot path. `profiling_artifacts/environment.rs` owns the environment contract for `ZIRCON_PROFILE_CAPTURE`, `ZIRCON_PROFILE_CAPTURE_SCREENSHOTS`, `ZIRCON_PROFILE_FORCE_SOFTBUFFER`, `ZIRCON_PROFILE_OUTPUT_ROOT`, and `ZIRCON_PROFILE_SESSION`, including session-id sanitization.

`profiling_artifacts/schema.rs` owns the serializable JSON payload records. Keeping the DTOs separate from the writer makes the artifact schema auditable without letting the root module accumulate export control flow and data declarations again.

## Geometry Ownership

`profiling_artifacts/geometry.rs` owns `UiProfileGeometry::from_presentation(...)` and the top-level geometry extraction assembly. It collects resize splitters directly, delegates tab frame collection to `geometry/tabs.rs`, delegates activity rail, pane, viewport-toolbar, template-node, and surface-frame control collection to `geometry/pane_frames.rs`, delegates route sample construction to `geometry/hit_samples.rs`, and delegates shared visibility, translation, intersection, center-point, and profile-frame push helpers to `geometry/frame_math.rs`.

The geometry subtree still owns the exact payload consumed by `tools/ui-profile-capture.ps1`: resize splitters, document/drawer/floating tabs, host-page tabs, activity-rail buttons, viewport frames, viewport-toolbar controls, dispatchable template controls, clickable frames, and route-hit samples. Surface-frame top-hit filtering remains private to the pane-frame child so profiling export does not expose runtime UI hit-test internals as a public API.

## Route-hit Ownership

`profiling_hit_routes.rs` owns the route-hit entry used to validate whether a captured profile frame still resolves at a given point. It is now a structural dispatcher only: the root receives the serialized route kind/id/surface tuple and delegates to the concrete route-family owner.

`profiling_hit_routes/chrome.rs` owns resize splitter and activity-rail button hit checks. `tabs.rs` owns document, drawer, floating-window, and host-page tab checks. `viewport_toolbar.rs` owns Scene/Game viewport toolbar hit checks through the existing surface hit-test helpers. `template.rs` owns template-control hit checks through runtime surface-frame hit testing. `geometry.rs` owns shared containment, translation, side-dock content frame, and floating-window content frame math.

This split keeps the profiling route matcher aligned with the profiling geometry subtree: top-level route dispatch remains easy to audit, while each chrome/pane/template hit family can evolve independently without reopening a single large matcher file.

## Test Ownership

`profiling_artifacts/tests.rs` owns module-local regressions that require private geometry helpers and private DTO fields. It covers absolute splitter/tab frame export, template control clipping, viewport-toolbar clipping, and top-hit filtering.

The tests remain in a child module rather than a public API because these checks freeze internal capture evidence semantics, not a crate-level contract.

## Validation Notes

The 2026-06-18 geometry split is implementation-first. Evidence for this slice is formatting, ownership scans, trailing-whitespace/diff checks, and scoped `zircon_editor` library type checks. Full Cargo test matrix and live profiling capture remain deferred to the milestone validation stage per the user's instruction.

The 2026-06-20 export/environment/schema split reduced `profiling_artifacts.rs` to a structural module entry. The new child owners are `export.rs` for artifact writes and screenshot capture, `environment.rs` for capture switches and export path selection, and `schema.rs` for the serialized profile DTOs. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; Cargo compile/test validation remains deferred because current package checks are blocked by unrelated runtime render-history errors before editor diagnostics.

The 2026-06-18 geometry subtree split reduced `profiling_artifacts/geometry.rs` from 614 lines to 155 lines. The new child owners are `geometry/frame_math.rs` at 112 lines, `geometry/hit_samples.rs` at 61 lines, `geometry/pane_frames.rs` at 235 lines, and `geometry/tabs.rs` at 84 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a geometry subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 profiling hit-route subtree split reduced `profiling_hit_routes.rs` from 388 lines to 29 lines. The new route-family owners are `profiling_hit_routes/tabs.rs` at 128 lines, `template.rs` at 100 lines, `viewport_toolbar.rs` at 98 lines, `chrome.rs` at 54 lines, and `geometry.rs` at 37 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a profiling hit-route subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.
