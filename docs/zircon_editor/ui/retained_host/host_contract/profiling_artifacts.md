---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/activity_rail_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/clickable_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/drawer_tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/center.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/intersection.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/named.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/translation.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/hit_samples.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/activity_rail.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/surface_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/surface_frame/controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes/dispatchable.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes/source.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_profile_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/resize_splitters.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/tabs/collect.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/named.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/tab.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/document.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/drawer.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/host_page.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/shared.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/mod.rs
  - tools/ui-profile-capture.ps1
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/activity_rail_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/clickable_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/drawer_tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/center.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/intersection.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/named.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/translation.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/hit_samples.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/activity_rail.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/surface_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/surface_frame/controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes/dispatchable.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes/source.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_profile_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/resize_splitters.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/tabs/collect.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/named.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/tab.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/document.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/drawer.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/host_page.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/shared.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - profiling artifact export/environment/schema ownership scan
  - profiling artifact schema frame/geometry/hit/named/tab ownership scan
  - profiling artifact geometry ownership scan
  - profiling artifact geometry subtree ownership scan
  - profiling artifact geometry assembly subowner ownership scan
  - profiling artifact geometry frame-math subowner ownership scan
  - profiling artifact pane-frame activity/content/pane/surface/template ownership scan
  - profiling artifact pane-frame surface controls ownership scan
  - profiling artifact pane-frame template-node source/dispatchable ownership scan
  - profiling hit-route subtree ownership scan
  - profiling hit-route tab-family ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Profiling Artifacts

`profiling_artifacts.rs` is the retained-host profiling artifact boundary. It is now a structural entry file that wires export, environment, schema, geometry, and module-local tests while preserving the single outward entry `export_present_artifacts(...)`.

`profiling_artifacts/export.rs` owns the export workflow: check the capture switches, create the session directory, serialize `ui_profile_geometry.json`, and optionally write `screenshot_reference.png` from the presenter snapshot path. `profiling_artifacts/environment.rs` owns the environment contract for `ZIRCON_PROFILE_CAPTURE`, `ZIRCON_PROFILE_CAPTURE_SCREENSHOTS`, `ZIRCON_PROFILE_FORCE_SOFTBUFFER`, `ZIRCON_PROFILE_OUTPUT_ROOT`, and `ZIRCON_PROFILE_SESSION`, including session-id sanitization.

`profiling_artifacts/schema.rs` is now the structural serializable payload entry. `schema/frame.rs` owns profile frame, point, size, and root layout records; `schema/geometry.rs` owns the top-level `UiProfileGeometry` document; `schema/named.rs` owns named frame records plus tab-to-frame conversion; `schema/tab.rs` owns tab frame records; and `schema/hit.rs` owns hit sample records. Keeping these DTOs separate from the writer makes the artifact schema auditable without letting the root module accumulate export control flow and data declarations again.

## Geometry Ownership

`profiling_artifacts/geometry.rs` owns `UiProfileGeometry::from_presentation(...)` and now acts as the top-level profile payload assembly entry. It pulls each field from a narrow child owner and keeps only the final `UiProfileGeometry` construction, presenter-backend label, window-size projection, and viewport-frame handoff in the parent.

`geometry/resize_splitters.rs` owns splitter frame capture. `drawer_tabs.rs` owns side/bottom/floating drawer tab aggregation while `tabs.rs` owns document/side/bottom/floating/host-page tab source selection and `tabs/collect.rs` owns the shared per-tab row projection. `activity_rail_buttons.rs` owns left/right rail button aggregation. `pane_profile_controls.rs` owns document/side/bottom/floating pane traversal for viewport-toolbar and dispatchable template-control frames. `clickable_frames.rs` owns the serialized clickable-frame aggregation order, `layout.rs` owns layout DTO projection, and `hit_samples.rs` owns route sample construction for every clickable frame.

`geometry/frame_math.rs` is now a structural geometry-math entry. `frame_math/visibility.rs` owns finite positive frame gating and visible profile-frame conversion, `translation.rs` owns absolute-frame translation for common and template-node frames, `intersection.rs` owns frame clipping, `center.rs` owns sample point centers, and `named.rs` owns `UiProfileNamedFrame` emission after visibility checks.

`geometry/pane_frames.rs` is now a structural entry. `pane_frames/activity_rail.rs` owns side activity-rail button frame collection, `content.rs` owns side-dock and floating-window content frame math, `pane.rs` owns pane-level body/toolbar/template-control sequencing, `surface_frame.rs` owns the optional surface-frame guard and test-visible entry, and `surface_frame/controls.rs` owns surface-frame pointer node iteration, clipping, top-hit filtering, and profile-frame emission. `template_nodes.rs` owns template-control iteration and clipped profile-frame projection, `template_nodes/source.rs` owns pane-kind to template-node model source selection, and `template_nodes/dispatchable.rs` owns the dispatchable-template-control predicate.

The geometry subtree still owns the exact payload consumed by `tools/ui-profile-capture.ps1`: resize splitters, document/drawer/floating tabs, host-page tabs, activity-rail buttons, viewport frames, viewport-toolbar controls, dispatchable template controls, clickable frames, and route-hit samples. Surface-frame top-hit filtering remains private to the pane-frame child so profiling export does not expose runtime UI hit-test internals as a public API.

## Route-hit Ownership

`profiling_hit_routes.rs` owns the route-hit entry used to validate whether a captured profile frame still resolves at a given point. It is now a structural dispatcher only: the root receives the serialized route kind/id/surface tuple and delegates to the concrete route-family owner.

`profiling_hit_routes/chrome.rs` owns resize splitter and activity-rail button hit checks. `tabs.rs` is now a structural tab-route entry: `tabs/document.rs` owns document-dock tab checks, `drawer.rs` owns left/right/bottom drawer tab checks, `floating.rs` owns floating-window tab checks, `host_page.rs` owns host-page tab checks, and `shared.rs` owns common tab row containment. `viewport_toolbar.rs` owns Scene/Game viewport toolbar hit checks through the existing surface hit-test helpers. `template.rs` owns template-control hit checks through runtime surface-frame hit testing. `geometry.rs` owns shared containment, translation, side-dock content frame, and floating-window content frame math.

This split keeps the profiling route matcher aligned with the profiling geometry subtree: top-level route dispatch remains easy to audit, while each chrome/pane/template hit family can evolve independently without reopening a single large matcher file.

## Test Ownership

`profiling_artifacts/tests.rs` owns module-local regressions that require private geometry helpers and private DTO fields. It covers absolute splitter/tab frame export, template control clipping, viewport-toolbar clipping, and top-hit filtering.

The tests remain in a child module rather than a public API because these checks freeze internal capture evidence semantics, not a crate-level contract.

## Validation Notes

The 2026-06-18 geometry split is implementation-first. Evidence for this slice is formatting, ownership scans, trailing-whitespace/diff checks, and scoped `zircon_editor` library type checks. Full Cargo test matrix and live profiling capture remain deferred to the milestone validation stage per the user's instruction.

The 2026-06-20 export/environment/schema split reduced `profiling_artifacts.rs` to a structural module entry. The new child owners are `export.rs` for artifact writes and screenshot capture, `environment.rs` for capture switches and export path selection, and `schema.rs` for the serialized profile DTOs. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; Cargo compile/test validation remains deferred because current package checks are blocked by unrelated runtime render-history errors before editor diagnostics.

The 2026-06-21 schema frame/geometry/hit/named/tab split reduced `profiling_artifacts/schema.rs` from 106 lines to a 12-line structural entry. `schema/frame.rs` owns scalar frame/point/size/layout DTOs and `FrameRect` conversions, `schema/geometry.rs` owns the top-level profile document, `schema/named.rs` owns named frame records plus tab conversion, `schema/tab.rs` owns tab frames, and `schema/hit.rs` owns hit samples. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a profiling artifact schema frame/geometry/hit/named/tab ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-18 geometry subtree split reduced `profiling_artifacts/geometry.rs` from 614 lines to 155 lines. The new child owners are `geometry/frame_math.rs` at 112 lines, `geometry/hit_samples.rs` at 61 lines, `geometry/pane_frames.rs` at 235 lines, and `geometry/tabs.rs` at 84 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a geometry subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 profiling hit-route subtree split reduced `profiling_hit_routes.rs` from 388 lines to 29 lines. The new route-family owners are `profiling_hit_routes/tabs.rs` at 128 lines, `template.rs` at 100 lines, `viewport_toolbar.rs` at 98 lines, `chrome.rs` at 54 lines, and `geometry.rs` at 37 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a profiling hit-route subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-21 profiling hit-route tab-family split reduced `profiling_hit_routes/tabs.rs` from 128 lines to a 9-line structural entry. The new child owners are `tabs/document.rs`, `drawer.rs`, `floating.rs`, `host_page.rs`, and `shared.rs`, preserving the existing route helper names consumed by `profiling_hit_routes/routes.rs`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a tab-family ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

The 2026-06-21 pane-frame subtree split reduced `profiling_artifacts/geometry/pane_frames.rs` from 252 lines to a 12-line structural entry. The new child owners are `activity_rail.rs` at 37 lines, `content.rs` at 23 lines, `pane.rs` at 36 lines, `surface_frame.rs` at 79 lines, and `template_nodes.rs` at 77 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a pane-frame activity/content/pane/surface/template ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check was not relaunched because the current editor package check lane has been timing out before actionable editor diagnostics, and full Cargo tests remain deferred per the user's instruction.

The 2026-06-21 pane-frame surface controls split reduced `profiling_artifacts/geometry/pane_frames/surface_frame.rs` from 79 lines to a 33-line optional surface-frame entry. `surface_frame/controls.rs` owns pointer-capable node traversal, host-space frame and clip translation, center-point top-hit verification, and named profile-frame emission. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a pane-frame surface controls ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred to the milestone testing stage per the user's feature-first instruction.

The 2026-06-21 pane-frame template-node source/dispatchable split reduced `profiling_artifacts/geometry/pane_frames/template_nodes.rs` from 77 lines to a 55-line template-control collection entry. `template_nodes/source.rs` owns pane-kind source selection across Hierarchy, Inspector, Assets, Welcome, diagnostics, export, UI Asset, and animation panes, while `template_nodes/dispatchable.rs` owns the enabled control/action/edit/commit/input predicate. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a pane-frame template-node source/dispatchable ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred to the milestone testing stage per the user's feature-first instruction.

The 2026-06-21 geometry assembly split reduced `profiling_artifacts/geometry.rs` from 164 lines to a 65-line retained profile assembly entry. The new child owners are `activity_rail_buttons.rs`, `clickable_frames.rs`, `drawer_tabs.rs`, `layout.rs`, `pane_profile_controls.rs`, and `resize_splitters.rs`, while `hit_samples.rs` gained the clickable-frame sample aggregation entry. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a geometry assembly subowner ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

The 2026-06-21 geometry tabs collection split reduced `profiling_artifacts/geometry/tabs.rs` from
88 lines to a 63-line tab source-selection entry. `tabs/collect.rs` owns the shared tab row loop,
visibility filtering, origin translation, close-frame translation, and `UiProfileTabFrame`
construction. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`,
a profiling artifact geometry tabs collection ownership scan, scoped trailing-whitespace scan, and
scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the
user's feature-first instruction.

The 2026-06-21 geometry frame-math split reduced `profiling_artifacts/geometry/frame_math.rs` from 138 lines to a 20-line structural re-export entry. The new child owners are `frame_math/{center,intersection,named,translation,visibility}.rs`; they keep the existing geometry helper names stable for downstream pane-frame, tab, splitter, and hit-sample owners. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a geometry frame-math subowner ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred to the milestone testing stage per the user's instruction.
