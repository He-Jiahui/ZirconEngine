---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/time.rs
  - zircon_runtime/src/core/frame_clock.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_runtime/src/core/framework/time/mod.rs
implementation_files:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/time.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
plan_sources:
  - user: 2026-05-16 continue Bevy-style runtime Time integration
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_time/src/lib.rs
  - dev/bevy/crates/bevy_time/src/time.rs
  - dev/bevy/crates/bevy_time/src/virt.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
tests:
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - cargo test -p zircon_runtime --lib time --locked
doc_type: module-detail
---

# Core Runtime Time

`zircon_runtime::core::time` is the runtime-owned clock bundle for Bevy-style frame time. The framework layer defines the neutral clock contracts, while this core layer stores one `RuntimeTimeClocks` instance inside each `CoreRuntime` and advances it through `CoreRuntime` or `CoreHandle`.

## Reference Evidence

Bevy's `TimePlugin` in `dev/bevy/crates/bevy_time/src/lib.rs` installs `Time`, `Time<Real>`, `Time<Virtual>`, and `Time<Fixed>` resources. `dev/bevy/crates/bevy_time/src/time.rs` documents the shared `delta`, `elapsed`, and clock access model. `dev/bevy/crates/bevy_time/src/virt.rs` defines virtual pause, relative speed, and max-delta clamping. `dev/bevy/crates/bevy_time/src/fixed.rs` states that fixed time follows virtual time, so pause, speed, and clamp policy also gate fixed updates.

Bevy's `FrameTimeDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs` records frame time, FPS, and frame count from `Time<Real>` plus the frame counter, and exposes those paths as `FrameTimeDiagnosticsPlugin::{FRAME_TIME, FPS, FRAME_COUNT}`. Zircon records the same class of values into the runtime-owned `DiagnosticStore` whenever `CoreHandle::advance_time_by(...)` runs, with one extra fixed-step metric for runtime simulation budgeting.

Bevy's default plugin group in `dev/bevy/crates/bevy_internal/src/default_plugins.rs` places `TimePlugin` in both `DefaultPlugins` and `MinimalPlugins`; `TimePlugin` then runs its time update in the `First` schedule before ordinary update work. Zircon does not yet run Bevy-style ECS schedules at the app layer, so this slice keeps schedule ownership in the runtime spine while still giving the app host an explicit dynamic-library `tick_frame` call before it requests redraw.

## Ownership Boundary

- `zircon_runtime::core::framework::time` owns the plain contracts: `Time<Real>`, `Time<Virtual>`, `Time<Fixed>`, and `FixedStepPlan`.
- `zircon_runtime::core::time` owns the runtime snapshot and update summary: `RuntimeTimeClocks` and `RuntimeTimeAdvance`.
- `zircon_runtime::core::time` also owns the stable Time diagnostic path constants: `TIME_FRAME_COUNT_DIAGNOSTIC`, `TIME_FIXED_STEPS_DIAGNOSTIC`, `TIME_FRAME_TIME_DIAGNOSTIC`, and `TIME_FPS_DIAGNOSTIC`.
- `CoreRuntimeInner` owns one `FrameClock` and one `RuntimeTimeClocks` bundle per runtime instance.
- `CoreRuntime` and `CoreHandle` expose read snapshots plus deterministic `advance_time_by(...)` and wall-clock `tick_time(...)` entry points.
- `CoreRuntime` also owns a `DiagnosticStore`; time advancement records frame-time diagnostics there.
- `zircon_runtime_interface::ZrRuntimeApiV1::tick_frame` is an optional appended ABI entry that lets hosts advance a dynamic runtime session without importing runtime implementation types.
- `zircon_app::RuntimeEntryApp::about_to_wait` calls `RuntimeSession::tick_frame()` before `request_redraw()`, matching Bevy's model where the outer app loop advances time before the next frame's update/render work.

This keeps the app host out of concrete clock storage. `zircon_app` can choose when to tick, but `zircon_runtime::core` remains the authority for the clocks and fixed-step plan.

## Behavior

`advance_time_by(real_delta, max_fixed_steps)` advances `Time<Real>` by the raw wall-clock delta, advances `Time<Virtual>` from that real delta using pause, relative speed, and max-delta policy, then accumulates the virtual delta into `Time<Fixed>`. Fixed steps drain up to the caller-supplied max step budget and return a `RuntimeTimeAdvance` with the original real delta plus the resulting `FixedStepPlan`. The same call records `TIME_FRAME_TIME_DIAGNOSTIC`, `TIME_FPS_DIAGNOSTIC`, `TIME_FRAME_COUNT_DIAGNOSTIC`, and `TIME_FIXED_STEPS_DIAGNOSTIC` into the runtime-owned diagnostic store.

The diagnostic path constants are `&'static str` values rather than Bevy-style const `DiagnosticPath` values because Zircon's `DiagnosticPath` currently owns a `String`. Keeping the public constants in `core::time` still gives callers a stable prelude-visible contract while avoiding a broader storage refactor in the diagnostics module.

`tick_time(max_fixed_steps)` reads the runtime-owned `FrameClock`, then delegates to the deterministic path. This keeps tests and replay-style callers able to bypass the wall clock, while real app loops still have a single runtime-owned clock path.

The dynamic runtime API exposes that wall-clock path through optional `tick_frame(session)`. `zircon_app` treats the function as optional for older ABI-v1 runtimes, but the current runtime exports it and routes the call to `RuntimeDynamicSession::tick_frame`, which advances the owned `CoreRuntime` with a local fixed-step cap selected from the session profile. The default cap is deliberately local to the dynamic session because it is host-loop policy rather than a cross-crate protocol value.

Virtual-time settings are adjusted through explicit methods on `CoreRuntime` and `CoreHandle`: pause, unpause, max-delta, relative speed, and fixed timestep. The methods mutate only the runtime time bundle and do not run schedules directly.

## Test Coverage

`zircon_runtime/src/tests/time.rs` covers:

- real, virtual, and fixed clocks advancing together from one runtime call,
- Bevy-style default fixed timestep behavior at 64 Hz,
- virtual pause halting both virtual delta and fixed accumulation,
- virtual relative speed and max-delta clamping feeding fixed-step planning,
- frame-time, FPS, frame-count, and fixed-step diagnostics collected through `collect_runtime_diagnostics`.

`zircon_runtime/src/tests/prelude.rs` verifies that `RuntimeTimeClocks`, `RuntimeTimeAdvance`, and the four Time diagnostic path constants are part of the stable runtime prelude.

Dynamic/app integration coverage now also verifies that:

- `ZrRuntimeApiV1` records `tick_frame` as the appended optional ABI field after `profile_control`,
- `zircon_runtime::dynamic_api` exports `tick_frame`, rejects unknown sessions, and accepts valid sessions,
- dynamic session creation rejects unknown profile bytes before runtime bootstrap and accepts the named `dev` profile,
- `zircon_app` loads the optional function through offset-gated table access, and
- `RuntimeEntryApp::about_to_wait` calls `session.tick_frame()` before requesting redraw.

The dev-profile continuation also guards that the dynamic `dev` session wires a `DiagnosticStoreLogSchedule` into the same `tick_frame` path, so Bevy-style time diagnostics can be emitted through `diagnostic_log` without widening the app/runtime ABI.
