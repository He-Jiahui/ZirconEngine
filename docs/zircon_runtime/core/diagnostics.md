---
related_code:
  - zircon_runtime/src/core/diagnostics/mod.rs
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/snapshot.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
plan_sources:
  - user: 2026-05-22 continue M10 render diagnostics and profiling bridge checklist
  - user: 2026-05-16 continue Bevy-style runtime Time diagnostics integration
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/docs/profiling.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
  - dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs
tests:
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/graphics/tests/render_profiling.rs
  - cargo test -p zircon_runtime --lib time --locked
  - cargo check -p zircon_runtime --profile profiling --features profiling --locked
doc_type: module-detail
---

# Core Runtime Diagnostics

`zircon_runtime::core::diagnostics` provides the read-only diagnostic snapshot surface for runtime tooling. The store contracts already existed as plain data structures; the current Bevy-parity slice makes `CoreRuntime` own one `DiagnosticStore` so frame and system metrics can accumulate in the same runtime instance that owns lifecycle, time, task pools, and services.

## Reference Evidence

Bevy's `FrameTimeDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs` registers `frame_time`, `fps`, and `frame_count` diagnostics from `Time<Real>` plus `FrameCount`. Bevy's `LogDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs` consumes the diagnostics store as a reporting layer rather than owning frame timing itself.

Zircon mirrors the ownership split: `CoreRuntime` records diagnostic measurements, while log/dev tooling can read snapshots later through `collect_runtime_diagnostics`.

## Ownership Boundary

- `DiagnosticStore` owns bounded series history, current values, smoothing, min/max, units, and subsystem tags.
- `CoreRuntimeInner` owns one `DiagnosticStore` per runtime instance.
- `CoreRuntime` and `CoreHandle` expose `record_diagnostic`, `diagnostic_store`, and `diagnostic_store_snapshot`.
- `CoreHandle::advance_time_by(...)` records Bevy-style time measurements after advancing runtime clocks.
- `collect_runtime_diagnostics` starts with the runtime-owned store and then overlays derived render, physics, animation, and profiling diagnostics.
- `diagnostic_log::format_diagnostic_store_snapshot(...)` and `write_diagnostic_store_snapshot(...)` turn store snapshots into process-log lines for dev-profile diagnostics.

The diagnostics store is not a global singleton. This keeps tests, runtime preview sessions, editor-host runtimes, and future export hosts isolated from each other.

## Render Diagnostics Bridge

M10.8 keeps render diagnostics on the same runtime-owned snapshot boundary. Bevy's `RenderDiagnosticsPlugin` records CPU/GPU pass elapsed time, pipeline statistics, and buffer-backed scalar diagnostics, then syncs finished rows into `DiagnosticsStore`. Zircon is not at that parity level yet: `RuntimeRenderDiagnostics` currently wraps a queried `RenderStats` snapshot, and `collect_runtime_diagnostics(...)` records only `render.submitted_frames`, `render.active_viewports`, and `render.last_graph_executed_pass_count` into the store.

That narrower bridge is still the correct consumer boundary. Runtime diagnostics panels, diagnostic log schedules, overlays, and editor tooling should consume `RuntimeDiagnosticsSnapshot` or `DiagnosticStoreSnapshot` instead of querying renderer-private state. The 2026-05-26 M10W focused diagnostics run passed `runtime_diagnostics` and `diagnostic_store` filters plus default/profiling checks, proving the current bridge and log cadence still work in this checkout. Promotion beyond the current bridge still requires adding stable diagnostic paths for product readiness, pass-level CPU timing, backend-gated GPU timing, pipeline/cache status, present/capture failures, render-asset residency, and mesh allocator memory. Profiling artifacts can support this evidence, but they do not replace store-backed diagnostics.

RenderDoc markers are explicitly debugging evidence, not profiling evidence. Bevy's profiling docs route GPU performance investigation through Tracy RenderQueue or vendor profilers, while RenderDoc remains a capture/debug tool. Zircon should preserve that distinction when wiring future GPU timestamp or pipeline-statistics rows.

## Time Diagnostics

Each nonzero time advance records:

- `time.frame_time` in milliseconds,
- `time.fps` in hertz,
- `time.frame_count` in frames,
- `time.fixed_steps` in fixed-step count for that outer update.

`time.frame_count` and `time.fixed_steps` are still recorded on zero-delta updates. `time.frame_time` and `time.fps` are skipped for zero deltas, matching Bevy's guard against dividing by zero.

## Test Coverage

`zircon_runtime/src/tests/time.rs` verifies that advancing runtime time records the expected frame time, FPS, frame count, and fixed-step measurements, and that `collect_runtime_diagnostics` includes those runtime-owned values.

`zircon_runtime/src/diagnostic_log/diagnostics.rs` verifies stable formatting for current, smoothed, min, and max diagnostic values. `zircon_runtime/src/tests/prelude.rs` continues to verify the public diagnostic store, snapshot, and diagnostic-log formatting helpers through the stable runtime prelude.

2026-05-26 M10W evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked runtime_diagnostics --jobs 1 --message-format short --color never`: PASS, 2 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked diagnostic_store --jobs 1 --message-format short --color never`: PASS, 5 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.
