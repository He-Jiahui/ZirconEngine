---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/side_effects.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/status.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge/dirty_flags.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge/mark_helpers.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/callbacks.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/presentation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/sync.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/targets.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/editor_panes.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/workbench_panes.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/floating_projection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/decision.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/fast_path.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/slow_path.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/pointer_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/viewport_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/shell_metrics.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/constructors.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources/bundle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources/events.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources/managers.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/session.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/input.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/interaction.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/template_bridges.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/template_bridges/bundle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/template_bridges/factory.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/finalize.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/runtime_backend.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/session_state.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/shell_bootstrap.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/report.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/view_rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features/action.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features/summary.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/side_effects.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/status.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge/dirty_flags.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge/mark_helpers.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/callbacks.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/presentation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/sync.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/targets.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/editor_panes.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/workbench_panes.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/floating_projection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/decision.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/fast_path.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/slow_path.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/pointer_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/viewport_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/shell_metrics.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/constructors.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources/bundle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources/events.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources/managers.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/session.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/input.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/interaction.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/template_bridges.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/template_bridges/bundle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/template_bridges/factory.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/finalize.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/runtime_backend.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/session_state.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/shell_bootstrap.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/report.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/view_rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features/action.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features/summary.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app host-lifecycle module-plugin projection ownership scan
  - app host-lifecycle module-plugin pane-data ownership scan
  - app module-plugin projection pane-data subowner ownership scan
  - app module-plugin projection rows ownership scan
  - app module-plugin projection feature action/summary subowner ownership scan
  - app host-lifecycle viewport-toolbar projection ownership scan
  - app host-lifecycle build-export projection ownership scan
  - app host-lifecycle native-window presenter ownership scan
  - app host-lifecycle native-window presenter subowner ownership scan
  - app host-lifecycle native-window presenter sync/target subowner ownership scan
  - app host-lifecycle effects/invalidation ownership scan
  - app host-lifecycle invalidation bridge dirty-flags/mark-helper subowner ownership scan
  - app host-lifecycle dispatch-effects subowner ownership scan
  - app host-lifecycle render-submission ownership scan
  - app host-lifecycle tick/shell-metrics subowner ownership scan
  - app host-lifecycle pane-payload ownership scan
  - app host-lifecycle pane-payload subowner ownership scan
  - app host-lifecycle recompute viewport ownership scan
  - app host-lifecycle recompute ownership scan
  - app host-lifecycle recompute invalidation ownership scan
  - app host-lifecycle recompute invalidation decision/fast/slow subowner ownership scan
  - app host-lifecycle recompute floating-projection ownership scan
  - app host-lifecycle recompute shell-snapshot ownership scan
  - app host-lifecycle recompute shell snapshot/builder/template subowner ownership scan
  - app host-lifecycle recompute late-phase subowner ownership scan
  - app host-lifecycle startup ownership scan
  - app host-lifecycle startup session/template-bridge ownership scan
  - app host-lifecycle startup resources/state ownership scan
  - app host-lifecycle startup resources bundle/events/managers subowner ownership scan
  - app host-lifecycle startup constructors/with-viewport subowner ownership scan
  - app host-lifecycle startup template-bridge bundle/factory subowner ownership scan
  - app host-lifecycle startup state interaction subowner ownership scan
  - app host-lifecycle startup state construction input/assembly subowner ownership scan
  - app host-lifecycle startup with-viewport session/runtime/bootstrap/finalize subowner ownership scan
  - app retained-host owner visibility compile-boundary scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Lifecycle

`app/host_lifecycle.rs` is the structural retained host lifecycle entry. It declares the lifecycle child owners and keeps retained host tick scheduling, shell metrics, startup, recompute, invalidation, effect, pane-payload, native-window, and render-submission responsibilities from accumulating in one root file.

New feature-specific projection helpers should move into app child modules when they are pure data shaping and do not own lifecycle sequencing.

## Startup

`app/host_lifecycle/startup.rs` is the structural startup family entry. `startup/constructors.rs` owns `RetainedEditorHost::new(...)` and test construction entry points, while `startup/with_viewport.rs` owns the ordered retained-host construction flow once a viewport controller exists. `startup/with_viewport/session_state.rs` owns startup session/state resolution, `startup/with_viewport/shell_bootstrap.rs` owns host-window bootstrap shell sizing, `startup/with_viewport/runtime_backend.rs` owns native plugin live host/runtime/play-mode backend setup, and `startup/with_viewport/finalize.rs` owns initial asset workspace sync, bootstrap refresh draining, and refresh diagnostics publication.

`app/host_lifecycle/startup/resources.rs` is the structural startup resources entry. `startup/resources/bundle.rs` owns the `StartupManagers` bundle shape consumed by construction. `startup/resources/managers.rs` owns runtime asset manager, editor asset manager, resource manager, and editor manager resolution plus final bundle assembly. `startup/resources/events.rs` owns asset/editor-resource/resource change receiver subscriptions.

`app/host_lifecycle/startup/session.rs` owns startup request interpretation, startup session resolution, initial `EditorState` construction, and the test-only startup-state fallback. `app/host_lifecycle/startup/template_bridges.rs` is the structural startup template-bridge entry: `template_bridges/bundle.rs` owns the `StartupTemplateBridges` bundle, and `template_bridges/factory.rs` owns shared builtin template runtime loading plus startup bridge construction for the host window, Workbench window, floating-window source, viewport toolbar, Inspector surface, pane surface, and Component Showcase runtime. `app/host_lifecycle/startup/state.rs` is the structural retained host startup-state entry.

`app/host_lifecycle/startup/state/construction.rs` is the structural construction entry. `state/construction/input.rs` owns the `StartupHostConstruction` DTO that carries already-resolved startup dependencies into the assembly phase. `state/construction/assembly.rs` owns one-time `RetainedEditorHost` field assembly for managers, runtime, template bridges, build/export state, native-window state, dirty flags, invalidation root, and startup presentation defaults. `app/host_lifecycle/startup/state/interaction.rs` owns the initial pointer bridges, pointer state, scroll surfaces, and asset pointer state used by construction.

`host_lifecycle/tick.rs` keeps the lifecycle methods that run after construction: `tick`, `refresh_ui`, and committed pointer layout publication. `host_lifecycle/shell_metrics.rs` owns shell-size synchronization, chrome snapshot building, frame visibility checks used by recompute viewport, and refresh invalidation diagnostics publication. Startup request interpretation and test-only startup-state fallback live with construction so the runtime tick path does not carry startup policy.

## Module Plugin Projection

`app/module_plugin_projection.rs` is the structural module-plugin panel projection entry. `app/module_plugin_projection/pane_data.rs` owns only the projection method shape: it loads the pane status report through its report child, asks the row child to project `ModulePluginStatusViewData`, and returns `ModulePluginsPaneViewData`.

`app/module_plugin_projection/pane_data/report.rs` owns active project plugin status lookup. It resolves the active project root, loads `zircon-project.toml` when available, asks `EditorManager` for native-aware plugin status, falls back to the builtin plugin catalog when the manifest is unavailable, and aggregates diagnostics for the pane.

`app/module_plugin_projection/pane_data/view_rows.rs` owns row DTO construction from `EditorPluginStatusReport`. It converts plugin status records into `ModulePluginStatusViewData`, including target/packaging text, capability lists, diagnostics text, and stable action labels/ids.

`app/module_plugin_projection/rows.rs` owns the pure presentation helper exports for those rows: primary action labels/ids, optional feature helpers, target-mode labels, packaging labels, and the fallback project manifest.

`app/module_plugin_projection/rows/features.rs` is the structural feature-row helper entry. `features/action.rs` owns optional feature action label/id selection. `features/summary.rs` owns optional feature availability/dependency summaries.

`host_lifecycle/recompute.rs` calls `RetainedEditorHost::module_plugins_pane_data(...)` during pane-payload collection, but the method implementation lives in `module_plugin_projection/pane_data.rs`, report IO/fallback lives in `pane_data/report.rs`, row DTO mapping lives in `pane_data/view_rows.rs`, and reusable row text helpers live under `rows.rs`. This keeps lifecycle orchestration from accumulating panel-specific catalog fallback, row mapping, label formatting, action-id construction, and feature-summary logic.

## Build Export Projection

`app/build_export_projection.rs` owns the Build/Export panel projection method and output-path diagnostic formatting. It loads the active project manifest, merges the desktop export profiles exposed by `app/build_export_actions.rs`, asks `EditorManager` for native-aware export plans, applies queued/running/completed job state onto `BuildExportTargetViewData`, and publishes the wizard view model for the first target.

`host_lifecycle.rs` still decides when the Build/Export pane participates in recompute and when build/export actions are dispatched. The projection module owns panel data shaping so lifecycle code does not carry target row construction, output diagnostics, report overlay, job overlay, or manifest-load presentation errors.

## Viewport Toolbar Projection

`app/viewport_toolbar_projection.rs` owns viewport-toolbar surface-frame attachment for docked Scene/Game panes and floating Scene/Game windows. It clones the current host presentation, computes per-pane toolbar sizes, attaches runtime surface frames through `BuiltinViewportToolbarTemplateBridge`, maps projection control ids to stable editor action ids, rebuilds the floating-window list with updated toolbar frames, and publishes the updated host presentation.

`host_lifecycle.rs` still decides when toolbar frames must be attached during recompute and native-window synchronization. The helper module owns the projection details so lifecycle code does not carry per-control action-id mapping or floating-window surface-frame rewriting.

## Native Window Presenters

`app/host_lifecycle/native_window_presenters.rs` is the structural native floating-window presenter entry. `native_window_presenters/sync.rs` owns native floating-window presenter synchronization after the main workbench model, chrome, geometry, pane payloads, runtime diagnostics, and floating-window projection bundle have been computed. `native_window_presenters/targets.rs` owns native floating-window target collection and empty-target presenter cleanup.

`app/host_lifecycle/native_window_presenters/payloads.rs` owns per-native-window Module Plugins, Build/Export, and Component Showcase runtime payload preparation.

`app/host_lifecycle/native_window_presenters/callbacks.rs` owns native floating-window callback wiring for newly-created presenters, including retained-host callback registration and close-request forwarding.

`app/host_lifecycle/native_window_presenters/presentation.rs` owns applying retained host presentation into each native window, attaching viewport-toolbar surface frames, and configuring native floating-window metadata.

`host_lifecycle.rs` still decides when native-window presenter synchronization runs during recompute. The child module owns the presenter details so lifecycle code does not carry target collection, native-window callback setup, per-window payload projection, or presentation application.

## Effects And Invalidation Bridge

`app/host_lifecycle/dispatch_effects.rs` owns retained-host dispatch effect flow. It applies layout-preset mutations, dirty-domain invalidation, active-recompute viewport resize dirty-domain filtering, and dispatch-result error wrapping.

`app/host_lifecycle/dispatch_effects/status.rs` owns retained-host status line and status-task progress mutation. It compares against current runtime state before invalidating presentation data.

`app/host_lifecycle/dispatch_effects/side_effects.rs` owns `UiHostEventEffects` side effects after dirty-domain invalidation: forwarding workbench notifications, refreshing asset workspace/details/previews, importing models with completion/failure notifications, opening the command palette, and presenting the welcome surface.

`app/host_lifecycle/invalidation_bridge.rs` is the structural invalidation bridge entry. `invalidation_bridge/dirty_flags.rs` owns the retained-host bridge from dirty-domain masks to legacy dirty flags and UI perf counters: it captures the pending UI perf scenario, records dirty layout/presentation/render/paint-only counters, mutates `HostInvalidationRoot`, and publishes paint-only invalidation diagnostics. `invalidation_bridge/mark_helpers.rs` owns app-internal layout/presentation/render mark helpers.

## Render Submission

`app/host_lifecycle/render_submission.rs` owns the render-path submit step that runs from `tick` when `render_dirty` is set. It consumes pending render invalidation reasons, records render rebuild diagnostics and UI perf counters, submits the runtime extract/UI bundle to the viewport backend, preserves `render_dirty` when the lazy viewport backend is not ready, and requests a non-reentrant viewport-content frame update for retry.

`host_lifecycle/tick.rs` owns tick sequencing and decides when render submission runs. The render-submission child module owns render-path side effects so `tick` does not accumulate backend readiness, diagnostics, and retry scheduling details.

## Pane Payloads

`app/host_lifecycle/pane_payloads.rs` owns the recompute payload DTO and collection order that feeds retained host presentation and native floating-window presentation. It collects preset names directly, then delegates editor-pane payloads and workbench-pane payloads to child owners behind `RetainedEditorHost::collect_host_lifecycle_pane_payloads(...)`.

`app/host_lifecycle/pane_payloads/editor_panes.rs` owns UI Asset and Animation Editor pane presentation collection from the current runtime view instances and `EditorManager` pane snapshots.

`app/host_lifecycle/pane_payloads/workbench_panes.rs` owns visibility-gated runtime diagnostics, Module Plugins pane data, and Build/Export pane data collection. It keeps pane visibility checks and default-empty pane payloads out of the root payload DTO file.

`host_lifecycle/recompute.rs` owns recompute sequencing, presentation assembly, viewport toolbar attachment, and native-window synchronization. The pane-payload child module owns cross-pane payload gathering so recompute orchestration does not carry per-pane snapshot loops, active-document filtering, UI Asset/Animation editor projection, or module/build-export pane-data construction.

## Recompute

`app/host_lifecycle/recompute.rs` owns dirty recompute orchestration. It consumes the invalidation decision, asks child owners for shell, floating-window, viewport/pointer, presentation/surface, and native-window substeps, commits the floating-window projection bundle and shell geometry, and clears the dirty flags that were satisfied.

`app/host_lifecycle/recompute/invalidation.rs` is the structural invalidation subphase entry. `recompute/invalidation/decision.rs` consumes invalidation reasons, decides whether the update is a pure paint-only fast path, and returns the paint-only reason set needed by the ordered slow path. `recompute/invalidation/fast_path.rs` owns paint-only dirty-flag completion plus paint-only diagnostics/UI perf counters. `recompute/invalidation/slow_path.rs` owns slow-path rebuild accounting plus slow-path diagnostics/UI perf counters.

`app/host_lifecycle/recompute/floating_projection.rs` owns the floating-window projection subphase: it recomputes the floating-window source bridge, resolves the shared source frames, synchronizes native floating-window projection bounds, reads native host state, and builds the `FloatingWindowProjectionBundle` consumed by presentation, pointer layout, and native presenter synchronization.

`app/host_lifecycle/recompute/shell.rs` is the structural shell snapshot entry. `recompute/shell/snapshot.rs` owns the `RecomputeShellSnapshot` DTO used by the ordered recompute flow. `recompute/shell/builder.rs` owns runtime layout/descriptors reads, chrome/model construction, model-build counter recording, shell geometry computation, and shell snapshot assembly. `recompute/shell/template_bridges.rs` owns root/workbench template-bridge layout recompute and componentized Workbench layout-frame capture.

`app/host_lifecycle/recompute/presentation.rs` owns retained host presentation application for the main workbench surface, including Workbench window chrome sync, Component Showcase runtime selection, pane payload wiring, root/workbench host projection attachment, and floating-window projection bundle handoff.

`app/host_lifecycle/recompute/viewport_surfaces.rs` owns the post-presentation viewport surface submission step. It attaches viewport toolbar runtime surface frames to the main UI and forwards world-space UI surface submissions from the host scene to the viewport backend.

`app/host_lifecycle/recompute/pointer_surfaces.rs` owns the final pointer-surface publication step for menu, welcome recent projects, hierarchy, detail, and asset pointer bridges.

`host_lifecycle/tick.rs` decides when recompute runs from the retained-host frame tick and refresh calls; other app paths can still call recompute directly through the app-visible lifecycle API. The recompute family owns the long ordered recompute phase so the root lifecycle file remains structural rather than carrying invalidation diagnostics, presentation, and projection details.

## Recompute Viewport

`app/host_lifecycle/recompute_viewport.rs` owns the viewport and pointer-bridge substep inside dirty recompute. It derives the current viewport content frame from componentized workbench layout frames, dispatches viewport resize effects when the viewport size changes, rebuilds chrome/model snapshots after resize effects, updates the viewport pointer frame, updates the shell pointer bridge with workbench layout frames plus floating-window projection, and syncs activity rail, host page, document tab, and drawer header pointer layouts.

`host_lifecycle.rs` still owns the recompute order and passes the mutable model/chrome pair through this substep before pane payload collection and presentation publication.

## Boundary Rules

- Keep `app/host_lifecycle.rs` as the structural retained host lifecycle entry.
- Keep `tick`, `refresh_ui`, and committed pointer layout publication in `app/host_lifecycle/tick.rs`.
- Keep shell-size synchronization, chrome snapshot building, frame visibility checks, and refresh diagnostics publication in `app/host_lifecycle/shell_metrics.rs`.
- Keep `app/host_lifecycle/startup.rs` as the structural startup family entry.
- Keep retained-host public/test construction entry points and startup viewport creation in `app/host_lifecycle/startup/constructors.rs`.
- Keep retained-host construction order in `app/host_lifecycle/startup/with_viewport.rs`.
- Keep startup session/state resolution in `app/host_lifecycle/startup/with_viewport/session_state.rs`.
- Keep host-window bootstrap shell sizing in `app/host_lifecycle/startup/with_viewport/shell_bootstrap.rs`.
- Keep native live host, runtime, and play-mode backend setup in `app/host_lifecycle/startup/with_viewport/runtime_backend.rs`.
- Keep startup asset sync, bootstrap refresh draining, and refresh diagnostics publication in `app/host_lifecycle/startup/with_viewport/finalize.rs`.
- Keep startup resources module declarations and startup-family exports in `app/host_lifecycle/startup/resources.rs`.
- Keep the `StartupManagers` bundle shape in `app/host_lifecycle/startup/resources/bundle.rs`.
- Keep startup manager/resource resolution and final bundle assembly in `app/host_lifecycle/startup/resources/managers.rs`.
- Keep asset/editor/resource change subscriptions in `app/host_lifecycle/startup/resources/events.rs`.
- Keep startup session resolution and startup-state test fallback in `app/host_lifecycle/startup/session.rs`.
- Keep `app/host_lifecycle/startup/state.rs` as the structural startup-state entry.
- Keep startup construction module declarations and exports in `app/host_lifecycle/startup/state/construction.rs`.
- Keep the startup construction input DTO in `app/host_lifecycle/startup/state/construction/input.rs`.
- Keep one-time retained-host field assembly, dirty flags, invalidation root construction, and non-interaction startup defaults in `app/host_lifecycle/startup/state/construction/assembly.rs`.
- Keep initial pointer bridges, pointer state, scroll surfaces, and asset pointer state initialization in `app/host_lifecycle/startup/state/interaction.rs`.
- Keep startup template-bridge module declarations and public family exports in `app/host_lifecycle/startup/template_bridges.rs`.
- Keep the `StartupTemplateBridges` bundle shape and startup-family visibility boundary in `app/host_lifecycle/startup/template_bridges/bundle.rs`.
- Keep startup builtin template runtime loading, startup template bridge construction, and Component Showcase runtime creation in `app/host_lifecycle/startup/template_bridges/factory.rs`.
- Keep recompute invalidation module declarations in `app/host_lifecycle/recompute/invalidation.rs`.
- Keep recompute invalidation reason consumption and paint-only decision output in `app/host_lifecycle/recompute/invalidation/decision.rs`.
- Keep paint-only fast-path dirty-flag completion and diagnostics in `app/host_lifecycle/recompute/invalidation/fast_path.rs`.
- Keep slow-path rebuild accounting and diagnostics in `app/host_lifecycle/recompute/invalidation/slow_path.rs`.
- Keep dirty recompute orchestration in `app/host_lifecycle/recompute.rs`.
- Keep floating-window source bridge recompute, native projection bounds sync, native host snapshot read, and floating projection bundle construction in `app/host_lifecycle/recompute/floating_projection.rs`.
- Keep recompute shell module declarations in `app/host_lifecycle/recompute/shell.rs`.
- Keep the recompute shell snapshot DTO in `app/host_lifecycle/recompute/shell/snapshot.rs`.
- Keep runtime layout/descriptors reads, chrome/model construction, and shell geometry computation in `app/host_lifecycle/recompute/shell/builder.rs`.
- Keep template/workbench bridge recompute and componentized layout-frame capture in `app/host_lifecycle/recompute/shell/template_bridges.rs`.
- Keep main retained host presentation application in `app/host_lifecycle/recompute/presentation.rs`.
- Keep post-presentation viewport toolbar/world-space surface submission in `app/host_lifecycle/recompute/viewport_surfaces.rs`.
- Keep final menu/welcome/hierarchy/detail/asset pointer-surface publication in `app/host_lifecycle/recompute/pointer_surfaces.rs`.
- Keep render submission and recompute call sites in `app/host_lifecycle.rs`.
- Keep dispatch effect dirty-domain flow, active-recompute viewport resize filtering, and dispatch-result error wrapping in `app/host_lifecycle/dispatch_effects.rs`.
- Keep status line/status task mutation in `app/host_lifecycle/dispatch_effects/status.rs`.
- Keep post-invalidation `UiHostEventEffects` side effects in `app/host_lifecycle/dispatch_effects/side_effects.rs`.
- Keep invalidation bridge module declarations in `app/host_lifecycle/invalidation_bridge.rs`.
- Keep retained-host dirty-domain to invalidation-root/legacy dirty-flag bridging in `app/host_lifecycle/invalidation_bridge/dirty_flags.rs`.
- Keep app-internal layout/presentation/render mark helpers in `app/host_lifecycle/invalidation_bridge/mark_helpers.rs`.
- Keep render-dirty extract submission, render-path diagnostics, backend-not-ready retry, and viewport-content frame update scheduling in `app/host_lifecycle/render_submission.rs`.
- Keep recompute pane payload DTOs and collection order in `app/host_lifecycle/pane_payloads.rs`.
- Keep UI Asset and Animation Editor pane payload gathering in `app/host_lifecycle/pane_payloads/editor_panes.rs`.
- Keep runtime diagnostics, Module Plugins, and Build/Export visibility-gated payload gathering in `app/host_lifecycle/pane_payloads/workbench_panes.rs`.
- Keep recompute viewport resize handling, post-resize chrome/model rebuild, viewport pointer frame updates, shell pointer bridge layout updates, and shell pointer layout sync calls in `app/host_lifecycle/recompute_viewport.rs`.
- Keep native floating-window presenter module declarations in `app/host_lifecycle/native_window_presenters.rs`.
- Keep native floating-window presenter `NativeWindowPresenterStore::sync_targets(...)` orchestration in `app/host_lifecycle/native_window_presenters/sync.rs`.
- Keep native floating-window target collection and empty-target presenter cleanup in `app/host_lifecycle/native_window_presenters/targets.rs`.
- Keep native-window pane payload preparation in `app/host_lifecycle/native_window_presenters/payloads.rs`.
- Keep native-window callback wiring and close-request forwarding in `app/host_lifecycle/native_window_presenters/callbacks.rs`.
- Keep native-window retained presentation application, viewport-toolbar frame attachment, and native metadata configuration in `app/host_lifecycle/native_window_presenters/presentation.rs`.
- Keep Build/Export action execution, async job queue state, and output folder mutation in `app/build_export_actions.rs`.
- Keep Build/Export pane-data projection and output diagnostic formatting in `app/build_export_projection.rs`; do not add target-row construction or report/job overlay logic back to `host_lifecycle.rs`.
- Keep module plugin action execution and callback handling in `app/module_plugin_actions.rs`.
- Keep `app/module_plugin_projection.rs` as the structural module-plugin projection entry.
- Keep module plugin pane-data orchestration in `app/module_plugin_projection/pane_data.rs`; do not add catalog fallback or row mapping back to lifecycle code.
- Keep module plugin status-report lookup, manifest fallback, and diagnostics aggregation in `app/module_plugin_projection/pane_data/report.rs`.
- Keep `ModulePluginStatusViewData` construction in `app/module_plugin_projection/pane_data/view_rows.rs`.
- Keep module plugin row label/action-id, target/packaging text, fallback-manifest helper, feature-helper exports, and helper tests in `app/module_plugin_projection/rows.rs`.
- Keep optional feature action label/id selection in `app/module_plugin_projection/rows/features/action.rs`.
- Keep optional feature availability/dependency summaries in `app/module_plugin_projection/rows/features/summary.rs`.
- Keep viewport toolbar surface-frame attachment and projection-control action-id mapping in `app/viewport_toolbar_projection.rs`.
- Keep invalidation mask ownership and dirty-domain policy in `app/invalidation.rs` plus the lifecycle bridge methods documented in `app/invalidation.md`.
- Move future pure pane projection helper families into focused child modules before `host_lifecycle.rs` grows another panel-specific formatting section.

## Validation Notes

The 2026-06-18 module-plugin projection split reduced `host_lifecycle.rs` from 1987 lines to 1739 lines. `module_plugin_projection.rs` is 257 lines and owns module plugin panel label/action/summary/fallback helpers plus the moved module-plugin projection tests. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle module-plugin projection ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 viewport-toolbar projection split reduced `host_lifecycle.rs` to 1523 lines. `viewport_toolbar_projection.rs` is 220 lines and owns toolbar surface-frame attachment for docked and floating Scene/Game panes plus projection-control action-id mapping. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle viewport-toolbar projection ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 module-plugin pane-data projection split reduced `host_lifecycle.rs` to 1411 lines. `module_plugin_projection.rs` is 367 lines and now owns `RetainedEditorHost::module_plugins_pane_data(...)` in addition to the helper family moved earlier. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle module-plugin pane-data ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 module-plugin projection row split reduced `module_plugin_projection.rs` from 349 lines to 114 lines. `module_plugin_projection/rows.rs` is 242 lines and owns row/action label helpers, feature summaries, target/packaging labels, fallback manifest construction, and moved projection helper regressions. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection rows ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 module-plugin feature action/summary subowner split reduced `module_plugin_projection/rows/features.rs` from 95 lines to a 5-line structural entry. `features/action.rs` is 50 lines and owns optional feature action label/id selection. `features/summary.rs` is 54 lines and owns optional feature state and dependency summaries.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection feature action/summary subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 module-plugin projection pane-data split reduced `module_plugin_projection.rs` from 119 lines to 2 lines. `module_plugin_projection/pane_data.rs` is 116 lines and owns active project manifest lookup, native plugin status report selection, builtin fallback diagnostics, Plugin Manager pane view-model construction, and row helper consumption. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection pane-data ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh full `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was not claimed because concurrent `zircon_runtime` Cargo jobs were active. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 module-plugin pane-data report/view-row subowner split reduced `module_plugin_projection/pane_data.rs` from 113 lines to a 23-line projection method owner. `pane_data/report.rs` is 45 lines and owns active project status lookup plus fallback diagnostics. `pane_data/view_rows.rs` is 66 lines and owns `ModulePluginStatusViewData` construction from `EditorPluginStatusReport`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection pane-data subowner ownership scan, a retained-host owner visibility compile-boundary scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 build-export projection split reduced `host_lifecycle.rs` to 1218 lines. `build_export_projection.rs` is 197 lines and owns `RetainedEditorHost::build_export_pane_data(...)` plus output diagnostic prefixing. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle build-export projection ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 native-window presenter split reduced `host_lifecycle.rs` to 1124 lines. `host_lifecycle/native_window_presenters.rs` is 117 lines and owns `RetainedEditorHost::sync_native_window_presenters(...)`, native floating-window target cleanup/sync, callback wiring for newly-created native windows, per-window pane payload projection, presentation application, toolbar surface-frame attachment, and native floating-window metadata application. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle native-window presenter ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 effects/invalidation bridge split reduced `host_lifecycle.rs` to 952 lines. `host_lifecycle/dispatch_effects.rs` is 122 lines and owns status mutation plus `UiHostEventEffects` application; `host_lifecycle/invalidation_bridge.rs` is 78 lines and owns dirty-mask perf counters, pending UI perf scenario capture, invalidation-root mutation, paint-only invalidation diagnostics, and mark helpers. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle effects/invalidation ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 invalidation bridge dirty-flags/mark-helper subowner split reduced `host_lifecycle/invalidation_bridge.rs` from 78 lines to a 2-line structural entry. `invalidation_bridge/dirty_flags.rs` is 64 lines and owns dirty-mask UI perf counter recording, pending UI perf scenario capture, invalidation-root mutation, legacy dirty-flag mutation, and paint-only diagnostics publication. `invalidation_bridge/mark_helpers.rs` is 17 lines and owns the app-internal layout/presentation/render mark helpers.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle invalidation bridge dirty-flags/mark-helper subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 startup split reduced `host_lifecycle.rs` to 587 lines. `host_lifecycle/startup.rs` is 343 lines and owns `RetainedEditorHost::new(...)`, test construction, startup session resolution, startup state fallback, manager/resource/event receiver setup, template bridge creation, native live-host/play-mode backend setup, initial bridge/surface state, and bootstrap asset sync. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup session/template-bridge split reduced `host_lifecycle/startup.rs` from 343 lines to 230 lines. `host_lifecycle/startup/session.rs` is 54 lines and owns startup request interpretation plus startup-state fallback; `host_lifecycle/startup/template_bridges.rs` is 85 lines and owns builtin template runtime loading plus all startup bridge/runtime construction. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup session/template-bridge ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup template-bridge bundle/factory subowner split reduced `host_lifecycle/startup/template_bridges.rs` from 85 lines to a 4-line structural entry. `template_bridges/bundle.rs` is 20 lines and owns the `StartupTemplateBridges` bundle shape plus the startup-family visibility boundary needed by sibling construction modules. `template_bridges/factory.rs` is 74 lines and owns builtin template runtime loading plus all startup bridge/runtime construction.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup template-bridge bundle/factory subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). The first compile-boundary pass exposed E0365/E0364/E0603 because `StartupTemplateBridges` and `create_startup_template_bridges(...)` were only `pub(super)` inside their child modules while sibling startup modules imported them through the structural parent. The fix scopes the bundle, its fields, and the factory function to `pub(in crate::ui::retained_host::app::host_lifecycle::startup)`, keeping visibility inside the startup family without widening to the app layer. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup resources/state split reduced `host_lifecycle/startup.rs` from 230 lines to 111 lines. `host_lifecycle/startup/resources.rs` is 64 lines and owns manager/resource resolution plus event subscriptions; `host_lifecycle/startup/state.rs` is 132 lines and owns one-time `RetainedEditorHost` field assembly. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup resources/state ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup resources bundle/events/managers subowner split reduced `host_lifecycle/startup/resources.rs` from 68 lines to a 6-line structural entry. `startup/resources/bundle.rs` is 20 lines and owns the `StartupManagers` bundle shape, `startup/resources/events.rs` is 38 lines and owns asset/editor/resource change receiver subscriptions, and `startup/resources/managers.rs` is 45 lines and owns manager/resource resolution plus final startup bundle assembly.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup resources bundle/events/managers subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup constructors/with-viewport subowner split reduced `host_lifecycle/startup.rs` from 111 lines to a 7-line structural startup family entry. `startup/constructors.rs` is 30 lines and owns `RetainedEditorHost::new(...)`, test construction, and startup viewport controller creation. `startup/with_viewport.rs` is 81 lines and owns retained-host construction once a viewport exists: manager/session/state resolution, bootstrap size, template bridges, native plugin live host, play-mode backend, host construction, startup asset sync, bootstrap refresh drain, and refresh diagnostics.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and an app host-lifecycle startup constructors/with-viewport subowner ownership scan. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` remains blocked before editor code by active `zircon_runtime::scene::dynamic_scene::session` owner-split work: `session/io/mod.rs` re-exports private IO helpers, producing E0364/E0603 visibility errors. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup with-viewport session/runtime/bootstrap/finalize subowner split reduced `host_lifecycle/startup/with_viewport.rs` from 84 lines to a 60-line construction-order owner. `with_viewport/session_state.rs` is 30 lines and owns startup session/state resolution, `with_viewport/shell_bootstrap.rs` is 12 lines and owns shell-size bootstrap, `with_viewport/runtime_backend.rs` is 33 lines and owns native live host/runtime/play-mode backend setup, and `with_viewport/finalize.rs` is 7 lines and owns startup asset sync/bootstrap refresh diagnostics.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup with-viewport session/runtime/bootstrap/finalize subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup state interaction subowner split reduced `host_lifecycle/startup/state.rs` from 136 lines to a 6-line structural entry. `state/construction.rs` is 132 lines and owns one-time `RetainedEditorHost` field assembly. `state/interaction.rs` is 64 lines and owns initial pointer bridges, pointer state, scroll surfaces, and asset pointer state initialization.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup state interaction subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 startup state construction input/assembly subowner split reduced `host_lifecycle/startup/state/construction.rs` from 128 lines to a 4-line structural entry. `state/construction/input.rs` is 23 lines and owns the `StartupHostConstruction` DTO. `state/construction/assembly.rs` is 108 lines and owns one-time `RetainedEditorHost` field assembly.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle startup state construction input/assembly subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 render-submission split reduced `host_lifecycle.rs` to 536 lines. `host_lifecycle/render_submission.rs` is 59 lines and owns render-dirty extract submission, render-path diagnostics, backend-not-ready retry, and viewport-content frame update scheduling. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle render-submission ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 pane-payload split reduced `host_lifecycle.rs` from 536 lines to 442 lines. `host_lifecycle/pane_payloads.rs` is 128 lines and owns preset names, UI Asset pane presentations, Animation Editor pane presentations, runtime diagnostics snapshots, Module Plugins pane data, and Build/Export pane data collection for recompute. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle pane-payload ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 recompute viewport split reduced `host_lifecycle.rs` from 442 lines to 393 lines. `host_lifecycle/recompute_viewport.rs` is 64 lines and owns viewport content frame derivation, viewport resize dispatch, post-resize chrome/model rebuild, viewport pointer frame update, shell pointer bridge layout update, and shell-level pointer layout sync for recompute. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute viewport ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 recompute orchestration split reduced `host_lifecycle.rs` from 375 lines to 96 lines. `host_lifecycle/recompute.rs` is 303 lines and owns the dirty recompute phase from invalidation-reason consumption through presentation application, viewport surface submission, native-window presenter synchronization, pointer-surface sync, and dirty-flag clearing. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh full `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was not claimed because a concurrent `zircon_runtime` Cargo test job was still active. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 host-lifecycle tick/shell-metrics subowner split reduced `host_lifecycle.rs` from 96 lines to a 10-line structural entry. `host_lifecycle/tick.rs` is 54 lines and owns tick scheduling, refresh, and committed pointer layout publication. `host_lifecycle/shell_metrics.rs` is 39 lines and owns shell-size synchronization, chrome snapshot building, frame visibility checks, and refresh invalidation diagnostics publication.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle tick/shell-metrics subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 recompute late-phase subowner split reduced `host_lifecycle/recompute.rs` from 237 lines to 186 lines. `recompute/presentation.rs` is 47 lines and owns main retained-host presentation application, `recompute/viewport_surfaces.rs` is 25 lines and owns viewport toolbar/world-space surface submission, and `recompute/pointer_surfaces.rs` is 18 lines and owns final pointer-surface publication.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute late-phase subowner ownership scan, and scoped `git diff --check`. Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 recompute invalidation decision/fast/slow subowner split reduced `host_lifecycle/recompute/invalidation.rs` from 84 lines to a 3-line structural entry. `recompute/invalidation/decision.rs` is 40 lines and owns invalidation reason consumption plus paint-only/slow-path decision output. `recompute/invalidation/fast_path.rs` is 31 lines and owns paint-only dirty-flag completion plus diagnostics. `recompute/invalidation/slow_path.rs` is 31 lines and owns slow-path rebuild accounting plus diagnostics.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute invalidation decision/fast/slow subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 recompute floating-projection subowner split reduced `host_lifecycle/recompute.rs` from 186 lines to 132 lines. `recompute/floating_projection.rs` is 65 lines and owns floating-window source bridge recompute, shared-source resolution, native projection bounds sync, native host lookup, and `FloatingWindowProjectionBundle` construction.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute floating-projection ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 recompute shell-snapshot subowner split reduced `host_lifecycle/recompute.rs` from 137 lines to 75 lines. `recompute/shell.rs` is 87 lines and owns runtime layout/descriptors reads, chrome/model construction, model-build counter recording, shell geometry computation, root/workbench template bridge recompute, and componentized workbench layout-frame capture.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute shell-snapshot ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 recompute shell snapshot/builder/template subowner split reduced `host_lifecycle/recompute/shell.rs` from 83 lines to a 3-line structural entry. `recompute/shell/snapshot.rs` is 12 lines and owns the recompute shell DTO. `recompute/shell/builder.rs` is 56 lines and owns shell snapshot construction. `recompute/shell/template_bridges.rs` is 34 lines and owns root/workbench template-bridge recompute plus layout-frame capture.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle recompute shell snapshot/builder/template subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 pane-payload subowner split reduced `host_lifecycle/pane_payloads.rs` from 122 lines to a 71-line payload DTO and collection-order owner. `pane_payloads/editor_panes.rs` is 39 lines and owns UI Asset/Animation Editor pane presentation gathering. `pane_payloads/workbench_panes.rs` is 49 lines and owns visibility-gated runtime diagnostics, Module Plugins, and Build/Export payload gathering.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle pane-payload subowner ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 dispatch-effects subowner split reduced `host_lifecycle/dispatch_effects.rs` from 116 lines to a 56-line dispatch-flow owner. `dispatch_effects/status.rs` is 25 lines and owns status line/task progress mutation. `dispatch_effects/side_effects.rs` is 44 lines and owns notification, asset refresh, import, command-palette, and welcome-surface side effects.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle dispatch-effects subowner ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 native-window presenter subowner split reduced `host_lifecycle/native_window_presenters.rs` from 113 lines to a 76-line presenter sync owner. `native_window_presenters/payloads.rs` is 42 lines and owns per-native-window Module Plugins, Build/Export, and Component Showcase runtime payload preparation. `native_window_presenters/callbacks.rs` is 27 lines and owns callback wiring plus close-request forwarding. `native_window_presenters/presentation.rs` is 55 lines and owns per-window retained presentation application, viewport-toolbar frame attachment, and native floating-window metadata configuration. `app/native_windows.rs` now re-exports `NativeFloatingWindowTarget` at the app boundary so these child owners can accept typed native targets.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle native-window presenter subowner ownership scan, and scoped `git diff --check` (only existing CRLF conversion warnings appeared). Focused `cargo check` remains deferred until a clear compile lane is available; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 native-window presenter sync/target subowner split reduced `host_lifecycle/native_window_presenters.rs` from 82 lines to a 5-line structural entry. `native_window_presenters/sync.rs` is 70 lines and owns `NativeWindowPresenterStore::sync_targets(...)` orchestration, callback/presentation closure wiring, pane-template runtime selection, and native-window sync status errors. `native_window_presenters/targets.rs` is 32 lines and owns target collection plus empty-target cleanup.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle native-window presenter sync/target subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
