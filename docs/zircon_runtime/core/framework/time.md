---
related_code:
  - zircon_runtime/src/core/time.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/real.rs
  - zircon_runtime/src/core/framework/time/virtual_clock.rs
  - zircon_runtime/src/core/framework/time/fixed.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - zircon_runtime/src/core/framework/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/real.rs
  - zircon_runtime/src/core/framework/time/virtual_clock.rs
  - zircon_runtime/src/core/framework/time/fixed.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - zircon_runtime/src/core/time.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - dev/bevy/crates/bevy_time/src/time.rs
  - dev/bevy/crates/bevy_time/src/virt.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/godot/core/os/main_loop.h
tests:
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/core/framework/tests.rs
  - cargo test -p zircon_runtime --lib time_framework --locked
  - cargo test -p zircon_runtime --lib framework_contract_types_are_constructible --locked
doc_type: module-detail
---

# Runtime Framework Time Contracts

## Purpose

`zircon_runtime::core::framework::time` is the neutral contract layer for Bevy-inspired runtime clocks. It gives runtime modules, app plugin groups, scene schedules, physics stepping, diagnostics, and editor tooling a common vocabulary for real time, virtual game time, and fixed timestep planning.

This was originally a lower-layer M4 foundation from the Bevy completion roadmap. The current runtime slice keeps those contracts here and adds `zircon_runtime::core::time` as the concrete owner that stores and advances one clock bundle per `CoreRuntime`. State scheduling, diagnostics emission, and app runner frame-loop consumption remain later milestones.

## Reference Evidence

Bevy `bevy_time` is the primary shape reference:

- `dev/bevy/crates/bevy_time/src/time.rs` defines the generic `Time<T>` clock with `delta`, `elapsed`, and context-specific clocks.
- `dev/bevy/crates/bevy_time/src/virt.rs` separates virtual game time from real wall-clock time, with pause, relative speed, and max delta clamping.
- `dev/bevy/crates/bevy_time/src/fixed.rs` models fixed timestep accumulation through a timestep and overstep accumulator.

Godot provides a secondary main-loop cross-check: `dev/godot/core/os/main_loop.h` separates variable `_process(delta)` from fixed `_physics_process(delta)`, reinforcing that fixed-step planning belongs in a reusable runtime contract instead of editor-only behavior.

Fyrox provides a Rust-engine cross-check through its engine, plugin, renderer, and UI contexts that carry elapsed time and delta as engine-owned values consumed by subsystems.

## Ownership Boundary

The time module lives under `zircon_runtime::core::framework` because it defines shared neutral data and helpers. It does not own process startup, frame pacing, rendering cadence, physics execution, or scene schedule dispatch. Those remain in `zircon_app`, `CoreRuntime`, scene systems, physics plugins, and render systems respectively.

The existing `FrameClock` remains available in `zircon_runtime::core` as a narrow wall-clock implementation. `CoreRuntime` now owns both a `FrameClock` and a `RuntimeTimeClocks` bundle, so callers can use deterministic `advance_time_by(...)` in tests/replay paths or wall-clock `tick_time(...)` in app loops while preserving the same `Time<Real>`, `Time<Virtual>`, and `Time<Fixed>` contract vocabulary.

## Data Model

The module is folder-backed so the root stays structural:

- `Time<T>` stores `delta`, `elapsed`, and `frame_index` for any clock marker.
- `Real` is a marker for wall-clock time that should not be paused or scaled.
- `Virtual` stores max-delta clamp, paused state, relative speed, and effective speed for game time.
- `Fixed` stores the timestep and overstep accumulator for deterministic fixed updates.
- `FixedStepPlan` reports how many fixed steps were drained, the timestep, consumed time, and remaining overstep.

`Duration` is used instead of raw seconds so contract values keep nanosecond precision and callers can choose their own `f32` or `f64` projections via the provided seconds helpers.

## Behavior

`Time<T>::advance_by(...)` advances a generic clock, records the current delta, accumulates elapsed time with saturating arithmetic, and increments the frame index.

`Time<Virtual>::advance_from_real_delta(...)` applies pause, relative speed, and max-delta clamping before advancing game time. A paused clock records a zero delta and does not accumulate elapsed virtual time.

`Time<Fixed>::drain_steps(max_steps)` consumes whole timestep chunks from the overstep accumulator, advances fixed time once per drained step, caps the number of steps to avoid spirals, and returns a `FixedStepPlan` for scheduler and diagnostics consumers. `RuntimeTimeClocks::advance_by(...)` feeds this accumulator from the current virtual delta, matching Bevy's rule that fixed time follows virtual time instead of raw wall-clock time.

## Intentional Divergence

Bevy wires time as ECS resources through `TimePlugin` and fixed-main schedules. Zircon's current foundation layer is not the ECS scheduler, so runtime integration stops at `CoreRuntime` clock ownership and fixed-step planning. Later scene/ECS and app-host milestones can consume the clocks from `CoreHandle` without changing the contract vocabulary.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers:

- construction through the existing framework contract smoke test,
- real clock delta/elapsed/frame-index advancement,
- virtual max-delta clamping, pause, relative speed, and effective speed,
- fixed timestep draining with max-step capping and retained overstep,
- root module structure so implementation stays in child files rather than `time/mod.rs`.

`zircon_runtime/src/tests/time.rs` covers the runtime-owned clock bundle, including real/virtual/fixed advancement, virtual pause, relative speed, max-delta clamp, and fixed-step planning from virtual time.

Milestone testing evidence is recorded in the active session note for `20260508-0455-bevy-time-foundation` and should be refreshed when app-host `FrameClock` migration begins.

Current validation status for this slice:

- direct rustfmt coverage for the Time files passed with child-module recursion disabled so sibling-owned framework modules were not pulled into the check,
- crate-level `cargo test -p zircon_runtime --lib time_framework --locked` is blocked before tests by active non-Time compile drift in asset, render, dynamic camera, and scene ECS/world lanes,
- an isolated temporary `rustc --edition 2021 --test` smoke harness against `zircon_runtime/src/core/framework/time/mod.rs` passed the same Time behavior and structural-root checks as implementation evidence until Cargo-level blockers clear.
