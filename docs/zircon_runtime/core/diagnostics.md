---
related_code:
  - zircon_runtime/src/core/diagnostics/mod.rs
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
plan_sources:
  - user: 2026-05-16 continue Bevy-style runtime Time diagnostics integration
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
  - dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs
tests:
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/tests/prelude.rs
  - cargo test -p zircon_runtime --lib time --locked
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
