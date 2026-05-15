---
related_code:
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/tracy.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/retained_host/app/component_showcase_runtime.rs
  - zircon_editor/src/ui/retained_host/app/pane_payload_visibility.rs
  - zircon_editor/src/ui/retained_host/app/presentation_cache.rs
  - zircon_editor/src/ui/retained_host/app/profiling.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/host/editor_runtime_client.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/performance_timeline.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/performance_timeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/performance_timeline_view_descriptor.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - tools/ui-profile-capture.ps1
implementation_files:
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/tracy.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/retained_host/app/component_showcase_runtime.rs
  - zircon_editor/src/ui/retained_host/app/pane_payload_visibility.rs
  - zircon_editor/src/ui/retained_host/app/presentation_cache.rs
  - zircon_editor/src/ui/retained_host/app/profiling.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/host/editor_runtime_client.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/performance_timeline.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/performance_timeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/performance_timeline_view_descriptor.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - tools/ui-profile-capture.ps1
plan_sources:
  - .codex/plans/Zircon 性能时间轴与 Tracy 集成设计.md
  - user: 2026-05-13 continue profiling timeline and Tracy integration milestone
  - user: 2026-05-14 UI 卡顿热点 Profiling 检测实现计划
tests:
  - zircon_editor/src/ui/retained_host/app/profiling.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - target: cargo check -p zircon_editor --profile profiling --features profiling --locked
  - target: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-tracy profiling-chrome" --locked
  - target: cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --profile profiling --features profiling --locked
  - target: cargo check -p zircon_runtime --profile profiling --features "profiling profiling-tracy" --locked
  - target: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - target: cargo test -p zircon_runtime --lib profiling --profile profiling --features "profiling profiling-chrome" --locked
  - target: cargo test -p zircon_editor --lib frame_rows_project_budget_bar_nodes --profile profiling --features profiling --locked --message-format=short
  - target: cargo test -p zircon_editor --lib template_text_updates_do_not_duplicate_static_nodes --profile profiling --features profiling --locked --message-format=short
  - target: cargo test -p zircon_runtime --lib core::diagnostics::profiling --offline --message-format=short
  - target: cargo test -p zircon_runtime --lib ui_hotspots_collect_gpu_presenter_counters --locked --message-format=short
  - target: cargo test -p zircon_editor --lib preview_loader --locked -- --nocapture
  - target: cargo test -p zircon_editor --lib tests::host::retained_callback_dispatch::template_bridge::workbench_projection::builtin_host_window_template_bridge_recomputes_surface_backed_frames_with_shell_size --locked -- --exact --nocapture
  - target: cargo test -p zircon_editor --lib tests::host::retained_callback_dispatch::asset::template_bridge::builtin_asset_surface_open_browser_dispatches_static_binding_from_template --locked -- --exact --nocapture
  - target: cargo test -p zircon_editor --lib startup_session --locked -- --nocapture
  - target: cargo test -p zircon_editor --lib create_project_and_open_persists_recent_project_and_returns_project_session --locked -- --nocapture
  - target: cargo test -p zircon_editor --lib workbench_main_interface_entries_are_template_backed_and_reflected --locked -- --nocapture
  - target: cargo test -p zircon_editor --lib visual_assets --locked -- --nocapture
  - target: cargo test -p zircon_editor --lib builtin_asset_surface_minimal_bridge_dispatches_without_startup_runtime --locked -- --nocapture
  - target: cargo test -p zircon_editor --lib builtin_welcome_surface_minimal_bridge_dispatches_without_startup_runtime --locked -- --nocapture
  - target: cargo check -p zircon_app --features "target-editor-host" --locked --message-format=short
  - target: cargo build -p zircon_app --bin zircon_editor --profile profiling --features "target-editor-host profiling profiling-chrome" --locked --message-format=short
  - target: PowerShell parser check for tools/ui-profile-capture.ps1
  - target: tools/ui-profile-capture.ps1 -SkipBuild -Scenario startup -AutoCloseSeconds 8
  - target: tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260514-223023-startup)
  - target: tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260514-225912-startup)
  - target: tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260514-232056-startup)
  - target: CARGO_TARGET_DIR=D:\cargo-targets\zircon-shared\ui-poset-batching tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 1 -SkipBuild (20260515-202131-startup and 20260515-202403-startup shared-target smoke)
  - target: CARGO_TARGET_DIR=D:\cargo-targets\zircon-shared\ui-poset-batching tools/ui-profile-capture.ps1 -Scenario click -AutoCloseSeconds 3 -AutoInteract -RequireScenarioEvidence -SkipBuild (20260515-203340-click auto-interaction GPU batch evidence)
doc_type: module-detail
---

# Editor Performance Timeline

## Purpose

The editor side of the profiling milestone consumes the runtime diagnostics spine without becoming the recorder owner. The retained host adds editor-stream frame and span samples for authoring work, and it can merge a dynamic runtime session snapshot through `EditorRuntimeClient::profile_control` when the runtime cdylib supports the optional ABI hook.

This is the M1/M2 bridge plus the first M3 panel surface. Runtime Diagnostics still reports whether profiling is active and how many frames, spans, counters, and over-budget frames are visible, while the Performance Timeline view projects the same snapshot into frame, span, hotspot, and capture-control rows.

## Editor Instrumentation

The retained host uses `zircon_runtime` profiling macros at stable CPU boundaries:

- `RetainedEditorHost::tick` records an `editor` frame called `retained_host_tick`.
- `recompute_if_dirty` records the shell presentation/layout recompute span.
- The recompute span is subdivided into read/build/layout/apply/sync phases so startup captures can distinguish model construction, template bridge layout, floating-window projection, pane payload collection, native-window presenter sync, viewport surface submission, and final pointer-layout synchronization. The `apply_presentation` phase is further split into shell-presentation construction, pane global updates, host-scene building, native-floating data building, contract conversion, and final `set_host_presentation` submission. Host-scene building also marks each dock pane, each chrome band, and menu subwork, so expensive startup panes can be separated from chrome assembly and hidden popup work.
- Viewport submission records `submit_viewport_extract` and `submit_extract_with_ui` spans.
- Viewport image polling records a span only when a new image is present.
- `SoftbufferHostPresenter::present` records presenter spans for planning/repaint, RGBA copy, and the softbuffer present call.

These spans stay in editor-owned modules and use the runtime profiling recorder as a shared diagnostic service. They do not move editor selection, workbench layout, native-window state, or viewport authoring state into `zircon_runtime`.

## Dynamic Runtime Merge

`EditorRuntimeClient` exposes a default `profile_control` method returning `Ok(None)`. `zircon_app::RuntimeSession` implements it by serializing `ProfileControlRequest`, calling the optional dynamic runtime `profile_control` function, decoding `ProfileControlResponse`, and freeing the returned ABI buffer.

When the `profiling` feature is enabled, `RetainedEditorHost::runtime_diagnostics_with_profile` starts from local editor diagnostics and asks the runtime client for a `Snapshot` response. If a runtime snapshot is returned, `app/profiling.rs` merges it into the editor snapshot:

- Runtime span ids and parent ids are offset when editor spans already exist.
- Frames, spans, and counters are appended so both `editor` and `runtime` streams are available to the diagnostics payload.
- Active and feature-enabled flags are OR-ed so either side can indicate live capture.
- Session ids are combined only when the editor and runtime report different ids.

The merge helper is feature-gated and lives in its own focused module to keep `host_lifecycle.rs` as lifecycle orchestration rather than a profiling transport implementation.

## Tracy Process Startup

When the editor or runtime preview is built with `profiling-tracy`, `zircon_app::EntryRunner` installs the runtime profiling Tracy sink before creating the editor host or runtime session. The dynamic runtime API entry installs the same sink for the cdylib image when the runtime library is loaded. This lets the editor host, app presenter, and dynamic runtime emit to Tracy through the same instrumentation macros while still treating Tracy as an external live viewer.

## UI Exposure

The Runtime Diagnostics pane now includes profiling detail rows when `ProfileSnapshot.feature_enabled` is true:

- `Profiling: active|inactive (<frames> frames, <spans> spans, <counters> counters)`.
- `Profiling over-budget frames: <count>`.

The Performance Timeline view uses `PanePayloadKind::PerformanceTimelineV1` and the built-in `performance_timeline_body.v2.ui.toml` template. The payload builder reads the merged `ProfileSnapshot`, derives recent frame rows, recent span rows, and top hotspot rows, then the retained host converts those rows into native template nodes so the panel works before a richer graph renderer exists.

Frame rows now carry budget visualization metadata in addition to text labels. The builder computes the frame duration as a ratio of the configured budget, a normalized bar-fill ratio, and a budget-marker ratio using the larger of duration and budget as the row scale. The retained-host conversion mutates the projected summary/session/output template nodes in place, then appends only dynamic frame/span/hotspot/control nodes. Each frame becomes four dynamic nodes: an inset track, a duration fill bar, a warning budget marker, and a compact overlay label such as `18.00 ms (108% budget / 16.67 ms budget)`. Over-budget frames use the `danger` fill and warning text tone, while in-budget frames use the accent fill. This keeps M3 visual feedback inside the existing pane payload and template-node path instead of introducing a separate graph renderer or touching the GPU command-stream migration.

Capture buttons are routed through `RetainedEditorHost::dispatch_performance_timeline_action`:

- `PerformanceTimeline.StartCapture` starts the editor recorder and asks the optional dynamic runtime profile-control hook to start its recorder.
- `PerformanceTimeline.StopCapture` stops both sides when available.
- `PerformanceTimeline.ExportReport` exports the local editor report and asks the dynamic runtime to export its report when supported.
- `PerformanceTimeline.Reset` clears captured samples locally and remotely when supported.

The status line reports both editor and runtime results. The panel then invalidates presentation data so the next diagnostics snapshot reflects the new active/export/reset state.

## UI Hotspot Export

The profiling exporter now writes `ui_hotspots.json` next to `timeline.zrtrace.json`, `timeline.perfetto.json`, `hotspots.json`, and `summary.md`. The UI report is built from retained-host counters named `ui.<scenario>.<metric>`, with scenarios covering startup, hover, click, drag, drawer resize, asset refresh, and viewport image refresh.

The retained host records counters at the boundaries that previously hid jank:

- dirty-domain requests from `invalidate_host` and paint-only invalidation.
- slow presentation/layout rebuilds and render-only rebuilds.
- chrome snapshot pulls and workbench model builds.
- chrome command stream full rebuilds and patches, emitted by the softbuffer presenter during profiling so the GPU migration path is measurable before it becomes the default.
- software fallback presenter submits while the GPU presenter is not yet active.
- GPU presenter upload bytes and draw calls when the command-stream backend is exercised.
- native pointer dispatch scenarios, including hover, click, drag, and drawer resize.
- full-frame versus region redraw requests.
- presenter full paint, region paint, and painted pixel count.

`ui_hotspots.json` flags the cases that should be fixed first: hover/click causing presentation rebuilds, hover pulling chrome snapshots or rebuilding workbench models, region scenarios requesting full-frame redraws, region redraw requests that still repaint the full frame, drawer resize entering the slow path, and viewport image updates dirtying layout or presentation. `summary.md` mirrors these alerts under `First Fix Candidates` before listing raw CPU spans, so an editor capture starts from retained-host slow-path evidence instead of guessing from generic CPU totals.

The startup capture loop exposed a retained-host chrome bottleneck before visual redesign work: hidden root menu popup template instancing consumed multiple seconds even though no popup was visible. Root popup rows now paint from retained menu item data when a menu is open, while startup stores only menu items and popup dimensions. Dock headers use the existing procedural hit-testable chrome path for the production first frame, and SVG preview/icon rasterization is cached process-wide so repeated tab/rail icons do not re-read and re-rasterize the same files. The report sequence under `target/zircon-profiles/20260514-164324-startup`, `20260514-165206-startup`, and `20260514-170437-startup` records this progression from menu popup instancing as the dominant hotspot to software presenter repaint as the next largest remaining cost.

The 2026-05-14 startup follow-up used the same capture pipeline to remove the next retained-host startup cliffs before any visual redesign. `target/zircon-profiles/20260514-200016-startup` confirmed that lazy-loading the component showcase runtime removed `new_load_builtin_templates` from the startup top spans, leaving `apply_convert_host_scene_data` as the dominant cost. `20260514-201128-startup` then showed icon-only preview metadata reducing `apply_build_host_scene_data` from roughly 142 ms to roughly 69 ms by avoiding SVG parse/raster work during scene construction. `20260514-201811-startup` added pane-conversion spans and exposed a first-use hierarchy conversion fallback that built the static full builtin runtime for about 203 ms. `20260514-203032-startup` routed hierarchy, inspector, console, and animation pane body projection through the shared startup runtime and removed that fallback from the hot list. The latest capture, `20260514-203843-startup`, keeps `retained_host:new` near 110 ms, `recompute_if_dirty` near 104 ms, `recompute_apply_presentation` near 86 ms, and `apply_build_host_scene_data` near 74 ms. Hidden build-export and module-plugin payload collection is now visibility-gated, so those panes no longer spend startup time when they are not active.

That latest report still shows a large `async_resolve_render_framework` aggregate, but it is on the asynchronous viewport/render-service path rather than the retained-host constructor path. The remaining startup work that is still directly visible to the UI path is now concentrated in host-scene assembly, startup-session resolution, first full paint, and software presenter repaint/copy/present. Idle hover in the same report remains below 1 ms p95 and records no slow-path presentation rebuild, which means the current jank focus has moved from pointer-time snapshot churn to startup/presenter work.

The next startup slice removed two more retained-host costs. `chrome_template_projection::surface_metrics_from_chrome_assets` no longer instantiates three v2 chrome assets just to read fixed shell heights; it returns constants matching the authored metric controls while leaving menu/page/status/rail node projection asset-backed. In `target/zircon-profiles/20260514-212044-startup`, `scene_surface_metrics` falls to 0-1 us and `apply_build_host_scene_data` drops to roughly 51 ms. The same slice moves native-floating-window target collection ahead of hidden pane payload construction, so the no-native-window startup case avoids build-export/module-plugin payload work and `recompute_native_window_presenters` falls to single-digit microseconds.

`target/zircon-profiles/20260514-212822-startup` then subdivided `new_resolve_startup_session`: the auto-open path spent roughly 19.7 ms validating all recent projects, 8 ms validating the last project again, and 18.7 ms opening the project. Project-mode startup does not show the recent-project list, so `resolve_session.rs` now validates that list only when startup actually falls back to Welcome. The follow-up capture `20260514-214204-startup` removes `validate_recent_projects` from the project-mode hot path, reduces `new_resolve_startup_session` to roughly 29.2 ms, and brings `retained_host:new` down to roughly 76.2 ms. Remaining direct UI costs are now startup runtime document loading, host-scene assembly, software first paint, and the unavoidable last-project validation/open path.

The `20260514-222022-startup` capture added asset-refresh plan counters and exposed the next first-tick loop: bootstrap had already synced the asset catalog, resource list, selected details, previews, and default scene, but queued startup events still replayed in the first tick and triggered `asset_refresh_reload_default_scene`. The retained host now drains those bootstrap-generated asset/editor/resource events immediately after initial workspace sync. `20260514-223023-startup` confirms the result: asset refresh inside the first event-loop frame is about 0.02 ms with zero incoming changes, the retained-host tick is about 0.38 ms, slow-path startup rebuild count drops from 2 to 1, presentation rebuild count drops from 4 to 2, and workbench model build count drops from 3 to 2. The drained counts remain visible as `ui.startup.drained_asset_change_count`, `ui.startup.drained_editor_asset_change_count`, and `ui.startup.drained_resource_change_count`.

SVG icon tree parsing now has a process cache as well. `visual_assets.rs` caches parsed `usvg::Tree` values by canonical path, modification time, and file length. This does not remove cold first-frame raster and tint work for unique icons, but it removes repeated parse-tree construction when the same SVG is requested again during chrome painting or later region repaints.

The next capture pair focused on fixed startup work that was still happening before user interaction. `20260514-225912-startup` removed eager asset/welcome dispatch bridge construction from the constructor path by keeping those hidden control documents out of the startup runtime and compiling them lazily on the first corresponding click/change route. It also showed that startup-session resolution had regressed to roughly 83 ms because the valid last project was validated and then opened, reading the same project state twice. `resolve_session.rs` now opens the last project once and validates the recent list only on failure when the Welcome page becomes visible. `20260514-232056-startup` confirms the effect: `new_resolve_startup_session` is roughly 29 ms, `new_load_shared_builtin_templates` is roughly 33 ms, and `retained_host:new` is roughly 82 ms. The remaining direct retained-host startup cost is concentrated in `apply_build_host_scene_data` and `host_presenter:present`; idle hover, asset refresh, and viewport image scenarios remain free of slow-path rebuild alerts.

## Capture Workflow

Profiling builds can be captured without adding a permanent UI panel. Set `ZIRCON_PROFILE_CAPTURE=1` before launching a profiling build; `zircon_app` starts the recorder during editor/runtime startup and exports the report when the process exits normally. Optional environment variables are:

- `ZIRCON_PROFILE_SESSION` for the report folder name.
- `ZIRCON_PROFILE_OUTPUT_ROOT` for the output root, defaulting to `target/zircon-profiles`.
- `ZIRCON_PROFILE_MAX_FRAMES`, `ZIRCON_PROFILE_MAX_SPANS`, and `ZIRCON_PROFILE_MAX_COUNTERS` for ring-buffer size.
- `ZIRCON_RUNTIME_LIBRARY` when the editor should load a staged profiling runtime DLL.

The helper script `tools/ui-profile-capture.ps1` builds profiling editor/runtime artifacts, sets these variables, runs the editor, and leaves a report under `target/zircon-profiles/<session>/`. When `CARGO_TARGET_DIR` is set, the script reads `zircon_editor.exe` and `zircon_runtime.dll` from that shared target directory's `profiling` folder instead of assuming repo-local `target/profiling`, so shared-target validation does not require manually staging binaries. Use `-AllUiScenarios` to capture startup, hover, click, drag, drawer resize, asset refresh, and viewport image scenarios as separate sessions, or pass `-ScenarioList idle_hover,click` for a narrower pass. Each scenario prints a short instruction before launch, echoes the first-fix/alert excerpt from `summary.md`, and prints the scenario's redraw/GPU/batch evidence from `ui_hotspots.json` after the editor exits.

`-AutoCloseSeconds <n>` is available for repeatable startup or idle smoke captures: the script launches the editor as a child process, waits until the main window exists, waits for the requested interval, asks that window to close normally, and treats a forced stop as a failed capture so partially exported profiles are not mistaken for valid evidence. `-AutoInteract` can be combined with auto-close to activate the editor window, wait briefly for the startup presentation, and inject scenario-specific client-area pointer movement, clicks, or drags. `-RequireScenarioEvidence` turns missing scenario evidence into a script failure; for example, click/drag/drawer-resize captures must produce a redraw plus fewer GPU draw calls than visible draw items, and viewport-image captures must produce paint-only region redraw plus GPU batch evidence. Idle hover currently treats pointer frame samples as automated event-path evidence and warns when no hover redraw occurs. Use `-UseTracy` to launch the bundled `dev/tracy/tracy-profiler.exe` for live viewing when the build includes `profiling-tracy`; use `-UseWpr` when a Windows Performance Recorder ETL is needed for WPA CPU scheduling analysis.

## Test Coverage

`zircon_editor/src/tests/host/pane_presentation.rs` covers Runtime Diagnostics payload projection of profiling counts and over-budget frames, and now checks Performance Timeline frame budget labels plus normalized bar ratios. The retained-host conversion module includes focused tests for the track, fill, budget marker, overlay label node metadata, and for updating static summary/session/output nodes without duplicating those template controls in the dynamic node list. Existing fixture construction in `pane_presentation.rs` and `template_runtime/pane_payload_projection.rs` verifies explicit `RuntimeDiagnosticsSnapshot` initializers include the new `profile` field. The app editor-host profiling check verifies `zircon_app::RuntimeSession` still exposes the optional runtime snapshot hook only through the editor-host integration path.
