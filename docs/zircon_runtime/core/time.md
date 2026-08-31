---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/time/product_policy.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/clock_source.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/mod.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/framework/time/policy.rs
implementation_files:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/time/product_policy.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/mod.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-time-diagnostics-static-review.md
  - user: 2026-05-16 continue Bevy-style runtime Time integration
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_time/src/lib.rs
  - dev/bevy/crates/bevy_time/src/time.rs
  - dev/bevy/crates/bevy_time/src/virt.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
tests:
  - tools/tests/test_frameworks_01_time_product_policy_owner_boundary.py
  - zircon_runtime/src/core/runtime/diagnostics/store.rs::tests::static_diagnostic_series_reuses_path_and_metadata_allocations
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - cargo test -p zircon_runtime --lib tests::time:: --locked
doc_type: module-detail
---

# Core Runtime Time

`zircon_runtime::core::runtime::time` owns the outer monotonic frame handoff and default policy for subsequently created Worlds. The framework layer defines neutral clock contracts; each `LevelSystem` owns its derived virtual/fixed clocks and fixed-step commit boundary.

## Reference Evidence

Bevy's `TimePlugin` in `dev/bevy/crates/bevy_time/src/lib.rs` installs generic real, virtual, and fixed time resources. Zircon represents its frame source explicitly as `Time<MonotonicReal>`, alongside `Time<Virtual>` and `Time<Fixed>`. `dev/bevy/crates/bevy_time/src/time.rs` documents the shared `delta`, `elapsed`, and clock access model. `dev/bevy/crates/bevy_time/src/virt.rs` defines virtual pause, relative speed, and max-delta clamping. `dev/bevy/crates/bevy_time/src/fixed.rs` states that fixed time follows virtual time, so pause, speed, and clamp policy also gate fixed updates.

Bevy's `FrameTimeDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs` records frame time, FPS, and frame count from its real-time resource plus the frame counter, and exposes those paths as `FrameTimeDiagnosticsPlugin::{FRAME_TIME, FPS, FRAME_COUNT}`. Zircon records the same real-time values from `Time<MonotonicReal>` into the runtime-owned `DiagnosticStore` whenever `CoreHandle::advance_time_by(...)` runs. Fixed-step telemetry must be World-scoped and committed; it is not a core global metric.

Bevy's default plugin group in `dev/bevy/crates/bevy_internal/src/default_plugins.rs` places `TimePlugin` in both `DefaultPlugins` and `MinimalPlugins`; `TimePlugin` then runs its time update in the `First` schedule before ordinary update work. Zircon does not yet run Bevy-style ECS schedules at the app layer, so this slice keeps schedule ownership in the runtime spine while still giving the app host an explicit dynamic-library `tick_frame` call before it requests redraw.

## Ownership Boundary

- `zircon_runtime::core::framework::time` owns the plain clock contracts plus requested policy validation: `ClockDomainRegistry`, `ClockDomainId`, `ClockDomainDescriptor`, `ClockDomainStamp`, `Time<MonotonicReal>`, `Time<Virtual>`, `Time<Fixed>`, `FixedStepPlan`, `TimePolicy`, `TimePolicyTransaction`, `TimePolicyError`, and the versioned `ProductTimePolicy` DTO/profile/error contracts.
- `zircon_runtime::core::runtime::time` owns `ProductTimePolicies::for_profile(...)` and `ProductTimePolicyDigest::from_policy(...)`; product default selection and BLAKE3 hashing do not live in the neutral framework owner. It also owns a private outer-frame/default-policy authority, immutable `FrameTimeSnapshot`, `FrameTimeDiscontinuity`, and `TimePolicyReceipt`. `RuntimeTimeClocks` is intentionally absent: no global virtual or fixed simulation clock exists.
- `zircon_runtime::core::runtime::time` owns the stable real-frame diagnostic paths: `TIME_FRAME_COUNT_DIAGNOSTIC`, `TIME_FRAME_TIME_DIAGNOSTIC`, and `TIME_FPS_DIAGNOSTIC`.
- `CoreRuntimeInner` owns one `FrameClock` and one private outer-frame/default-policy authority per runtime instance. `ClockSource` is the narrow injection contract for `FrameClock` samples; the default `SystemMonotonic` source preserves the direct `Instant::now()` path. `ManualClockSource` is the public caller-driven source for deterministic tests and replay input: it accepts only non-decreasing elapsed samples, so replay seek must create a new source and explicitly rebase the frame clock. `CoreRuntime::with_clock_source(...)` remains the general injection point for future external authorities. `FrameClockRebaseReceipt` records each explicit baseline generation, typed cause, and first-tick strategy.
- `CoreRuntime` and `CoreHandle` expose read snapshots plus deterministic `advance_time_by(...)`, wall-clock `tick_time(...)`, and `submit_clock_discontinuity(...)` entry points.
- `CoreRuntime` also owns a `DiagnosticStore`; time advancement records frame-time diagnostics there.
- `zircon_runtime_interface::ZrRuntimeApiV2::tick_frame` is an optional function entry in the V2 table that lets hosts advance a dynamic runtime session without importing runtime implementation types.
- `zircon_app::RuntimeEntryApp::about_to_wait` calls `RuntimeSession::tick_frame()` before `request_redraw()`, matching Bevy's model where the outer app loop advances time before the next frame's update/render work.

This keeps the app host out of concrete clock storage. `zircon_app` can choose when to tick; `zircon_runtime::core` remains the authority for outer real time while `LevelSystem` remains the authority for simulation time.

## Behavior

`advance_time_by(real_delta, max_fixed_steps)` advances only `Time<MonotonicReal>` by the raw monotonic frame delta. It returns an immutable `FrameTimeSnapshot` containing the real delta, outer-frame index, real `ClockDomainStamp`, optional discontinuity, and the maximum number of fixed steps an outer-frame owner permits each World to commit. `LevelSystem` derives virtual delta, pause, speed, fixed debt, and `FixedStepPlan` from this evidence under its own policy. The same core call records `TIME_FRAME_TIME_DIAGNOSTIC`, `TIME_FPS_DIAGNOSTIC`, and `TIME_FRAME_COUNT_DIAGNOSTIC` into the runtime-owned diagnostic store.

The diagnostic path constants are `&'static str` values rather than Bevy-style const `DiagnosticPath` values because Zircon's `DiagnosticPath` currently owns a `String`. Keeping the public constants re-exported through the curated `core` and prelude facades still gives callers a stable contract while avoiding a broader storage refactor in the diagnostics module.

Time diagnostics use the runtime store's static-series fast path. The first sample establishes the owned path, unit, and tags; subsequent samples with the same metadata perform a borrowed path lookup and update only the numeric history. All three real-frame rows are written while holding one diagnostic-store lock, avoiding repeated lock cycles and metadata allocations per frame. Dynamic diagnostic paths continue to use the generic record API.

`tick_time(max_fixed_steps)` reads the runtime-owned `FrameClock`, then delegates to the deterministic path. `ManualClockSource` advances only when its owner supplies a positive duration or a later elapsed position, so tests do not sleep and a malformed replay input cannot turn into an unqualified zero-delta frame. Real app loops still have a single runtime-owned clock path. The injection boundary is intentionally not a global clock service: task deadlines, OS file watchers, telemetry, wall-clock presentation, and profiling continue to use their owner-specific time sources, so they remain correct while the runtime clock is paused, replayed, or controlled.

After complete dynamic-session construction succeeds, construction calls the crate-internal `CoreRuntime::rebase_frame_clock()` immediately before returning the runnable session. Its `FrameClockRebaseReceipt` records `MeasureFromRebase`: the first runtime tick measures real time after the activation rebase while excluding module, project, level, UI, and operation-handler loading elapsed time. It does not synthesize a zero delta or introduce another clock source.

`ClockDiscontinuity` is the only platform-facing route into this authority. The dynamic runtime adapter maps foreground, background, suspended, resumed, window-occlusion, and surface-recreated events to a typed cause, then submits it through `CoreRuntime::submit_clock_discontinuity(...)`. The accepted `FrameClockRebaseReceipt` records `FrameClockRebaseCause::ClockDiscontinuity(...)` plus `MeasureFromRebase`; the next `FrameTimeSnapshot` carries that receipt once through `FrameTimeDiscontinuity::FrameClockRebased` and stamps its real domain. Each Level copies that source generation into its own virtual/fixed stamps when it accepts the frame. This replaces the prior behavior where lifecycle signals only changed input focus and a long pause could reach the next tick as an unqualified wall-clock gap.

The dynamic runtime API exposes that wall-clock path through optional `tick_frame(session)`. `zircon_app` treats the function as optional for older ABI-v1 runtimes, but the current runtime exports it and routes the call to `RuntimeDynamicSession::tick_frame`. During session construction, the selected versioned `ProductTimePolicy` is validated and committed before module activation; each frame reads its bounded fixed-step budget from that accepted policy rather than from a local raw constant.

Virtual pause/unpause is Level state. `CoreRuntime::apply_time_policy(...)` and `CoreHandle::apply_time_policy(...)` validate and replace the default policy for subsequently created Levels; they do not mutate existing World clocks, debt, or epochs. A rejected policy returns a typed `TimePolicyError` and leaves the previous default and generation intact; an accepted policy returns `TimePolicyReceipt` with the previous and applied policies, generation, and whether the effective default changed. Existing Levels use `LevelSystem::apply_time_policy(...)`, which rejects all mutation during an active fixed transaction and rejects a fixed-timestep change while debt is pending. Virtual-only policy changes may proceed with debt, but advance only the virtual clock-domain epoch.

`LevelSystem::fixed_interpolation_context()` and `RuntimeSceneSystemContext::fixed_interpolation()` expose two committed fixed endpoints, actual current debt, timestep, and a bounded residual fraction. The initial World is represented as a committed baseline without a `SimulationTickId`. These APIs never expose a begun step, and World replacement clears the interpolation history before a new generation can publish a fixed tick.

Core commits only the default-policy generation. A Level commits its own policy generation and simulation timeline. Neither configuration transaction runs schedules or silently reinterprets fixed debt; replay and rate-change migration remain separate time-architecture work.

## Test Coverage

`zircon_runtime/src/tests/time.rs` covers:

- outer real-clock delta, frame index, source stamp, discontinuity, and fixed-step budget handoff,
- default World-policy transactions without creation of a duplicate derived clock,
- source-generation propagation through an outer real-domain discontinuity snapshot,
- frame-clock rebase receipts incrementing monotonically and recording the activation baseline strategy,
- injected monotonic sources driving a frame tick and rebase without sleeping or reading the system clock,
- `ManualClockSource` rejecting backward replay samples while preserving its accepted elapsed position,
- frame-time, FPS, and frame-count diagnostics collected through `collect_runtime_diagnostics` without a global fixed-step row.

`zircon_runtime/src/tests/prelude.rs` verifies that `FrameTimeSnapshot` and the three real-frame diagnostic path constants are part of the stable runtime prelude. World-local virtual/fixed behavior is covered in `scene/tests/ecs_schedule`.

Dynamic/app integration coverage now also verifies that:

- `ZrRuntimeApiV2` records `tick_frame` after `profile_control`; V2 table layout is negotiated as a whole while the function pointer remains capability-optional,
- `zircon_runtime::dynamic_api` exports `tick_frame`, rejects unknown sessions, and accepts valid sessions,
- dynamic session creation rejects unknown profile bytes before runtime bootstrap and accepts the named `dev` profile,
- `zircon_app` loads the optional function through offset-gated table access, and
- `RuntimeEntryApp::about_to_wait` calls `session.tick_frame()` before requesting redraw.
- successful dynamic-session construction records the initial frame-clock rebase receipt before its first tick.

The static-series storage-reuse regression is implemented in `core/runtime/diagnostics/store.rs`; coordinated Cargo and allocation-benchmark validation remain pending.

The dev-profile continuation also guards that the dynamic `dev` session wires a `DiagnosticStoreLogSchedule` into the same `tick_frame` path, so Bevy-style time diagnostics can be emitted through `diagnostic_log` without widening the app/runtime ABI.
