---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app host-lifecycle module-plugin projection ownership scan
  - app host-lifecycle module-plugin pane-data ownership scan
  - app module-plugin projection rows ownership scan
  - app host-lifecycle viewport-toolbar projection ownership scan
  - app host-lifecycle build-export projection ownership scan
  - app host-lifecycle native-window presenter ownership scan
  - app host-lifecycle effects/invalidation ownership scan
  - app host-lifecycle render-submission ownership scan
  - app host-lifecycle pane-payload ownership scan
  - app host-lifecycle recompute viewport ownership scan
  - app host-lifecycle startup ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Lifecycle

`app/host_lifecycle.rs` owns retained editor host startup, tick scheduling, dirty recompute orchestration, presentation publication, and lifecycle call sites for status/effect/invalidation work. It should stay focused on host lifecycle flow and on the state transitions that decide when layout, presentation, render, and paint-only work run.

The file is still large because it is the bridge between `RetainedEditorHost` state, workbench model snapshots, native window projection, viewport extract submission, and retained host presentation. New feature-specific projection helpers should move into app child modules when they are pure data shaping and do not own lifecycle sequencing.

## Startup

`app/host_lifecycle/startup.rs` owns retained editor host construction. It resolves runtime/editor/resource managers, subscribes asset/editor/resource change receivers, resolves the startup session and initial `EditorState`, loads shared builtin template runtimes, builds all startup template bridges, constructs the native plugin live host and play-mode backend, initializes pointer bridges, scroll surfaces, dirty flags, invalidation root, and startup viewport size, then performs the initial asset workspace sync and bootstrap refresh diagnostics publication.

`host_lifecycle.rs` keeps the lifecycle methods that run after construction: `tick`, `refresh_ui`, shell-size synchronization, recompute orchestration, render submission, viewport-image polling, and pane-payload collection. Startup request interpretation and test-only startup-state fallback live with construction so the runtime tick path does not carry startup policy.

## Module Plugin Projection

`app/module_plugin_projection.rs` owns the module-plugin panel projection method. It reads plugin status for the active project, falls back to the builtin plugin catalog when the project manifest is unavailable, aggregates diagnostics, and builds `ModulePluginsPaneViewData` rows.

`app/module_plugin_projection/rows.rs` owns the pure presentation helpers for those rows: primary action labels/ids, optional feature action labels/ids, optional feature dependency summaries, target-mode labels, packaging labels, and the fallback project manifest.

`host_lifecycle.rs` calls `RetainedEditorHost::module_plugins_pane_data(...)` during recompute, but the method implementation lives in `module_plugin_projection.rs` and row formatting lives under its child module. This keeps lifecycle orchestration from accumulating panel-specific catalog fallback, row mapping, label formatting, action-id construction, and feature-summary logic.

## Build Export Projection

`app/build_export_projection.rs` owns the Build/Export panel projection method and output-path diagnostic formatting. It loads the active project manifest, merges the desktop export profiles exposed by `app/build_export_actions.rs`, asks `EditorManager` for native-aware export plans, applies queued/running/completed job state onto `BuildExportTargetViewData`, and publishes the wizard view model for the first target.

`host_lifecycle.rs` still decides when the Build/Export pane participates in recompute and when build/export actions are dispatched. The projection module owns panel data shaping so lifecycle code does not carry target row construction, output diagnostics, report overlay, job overlay, or manifest-load presentation errors.

## Viewport Toolbar Projection

`app/viewport_toolbar_projection.rs` owns viewport-toolbar surface-frame attachment for docked Scene/Game panes and floating Scene/Game windows. It clones the current host presentation, computes per-pane toolbar sizes, attaches runtime surface frames through `BuiltinViewportToolbarTemplateBridge`, maps projection control ids to stable editor action ids, rebuilds the floating-window list with updated toolbar frames, and publishes the updated host presentation.

`host_lifecycle.rs` still decides when toolbar frames must be attached during recompute and native-window synchronization. The helper module owns the projection details so lifecycle code does not carry per-control action-id mapping or floating-window surface-frame rewriting.

## Native Window Presenters

`app/host_lifecycle/native_window_presenters.rs` owns native floating-window presenter synchronization after the main workbench model, chrome, geometry, pane payloads, runtime diagnostics, and floating-window projection bundle have been computed. It collects native floating-window targets, handles empty-target presenter cleanup, prepares per-window module-plugin/build-export pane payloads, wires callbacks for newly-created native windows, attaches close handling, applies retained host presentation into each native window, attaches viewport-toolbar surface frames for native windows, and configures native floating-window metadata.

`host_lifecycle.rs` still decides when native-window presenter synchronization runs during recompute. The child module owns the presenter details so lifecycle code does not carry target collection, native-window callback setup, per-window payload projection, or presentation application.

## Effects And Invalidation Bridge

`app/host_lifecycle/dispatch_effects.rs` owns retained-host status mutations and `UiHostEventEffects` application. It preserves the app-internal `RetainedEditorHost` methods for setting the status line/task progress, applying dispatch results, applying active-recompute viewport resize effects, forwarding workbench notifications, refreshing asset views, importing models, opening the command palette, and presenting the welcome surface.

`app/host_lifecycle/invalidation_bridge.rs` owns the retained-host bridge from dirty-domain masks to legacy dirty flags and UI perf counters. It captures the pending UI perf scenario, records dirty layout/presentation/render/paint-only counters, mutates `HostInvalidationRoot`, exposes app-internal layout/presentation/render mark helpers, and publishes paint-only invalidation diagnostics.

## Render Submission

`app/host_lifecycle/render_submission.rs` owns the render-path submit step that runs from `tick` when `render_dirty` is set. It consumes pending render invalidation reasons, records render rebuild diagnostics and UI perf counters, submits the runtime extract/UI bundle to the viewport backend, preserves `render_dirty` when the lazy viewport backend is not ready, and requests a non-reentrant viewport-content frame update for retry.

`host_lifecycle.rs` still owns tick sequencing and decides when render submission runs. The child module owns render-path side effects so `tick` does not accumulate backend readiness, diagnostics, and retry scheduling details.

## Pane Payloads

`app/host_lifecycle/pane_payloads.rs` owns the recompute payload collection that feeds retained host presentation and native floating-window presentation. It collects preset names, UI Asset pane presentations, Animation Editor pane presentations, runtime diagnostics snapshots, Module Plugins pane data, and Build/Export pane data behind `RetainedEditorHost::collect_host_lifecycle_pane_payloads(...)`.

`host_lifecycle.rs` still owns recompute sequencing, presentation assembly, viewport toolbar attachment, and native-window synchronization. The child module owns cross-pane payload gathering so lifecycle orchestration does not carry per-pane snapshot loops, active-document filtering, UI Asset/Animation editor projection, or module/build-export pane-data construction.

## Recompute Viewport

`app/host_lifecycle/recompute_viewport.rs` owns the viewport and pointer-bridge substep inside dirty recompute. It derives the current viewport content frame from componentized workbench layout frames, dispatches viewport resize effects when the viewport size changes, rebuilds chrome/model snapshots after resize effects, updates the viewport pointer frame, updates the shell pointer bridge with workbench layout frames plus floating-window projection, and syncs activity rail, host page, document tab, and drawer header pointer layouts.

`host_lifecycle.rs` still owns the recompute order and passes the mutable model/chrome pair through this substep before pane payload collection and presentation publication.

## Boundary Rules

- Keep `tick`, `refresh_ui`, `recompute_if_dirty`, and the native-window/effect/invalidation call sites in `host_lifecycle.rs`.
- Keep retained-host construction, startup session resolution, manager/resource resolution, startup bridge creation, initial pointer/surface state initialization, startup asset sync, and startup-state test fallback in `app/host_lifecycle/startup.rs`.
- Keep `tick`, `refresh_ui`, `recompute_if_dirty`, render submission, and recompute call sites in `app/host_lifecycle.rs`.
- Keep retained-host status mutation and `UiHostEventEffects` side-effect application in `app/host_lifecycle/dispatch_effects.rs`.
- Keep retained-host dirty-domain to invalidation-root/legacy dirty-flag bridging in `app/host_lifecycle/invalidation_bridge.rs`.
- Keep render-dirty extract submission, render-path diagnostics, backend-not-ready retry, and viewport-content frame update scheduling in `app/host_lifecycle/render_submission.rs`.
- Keep recompute pane payload collection, including preset/runtime diagnostics/UI Asset/Animation/Module Plugins/Build Export payload gathering, in `app/host_lifecycle/pane_payloads.rs`.
- Keep recompute viewport resize handling, post-resize chrome/model rebuild, viewport pointer frame updates, shell pointer bridge layout updates, and shell pointer layout sync calls in `app/host_lifecycle/recompute_viewport.rs`.
- Keep native floating-window presenter target sync, callback setup, per-window pane payload projection, and presentation application in `app/host_lifecycle/native_window_presenters.rs`.
- Keep Build/Export action execution, async job queue state, and output folder mutation in `app/build_export_actions.rs`.
- Keep Build/Export pane-data projection and output diagnostic formatting in `app/build_export_projection.rs`; do not add target-row construction or report/job overlay logic back to `host_lifecycle.rs`.
- Keep module plugin action execution and callback handling in `app/module_plugin_actions.rs`.
- Keep module plugin pane-data status lookup, diagnostics aggregation, and view-model construction in `app/module_plugin_projection.rs`; do not add catalog fallback or row mapping back to `host_lifecycle.rs`.
- Keep module plugin row label/action-id, target/packaging text, feature-summary, fallback-manifest helper, and helper tests in `app/module_plugin_projection/rows.rs`.
- Keep viewport toolbar surface-frame attachment and projection-control action-id mapping in `app/viewport_toolbar_projection.rs`.
- Keep invalidation mask ownership and dirty-domain policy in `app/invalidation.rs` plus the lifecycle bridge methods documented in `app/invalidation.md`.
- Move future pure pane projection helper families into focused child modules before `host_lifecycle.rs` grows another panel-specific formatting section.

## Validation Notes

The 2026-06-18 module-plugin projection split reduced `host_lifecycle.rs` from 1987 lines to 1739 lines. `module_plugin_projection.rs` is 257 lines and owns module plugin panel label/action/summary/fallback helpers plus the moved module-plugin projection tests. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle module-plugin projection ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 viewport-toolbar projection split reduced `host_lifecycle.rs` to 1523 lines. `viewport_toolbar_projection.rs` is 220 lines and owns toolbar surface-frame attachment for docked and floating Scene/Game panes plus projection-control action-id mapping. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle viewport-toolbar projection ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 module-plugin pane-data projection split reduced `host_lifecycle.rs` to 1411 lines. `module_plugin_projection.rs` is 367 lines and now owns `RetainedEditorHost::module_plugins_pane_data(...)` in addition to the helper family moved earlier. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle module-plugin pane-data ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 module-plugin projection row split reduced `module_plugin_projection.rs` from 349 lines to 114 lines. `module_plugin_projection/rows.rs` is 242 lines and owns row/action label helpers, feature summaries, target/packaging labels, fallback manifest construction, and moved projection helper regressions. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection rows ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 build-export projection split reduced `host_lifecycle.rs` to 1218 lines. `build_export_projection.rs` is 197 lines and owns `RetainedEditorHost::build_export_pane_data(...)` plus output diagnostic prefixing. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle build-export projection ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 native-window presenter split reduced `host_lifecycle.rs` to 1124 lines. `host_lifecycle/native_window_presenters.rs` is 117 lines and owns `RetainedEditorHost::sync_native_window_presenters(...)`, native floating-window target cleanup/sync, callback wiring for newly-created native windows, per-window pane payload projection, presentation application, toolbar surface-frame attachment, and native floating-window metadata application. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle native-window presenter ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 effects/invalidation bridge split reduced `host_lifecycle.rs` to 952 lines. `host_lifecycle/dispatch_effects.rs` is 122 lines and owns status mutation plus `UiHostEventEffects` application; `host_lifecycle/invalidation_bridge.rs` is 78 lines and owns dirty-mask perf counters, pending UI perf scenario capture, invalidation-root mutation, paint-only invalidation diagnostics, and mark helpers. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle effects/invalidation ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 startup split reduced `host_lifecycle.rs` to 587 lines. `host_lifecycle/startup.rs` is 343 lines and owns `RetainedEditorHost::new(...)`, test construction, startup session resolution, startup state fallback, manager/resource/event receiver setup, template bridge creation, native live-host/play-mode backend setup, initial bridge/surface state, and bootstrap asset sync. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 render-submission split reduced `host_lifecycle.rs` to 536 lines. `host_lifecycle/render_submission.rs` is 59 lines and owns render-dirty extract submission, render-path diagnostics, backend-not-ready retry, and viewport-content frame update scheduling. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle render-submission ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 pane-payload split reduced `host_lifecycle.rs` from 536 lines to 442 lines. `host_lifecycle/pane_payloads.rs` is 128 lines and owns preset names, UI Asset pane presentations, Animation Editor pane presentations, runtime diagnostics snapshots, Module Plugins pane data, and Build/Export pane data collection for recompute. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle pane-payload ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 recompute viewport split reduced `host_lifecycle.rs` from 442 lines to 393 lines. `host_lifecycle/recompute_viewport.rs` is 64 lines and owns viewport content frame derivation, viewport resize dispatch, post-resize chrome/model rebuild, viewport pointer frame update, shell pointer bridge layout update, and shell-level pointer layout sync for recompute. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute viewport ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
