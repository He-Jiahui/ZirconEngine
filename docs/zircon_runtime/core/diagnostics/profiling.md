---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/diagnostics/profiling/recorder.rs
  - zircon_runtime/src/core/diagnostics/profiling/scope.rs
  - zircon_runtime/src/core/diagnostics/profiling/tracy.rs
  - zircon_runtime/src/core/diagnostics/profiling/hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/snapshot.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/diagnostics/profiling/recorder.rs
  - zircon_runtime/src/core/diagnostics/profiling/scope.rs
  - zircon_runtime/src/core/diagnostics/profiling/tracy.rs
  - zircon_runtime/src/core/diagnostics/profiling/hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
plan_sources:
  - .codex/plans/Zircon 性能时间轴与 Tracy 集成设计.md
  - user: 2026-05-13 continue profiling timeline and Tracy integration milestone
tests:
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/diagnostics/profiling/recorder.rs
  - zircon_runtime/src/core/diagnostics/profiling/hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - target: cargo check -p zircon_runtime --profile profiling --features profiling --locked
  - target: cargo test -p zircon_runtime --lib profiling --profile profiling --features profiling --locked
  - target: cargo check -p zircon_runtime --profile profiling --features "profiling profiling-tracy" --locked
  - target: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-tracy profiling-chrome" --locked
doc_type: module-detail
---

# Runtime Profiling Diagnostics

## Purpose

`zircon_runtime::core::diagnostics::profiling` owns Zircon's first CPU timeline spine. It is intentionally below graphics, dynamic runtime sessions, and editor host code so those layers can add spans without owning recorder state or export formats.

The subsystem is compiled behind the `profiling` feature. The workspace adds a dedicated `profiling` Cargo profile that inherits release optimizations while retaining debug symbols. `zircon_runtime/build.rs` rejects ordinary `--release` builds that enable `profiling`, `profiling-chrome`, `profiling-tracy`, or `profiling-memory`; profiling runs should use `cargo build --profile profiling --features profiling ...`.

## Runtime Shape

`ProfileRecorder` is a process-local ring-buffer recorder. `start_capture` normalizes `ProfileCaptureConfig`, resets the origin timestamp, clears existing frames/spans/counters, and starts accepting samples. `stop_capture` leaves captured samples readable but stops accepting new ones. `reset_capture` clears all sample buffers.

`ProfileScope` and `ProfileFrameScope` are RAII guards created by macros. Scope state is thread-local so nested spans can record parent ids, path strings, depth, and the current frame index without passing context through every call. Frame scopes track one monotonically increasing frame index per stream, so editor and runtime frames can coexist in the same snapshot.

The public macros are:

- `profile_frame!(stream, name)` for frame boundaries.
- `profile_scope!(stream, category, name)` for CPU span samples.
- `profile_counter!(stream, name, value)` for instantaneous counters.

When `profiling` is disabled, the macro bodies are cfg-stripped and do not evaluate their arguments. When `profiling-tracy` is enabled, the same macros also emit `tracing` spans or events and `profile_frame!` creates a Tracy frame-mark guard that emits `tracy.frame_mark = true` when the frame scope exits.

## Tracy Sink

`profiling/tracy.rs` installs `tracing_tracy::TracyLayer` through `initialize_tracy_sink`. Installation is idempotent per linked image and returns a status instead of panicking when another subscriber is already installed. `zircon_app` calls it during editor/runtime process startup, and `zircon_runtime::dynamic_api::zircon_runtime_get_api_v1` calls it for the dynamically loaded runtime image. This covers both statically linked app/editor spans and the runtime cdylib's own tracing statics without moving process startup policy into the recorder.

The sink follows the same reference shape used by Bevy's `trace_tracy` support: spans are regular `tracing` spans, and frame boundaries are info events containing the `tracy.frame_mark` field so Tracy can draw frame marks in the external GUI.

## Export And Hotspots

`export_report` snapshots the recorder, analyzes hotspots, and writes profiling artifacts under `<output_root>/<session-id>/`:

- `timeline.zrtrace.json`: native Zircon snapshot JSON.
- `timeline.perfetto.json`: Chrome/Perfetto complete-event JSON, written only when the build includes `profiling-chrome` and the capture config keeps `include_perfetto = true`.
- `hotspots.json`: grouped span-cost report.
- `ui_hotspots.json`: retained-host UI slow-path counter aggregation.
- `summary.md`: human-readable frame/span/counter and top-hotspot summary.

`analyze_hotspots` groups spans by `stream/category/name/path`. It reports total, average, p95, max, count, distinct frame count, and over-budget count. Hints are intentionally conservative: they only point to recorded span names that exceeded or accumulated against the configured budget, and they do not infer causes that were not sampled.

## Instrumentation Boundaries

The first profiling slice records coarse CPU spans at stable engine seams:

- Dynamic runtime ABI calls: event handling, frame capture, accessibility capture, viewport surface bind/unbind, and present.
- `RuntimeRenderBridge`: extract submit, surface bind/unbind, and present.
- Render framework submit/present/capture internals: submission context build, runtime submission preparation, render/present pipeline, feedback collection, and counters for submitted frames.
- Core lifecycle: module register, activate, deactivate, and service resolution.

Upper-layer app/editor spans are deliberately consumers of this core module; the recorder remains in runtime diagnostics and does not move process-host or authoring state into runtime world data.

## Diagnostics Snapshot

`RuntimeDiagnosticsSnapshot` now carries `profile: ProfileSnapshot`. `collect_runtime_diagnostics` pulls the in-process profiling snapshot next to render, physics, animation, and diagnostic-store data so existing diagnostics panels can display profile state without a separate runtime-owned UI path.

## Test Coverage

Recorder tests cover ring-buffer truncation. Profiling macro tests cover nested span parentage and disabled-feature no-op argument behavior. Hotspot tests cover total/p95 ordering. Export tests cover Perfetto event shape and expected artifact names. Dynamic API tests cover optional `profile_control` exposure, invalid JSON rejection before session lookup, and snapshot serialization.
