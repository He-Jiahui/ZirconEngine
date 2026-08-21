---
related_code:
  - tools/dev-fast-build.ps1
  - tools/zircon_build.py
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - dev/bevy/docs/profiling.md
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/tracy.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_interface/src/profiling/session_path.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - tools/profile-capture-paths.ps1
  - tools/ui-profile-capture.ps1
  - tools/mvp/Capture-RenderExtractBaseline.ps1
  - tools/tests/ui-profile-capture-output-contract.Tests.ps1
  - tools/tests/render-extract-baseline-capture.Tests.ps1
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/render.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/snapshot.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/tests/render_profiling.rs
implementation_files:
  - tools/dev-fast-build.ps1
  - tools/zircon_build.py
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/tracy.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_interface/src/profiling/session_path.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - tools/profile-capture-paths.ps1
  - tools/ui-profile-capture.ps1
  - tools/mvp/Capture-RenderExtractBaseline.ps1
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - .codex/plans/Zircon 性能时间轴与 Tracy 集成设计.md
  - user: 2026-05-13 continue profiling timeline and Tracy integration milestone
  - user: 2026-05-22 continue M10 render diagnostics and profiling bridge checklist
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/zircon_runtime/graphics/render-product-submit.md
tests:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs
  - planned milestone test: ring_push_evicts_oldest_sample_at_capacity
  - zircon_runtime/src/core/runtime/diagnostics/profiling/hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling/session_path.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment/tests.rs
  - tools/tests/ui-profile-capture-output-contract.Tests.ps1
  - tools/tests/render-extract-baseline-capture.Tests.ps1
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime/src/graphics/tests/render_profiling.rs
  - target: cargo check -p zircon_runtime --profile profiling --features profiling --locked
  - target: cargo test -p zircon_runtime --lib profiling --profile profiling --features profiling --locked
  - target: cargo test -p zircon_runtime --lib profile_scope_enter_named_captures_runtime_generated_names --profile profiling --features profiling --locked --message-format=short
  - target: cargo test -p zircon_runtime --lib profile_dynamic_scope_macro_captures_runtime_generated_names --profile profiling --features profiling --locked --message-format=short
  - target: cargo test -p zircon_runtime --lib render_submit_records_render_graph_pass_and_wait_spans --profile profiling --features profiling --locked --message-format=short
  - target: cargo test -p zircon_runtime --lib direct_runtime_frame_submit_nests_render_graph_spans_under_pipeline_scope --profile profiling --features profiling --locked --message-format=short
  - target: cargo test -p zircon_runtime --lib direct_runtime_frame_submit_exports_perfetto_trace_artifacts --profile profiling --features "profiling profiling-chrome" --locked --message-format=short
  - target: cargo check -p zircon_runtime --profile profiling --features "profiling profiling-tracy" --locked
  - target: python tools/zircon_build.py --targets runtime --out E:\builds\zircon-profile --mode profiling --runtime-features target-client,profiling,profiling-tracy --dry-run
  - target: ./tools/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride "target-client profiling profiling-tracy"
  - target: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-tracy profiling-chrome" --locked
  - target: cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Profiling Diagnostics

## Purpose

`zircon_runtime::core::diagnostics::profiling` owns Zircon's first CPU timeline spine. It is intentionally below graphics, dynamic runtime sessions, and editor host code so those layers can add spans without owning recorder state or export formats.

The subsystem is compiled behind the `profiling` feature. The workspace adds a dedicated `profiling` Cargo profile that inherits release optimizations while retaining debug symbols. `zircon_runtime/build.rs` rejects ordinary `--release` builds that enable `profiling`, `profiling-chrome`, `profiling-tracy`, or `profiling-memory`; profiling runs should use `cargo build --profile profiling --features profiling ...`.

Runtime 07 M0.2 now has tool-level entry points for that profile. The staged
build tool maps `--mode profiling` to Cargo `--profile profiling`, and
`--runtime-features target-client,profiling,profiling-tracy` turns on the
runtime spans and Tracy bridge for a client runtime sample. The fast-build helper
uses the same Cargo profile through `-CargoProfile profiling`:

```powershell
python tools/zircon_build.py --targets runtime --out E:\builds\zircon-profile --mode profiling --runtime-features target-client,profiling,profiling-tracy
./tools/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride "target-client profiling profiling-tracy"
```

Those commands are the reproducible M0.2 build entry points. As of 2026-06-17
the tool path is statically wired, while the actual profiling build result is
still deferred until no other Cargo/rustc lane is using the shared checkout.

## Runtime Shape

`ProfileRecorder` is a process-local ring-buffer recorder. Frames, spans, and counters use `VecDeque`: when a configured limit is reached, `push_ring` removes the oldest sample with `pop_front` and appends the new sample with `push_back`. Eviction is therefore constant time and does not shift every retained element. This matters most for spans because the default capture limit is 16,384; a full `Vec` plus `remove(0)` would otherwise make every later recorded span move the remaining buffer and distort the profile being measured. `snapshot` collects each deque from oldest to newest into the public `Vec` DTO and publishes one `ProfileRecorderRetentionSnapshot` with capacity, written, overwritten, retained, oldest-sequence, and newest-sequence evidence for each queue. `start_capture` and `reset` clear both samples and sequence authority. Editor/runtime snapshot merge appends recorder rows rather than aggregating them, preserving which bounded source lost samples.

`tools/ui-profile-latency-evidence.ps1` consumes that retention authority when it derives `ui_surface_present_outcomes.json` schema 4. Interaction acceptance fails closed when the timeline has no recorder rows, a row violates `written = overwritten + retained` or its sequence bounds, or any frame/span/counter queue overwrote a measured sample. With complete zero-overwrite evidence, the current UI interaction budgets are `input_to_damage p95 <= 1,000 us` and `damage_to_submit p95 <= 8,000 us`. These gates do not yet prove that every pointer action produced a typed damage/no-damage outcome; that separate sequence/outcome contract remains required before a product stress milestone can be promoted.

`tools/ui-profile-process-evidence.ps1` owns the matching process-cost contract. It cross-checks measured processor time against the exported per-core and whole-system percentages, rejects an average above one logical core, caps end-of-run working-set and private-byte growth at 64 MiB, and caps their peak growth at 96 MiB. Scenario gates also normalize processor time by completed work: click storms allow 0.5 ms per click, pointer-move and wheel storms allow 0.25 ms per event, and native resize allows 35 ms per completed step. These are acceptance ceilings, not observed performance claims. `tools/ui-profile-capture.ps1` defaults to one warmup run followed by three separately source-bound measured runs with a two-second quiescence interval; only measured runs can satisfy `RequireScenarioEvidence`, and each measured run must pass independently.

This bounded-history shape follows `dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs`, where `Diagnostic` also uses `VecDeque`, `pop_front`, and `push_back` for capped measurement history. Zircon retains separate frame/span/counter queues and the existing `ProfileCaptureConfig` limits because its export DTO and hotspot analyzers consume those three streams independently.

`start_capture` normalizes `ProfileCaptureConfig`, resets the origin timestamp, clears existing frames/spans/counters, and starts accepting samples. `stop_capture` leaves captured samples readable but stops accepting new ones. `reset_capture` clears all sample buffers.

`ProfileScope` and `ProfileFrameScope` are RAII guards created by macros. Scope state is thread-local so nested spans can record parent ids, path strings, depth, and the current frame index without passing context through every call. Frame scopes track one monotonically increasing frame index per stream, so editor and runtime frames can coexist in the same snapshot.

The public macros are:

- `profile_frame!(stream, name)` for frame boundaries.
- `profile_scope!(stream, category, name)` for CPU span samples.
- `profile_dynamic_scope!(stream, category, name)` for runtime-generated scope names such as render pass or stage names.
- `profile_counter!(stream, name, value)` for instantaneous counters.

When `profiling` is disabled, the static macro bodies are cfg-stripped and do not evaluate their arguments. `profile_dynamic_scope!` only evaluates its name when either `profiling` or `profiling-tracy` is enabled, so render graph pass names are not allocated in ordinary runtime builds. When `profiling-tracy` is enabled, the same macros also emit `tracing` spans or events and `profile_frame!` creates a Tracy frame-mark guard that emits `tracy.frame_mark = true` when the frame scope exits.

## Tracy Sink

`profiling/tracy.rs` installs `tracing_tracy::TracyLayer` through `initialize_tracy_sink`. Installation is idempotent per linked image and returns a status instead of panicking when another subscriber is already installed. `zircon_app` calls it during editor/runtime process startup, and `zircon_runtime::dynamic_api::zircon_runtime_get_api_v2` calls it for the dynamically loaded runtime image. This covers both statically linked app/editor spans and the runtime cdylib's own tracing statics without moving process startup policy into the recorder.

The sink follows the same reference shape used by Bevy's `trace_tracy` support: spans are regular `tracing` spans, and frame boundaries are info events containing the `tracy.frame_mark` field so Tracy can draw frame marks in the external GUI.

## Export And Hotspots

`export_report` snapshots the recorder, analyzes hotspots, and writes profiling artifacts under `<output_root>/<session-basename>/`:

- `timeline.zrtrace.json`: native Zircon snapshot JSON.
- `timeline.perfetto.json`: Chrome/Perfetto complete-event JSON, written only when the build includes `profiling-chrome` and the capture config keeps `include_perfetto = true`.
- `hotspots.json`: grouped span-cost report.
- `counter_hotspots.json`: generic finite positive counter aggregation for Runtime 07 evidence streams such as extract, ECS, asset worker, animation scene, time, schedule, and task counters.
- `ui_hotspots.json`: retained-host UI slow-path counter aggregation.
- `summary.md`: human-readable frame/span/counter and top-hotspot summary.

The session directory is always one sanitized child basename of `output_root`. Empty or dot-only IDs become `session`, leading and trailing dots are removed, separators and other non-portable UTF-8 bytes become `_`, and Windows device basenames (`CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, and `LPT1`-`LPT9`, including extensions) receive a `session_` prefix. A stable 64-bit hash suffix distinguishes lossy or truncated IDs, and the complete basename is capped at 96 ASCII bytes. Runtime export, Editor artifact discovery, UI profile capture and MVP render-extract capture consume the same `zircon_runtime_interface`/PowerShell contract. This closes parent-directory traversal, device-name, collision and path-length failures; atomic staging, a capture manifest, checksums, stale-artifact removal, and one-step publication remain open under Runtime03 P1-7.

Runtime 15 F5 profile export typed errors are now owned here under status `runtime_15_profile_export_typed_errors_static_passed_cargo_deferred`. `export.rs` exposes `ProfileExportError` / `ProfileExportResult`, with `ProfileExportError::FeatureDisabled`, `ProfileExportError::CreateExportDirectory`, `ProfileExportError::JsonSerialize`, and `ProfileExportError::WriteFile` preserving the failing feature gate, directory, JSON, and file-write sources. The only string downgrade remains the external `ProfileControlResponse` DTO boundary. `review_f5_profile_export_uses_typed_error` locks this contract across Runtime 15 plans, status-output expectations, and this module doc.

`analyze_hotspots` groups spans by `stream/category/name/path`. It reports total, average, p95, max, count, distinct frame count, and over-budget count. Hints are intentionally conservative: they only point to recorded span names that exceeded or accumulated against the configured budget, and they do not infer causes that were not sampled.

`analyze_counter_hotspots` produces `CounterHotspotReport` by grouping finite positive `ProfileCounterSnapshot` values by `stream/name/path`. Each `CounterHotspotEntry` reports total, average, p95, max, latest, sample count, and distinct frame count. The export is evidence ranking only: it does not promote a Runtime 07 M2 optimization without adjacent frame-span or authoritative FPS/profile samples. `ProfileControlResponse.counter_hotspot_report` returns the same report as part of `export_report`, and `summary.md` includes a `Counter Hotspots` section plus first-fix candidates only after UI alerts and span hotspots.

## Instrumentation Boundaries

The first profiling slice records coarse CPU spans at stable engine seams:

- Dynamic runtime ABI calls: event handling, frame capture, accessibility capture, viewport surface bind/unbind, and present.
- `RuntimeRenderBridge`: extract submit, surface bind/unbind, and present.
- Render framework submit/present/direct-runtime-frame/capture internals: submission context build, runtime submission preparation, render/present pipeline, feedback collection, and counters for submitted frames.
- Render-framework contention: `WgpuRenderFramework::lock_operation` and `lock_state` wrap acquisition of the serialized operation mutex and mutable framework-state mutex in `render_framework.wait` spans. These spans measure CPU lock-acquire time at the runtime render-framework boundary without exposing editor/UI batching internals.
- Render graph execution: `execute_graph_stage` records `render_graph.stage:<Stage>` spans and each non-culled executable pass records `render_graph.pass:<pass-name>` beneath the active runtime render/present pipeline span. The pass span surrounds executor dispatch plus execution-record update so the M4 timeline can attribute CPU render work by compiled graph stage and pass while GPU timestamp work remains a later extension.
- Core lifecycle: module register, activate, deactivate, and service resolution.

Upper-layer app/editor spans are deliberately consumers of this core module; the recorder remains in runtime diagnostics and does not move process-host or authoring state into runtime world data.

## M10.8 Render Profiling Boundary

The profiling module is one evidence source for M10.8, not the whole render diagnostics bridge. Bevy separates the surfaces: `RenderDiagnosticsPlugin` records render pass CPU/GPU elapsed time, pipeline statistics, and buffer-backed scalar diagnostics into `DiagnosticsStore`, while `docs/profiling.md` explains CPU tracing, Tracy RenderQueue, and vendor GPU profilers as profiling workflows. It also states that RenderDoc is a debugging tool, not a profiler.

Zircon's current profiling support records CPU timeline spans around render submit, surface present, capture, framework locks, graph stages, and graph passes in profiling builds. Those spans can prove where CPU render work sits in a captured timeline and can support hotspot/perfetto artifacts. They do not prove GPU timestamp queries, pipeline-statistics rows, render-asset residency, mesh allocator memory, or render-thread overlap telemetry.

M10.8 promotion therefore needs two linked but separate outputs: store-backed render diagnostics for normal runtime/dev tooling, and profiling artifacts for optional timeline analysis. A profiling artifact smoke test can support the gate only when the corresponding `RuntimeDiagnosticsSnapshot` / `DiagnosticStore` paths remain the tooling boundary. The 2026-05-26 M10W focused run passed the profiling-profile tests and check, while the normal diagnostics filters passed separately; this promotes only CPU timeline/artifact support and not GPU timestamp, pipeline-statistics, render-asset residency, or render-thread telemetry gaps.

## Diagnostics Snapshot

`RuntimeDiagnosticsSnapshot` now carries `profile: ProfileSnapshot`. `collect_runtime_diagnostics` pulls the in-process profiling snapshot next to render, physics, animation, and diagnostic-store data so existing diagnostics panels can display profile state without a separate runtime-owned UI path.

## Test Coverage

Process-evidence tests cover one-core CPU, CPU-per-operation, CPU-percentage consistency, end/peak memory growth, and exact budget boundaries. The broader capture output contract verifies the latency/process modules and the warmup/measured-run policy are source-bound in the capture manifest.

Recorder tests cover ring-buffer truncation, the exact overwrite booleans, per-stream retention/sequence publication, and reset of that authority. `ring_push_evicts_oldest_sample_at_capacity` fixes the storage contract to a deque-shaped bounded queue and verifies oldest-first eviction while preserving sample order. The PowerShell latency module tests cover schema 4 export, missing retention, any overwrite, both p95 budget edges, and valid zero-overwrite evidence; the broader capture output contract also verifies the module is source-bound in the capture manifest. Profiling macro tests cover nested span parentage, dynamic runtime-generated scope names, and disabled-feature no-op argument behavior. Hotspot tests cover total/p95 ordering. Counter hotspot tests cover counter grouping, ordering, finite-positive filtering, latest sample tracking, and `counter_hotspots.json` export/summary presence. Export tests cover profile artifact writing, perfetto opt-out, and typed directory creation failures through `export_snapshot_reports_typed_directory_error_source`. Dynamic API tests cover optional `profile_control` exposure, invalid JSON rejection before session lookup, and snapshot serialization. Graphics profiling tests submit real headless generated and direct runtime frames in profiling builds and assert that operation/state wait spans, render-framework build/prepare/render/feedback spans, plus render graph stage/pass spans appear in the captured runtime timeline with the expected nesting. Runtime 07 F3 also has `direct_runtime_frame_submit_exports_perfetto_trace_artifacts` for `profiling-chrome`: it submits a direct `ViewportRenderFrame`, exports `timeline.zrtrace.json`, `timeline.perfetto.json`, `hotspots.json`, and `summary.md`, and requires both trace files to retain the `submit_runtime_frame` / `render_frame_with_pipeline` / `DepthPrepass` / `depth-prepass` path. Its status anchor is `render_direct_runtime_frame_trace_export_static_passed_profile_timeout_fps_pending`: static guards cover the source/docs anchors, while the cargo profiling test still needs a clean build lane and the authoritative vampire FPS gate remains open.

2026-05-26 M10W evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --lib profiling --profile profiling --features profiling --locked --jobs 1 --message-format short --color never`: PASS, 20 matching lib tests passed after the initial cold profiling-profile compile timed out before test execution.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --profile profiling --features profiling --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.
