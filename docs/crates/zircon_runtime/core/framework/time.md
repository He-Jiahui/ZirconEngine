---
related_code:
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/time/product_policy.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/domain/mod.rs
  - zircon_runtime/src/core/framework/time/monotonic_real.rs
  - zircon_runtime/src/core/framework/time/virtual_clock.rs
  - zircon_runtime/src/core/framework/time/fixed.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - zircon_runtime/src/core/framework/time/policy.rs
  - zircon_runtime/src/core/framework/time/product_policy.rs
  - zircon_runtime/src/core/framework/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/domain/mod.rs
  - zircon_runtime/src/core/framework/time/monotonic_real.rs
  - zircon_runtime/src/core/framework/time/virtual_clock.rs
  - zircon_runtime/src/core/framework/time/fixed.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - zircon_runtime/src/core/framework/time/policy.rs
  - zircon_runtime/src/core/framework/time/product_policy.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/time/product_policy.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - dev/bevy/crates/bevy_time/src/time.rs
  - dev/bevy/crates/bevy_time/src/virt.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/App.h
  - dev/godot/core/os/main_loop.h
tests:
  - tools/tests/test_frameworks_01_time_product_policy_owner_boundary.py
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/core/framework/tests.rs
  - cargo test -p zircon_runtime --lib time_framework --locked
  - cargo test -p zircon_runtime --lib framework_contract_types_are_constructible --locked
doc_type: module-detail
---

# Runtime Framework Time Contracts

## Purpose

`zircon_runtime::core::framework::time` is the neutral contract layer for Bevy-inspired runtime clocks. It gives runtime modules, app plugin groups, scene schedules, physics stepping, diagnostics, and editor tooling a common vocabulary for real time, virtual game time, and fixed timestep planning.

This was originally a lower-layer M4 foundation from the Bevy completion roadmap. The current runtime slice keeps those contracts here; `CoreRuntime` owns only the outer real-frame handoff and default World policy, while `LevelSystem` owns derived simulation clocks. State scheduling, World fixed telemetry, and app runner frame-loop consumption remain separate milestones.

## Reference Evidence

Bevy `bevy_time` is the primary shape reference:

- `dev/bevy/crates/bevy_time/src/time.rs` defines the generic `Time<T>` clock with `delta`, `elapsed`, and context-specific clocks.
- `dev/bevy/crates/bevy_time/src/virt.rs` separates virtual game time from real wall-clock time, with pause, relative speed, and max delta clamping.
- `dev/bevy/crates/bevy_time/src/fixed.rs` models fixed timestep accumulation through a timestep and overstep accumulator.

Godot provides a secondary main-loop cross-check: `dev/godot/core/os/main_loop.h` separates variable `_process(delta)` from fixed `_physics_process(delta)`, reinforcing that fixed-step planning belongs in a reusable runtime contract instead of editor-only behavior.

Fyrox provides a Rust-engine cross-check through its engine, plugin, renderer, and UI contexts that carry elapsed time and delta as engine-owned values consumed by subsystems.

## Ownership Boundary

The time module lives under `zircon_runtime::core::framework` because it defines shared neutral data and invariant-preserving DTO methods. It does not own product preset selection, policy hashing, process startup, frame pacing, rendering cadence, physics execution, or scene schedule dispatch. Product preset and digest behavior belongs to `core::runtime::time`; the remaining owners stay in `zircon_app`, `CoreRuntime`, scene systems, physics plugins, and render systems respectively.

The existing `FrameClock` remains available in `zircon_runtime::core` as a narrow monotonic frame-delta implementation. `CoreRuntime` owns that outer real clock and a default policy for subsequently created Levels; callers can use deterministic `advance_time_by(...)` in tests/replay paths or `tick_time(...)` in app loops. `LevelSystem` owns the `Time<Virtual>` and `Time<Fixed>` instances that derive from each frame handoff. Platform lifecycle signals do not mutate framework clocks directly: the core frame authority accepts typed discontinuities and records its rebase policy in the next frame receipt.

## Data Model

The module is folder-backed so the root stays structural:

- `ClockDomainRegistry` is a zero-allocation, versioned taxonomy for monotonic real, UTC, world virtual/fixed, input, render, audio, network, media, and editor-preview domains. `ClockDomainDescriptor` makes the domain unit explicit.
- `Time<T>` requires a `ClockDomainMarker` and stores `delta`, `elapsed`, `frame_index`, plus a `ClockDomainStamp` carrying canonical domain, unit, epoch, and frame-source generation.
- `MonotonicReal` is a marker for non-paused, non-scaled frame-source time. UTC/calendar wall-clock time is not represented by this contract.
- `Virtual` stores max-delta clamp, paused state, relative speed, and effective speed for game time.
- `Fixed` stores the timestep and overstep accumulator for deterministic fixed updates.
- `FixedStepPlan` reports how many fixed steps a non-mutating proposal admits, the timestep, proposed consumption, and remaining debt. It exposes full debt duration, whole-step count, and an unbounded timestep ratio for scheduling/health telemetry; `interpolation_fraction()` exposes only the fractional remainder between adjacent fixed states.
- `TimePolicy` validates neutral virtual/fixed clock values before a default-policy or Level-local policy transaction mutates them.
- `ProductTimePolicy` is the versioned DTO for a `client`/`headless`/`editor`/`test` role, neutral time policy, and bounded fixed-step execution budget. It validates schema and field invariants but does not choose product defaults or hash itself.
- `core::runtime::time::ProductTimePolicies` maps a product role to the current runtime preset. `ProductTimePolicyDigest::from_policy(...)` is the sole canonical BLAKE3 implementation used by later BuildSet, replay, and diagnostics admission.

`Duration` is used instead of raw seconds so contract values keep nanosecond precision and callers can choose their own `f32` or `f64` projections via the provided seconds helpers.

## Behavior

`Time<T>` is a public observation contract. Clock mutation stays crate-internal: the outer runtime advances monotonic real time, and the owning `LevelSystem` advances virtual time and commits fixed time. Downstream crates cannot call `context_mut`, `advance_by`, virtual advancement, pause/resume, fixed-debt accumulation, or batch fixed-step draining on these observations; pause/resume enters through the owning runtime/Level policy transaction.

`ClockDomainStamp` is value-owned by its `Time<T>` rather than looked up through a global clock service. A committed virtual/fixed `TimePolicy` rolls the affected derived-domain epoch; a `FrameClock` rebase updates the common source generation before the resulting frame snapshot is published. This makes cross-system comparisons reject stale policy or source provenance without adding per-frame allocation or locking.

The internal virtual-clock advancement applies pause, relative speed, and max-delta clamping before advancing game time. A paused clock records a zero delta and does not accumulate elapsed virtual time.

`Time<Fixed>::plan_steps(max_steps)` observes bounded work without mutation; `try_commit_step()` consumes one timestep only after the owning Level's fixed transaction succeeds. The Level controller feeds this accumulator from its local virtual delta, matching Bevy's rule that fixed time follows virtual time instead of raw wall-clock time while preserving Zircon's commit boundary.

`ProductTimePolicy::time_policy_transaction()` first validates its schema version, its non-zero fixed-step budget, and the contained `TimePolicy`. A product host then submits that transaction through `CoreRuntime::apply_time_policy(...)`; the profile selection itself does not receive direct mutable clock setters. The `Headless` discriminant remains `2`, preserving the versioned policy digest identity while keeping non-network execution distinct from a server role.

Product hosts obtain defaults through `ProductTimePolicies::for_profile(...)` and compute a stable identity through `ProductTimePolicyDigest::from_policy(...)`. The former `ProductTimePolicy::{client,headless,editor,test,digest}` inherent behavior does not exist, and framework time contains no BLAKE3 dependency.

## Intentional Divergence

Bevy wires time as ECS resources through `TimePlugin` and fixed-main schedules. Zircon's core integration stops at outer real-frame ownership; `LevelSystem` owns virtual/fixed planning and fixed-step commit. App-host code consumes `FrameTimeSnapshot`, and scene/ECS code consumes the resulting World-local context without changing the contract vocabulary.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers:

- construction through the existing framework contract smoke test,
- engine-owned real clock delta/elapsed/frame-index advancement,
- virtual max-delta clamping, pause, relative speed, and effective speed,
- bounded fixed-step proposal arithmetic with max-step capping and retained debt,
- root module structure so implementation stays in child files rather than `time/mod.rs`.

`zircon_runtime/src/tests/time.rs` covers outer real-time advancement, budget/discontinuity handoff, default-policy transactions, injected sources, and real-frame diagnostics. `zircon_runtime/src/scene/tests/ecs_schedule` covers World-local virtual/fixed advancement, pause, scale, debt, and fixed-step transactions.

`tools/tests/test_frameworks_01_time_product_policy_owner_boundary.py` locks the DTO/kernel split, unique runtime preset/digest owner, curated facade origin, read-only production clock surface, absence of batch fixed-step draining in product source, and absence of deleted inherent preset calls.
