# Runtime22 Outer-Frame Snapshot Consumption Plan

- Date: 2026-08-29
- Session: `root-runtime22-checkpoint-atomicity-20260829`
- Parent: `22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- Slice: M3 `World/system clock domains`
- Status: RED regressions and World-side implementation complete; Core transfer and managed
  validation pending

## Proven Defect

`FrameTimeSnapshot` is a public, copyable handoff. A caller may legally advance the Core clock for
frame N, advance it again for frame N+1, and then tick a Level with the saved frame-N snapshot.
`WorldDriver::tick_level` currently combines frame N's outer index and real delta with a fresh
`core.real_time()` read from frame N+1. One `SystemTickContext` can therefore describe a clock state
that never existed.

`WorldTimeController::advance` also accepts the same snapshot more than once or accepts an older
outer-frame index after a newer one. Those inputs mutate virtual elapsed time and fixed debt again,
so duplicated or out-of-order delivery can silently duplicate simulation work.

## Required Invariants

- `FrameTimeSnapshot` captures real elapsed time, delta, outer-frame index, discontinuity, and the
  monotonic-real domain stamp under the same Core time-authority lock.
- A Level derives every real-time `SystemTickContext` field from that immutable snapshot. No Level
  tick may reread the mutable Core clock to complete old evidence.
- Each Level tracks its last consumed outer-frame index. After its first accepted snapshot, only the
  strict successor is legal unless the new snapshot carries an explicit lifecycle discontinuity.
  Duplicate, decreasing, and unexplained skipped indices reject before virtual elapsed, fixed debt,
  policy generation, or schedule state changes.
- Consumption state is Level-local. Two Levels may independently consume the same outer snapshot
  exactly once.
- Rejection is typed and remains visible through `LevelTickError` and the dynamic-session boundary.

## Architecture

1. Extend the Core-owned `FrameTimeSnapshot` with the captured monotonic-real elapsed value.
2. Extend `WorldTimeSnapshot` with the same immutable value and use it when building the real-time
   system tick context.
3. Replace `WorldTimeController::advance` with a fallible consume operation guarded by a typed
   `WorldTimeAdvanceError`. Validate the outer-frame index before any clock mutation, then publish
   the accepted index only with the derived snapshot.
4. Project the typed rejection through `LevelSystem`, `WorldDriver`, `LevelTickError`, and the
   dynamic runtime error without a compatibility overload or fallback to the latest Core clock.

## RED-GREEN Regression

- Delayed delivery: capture N, advance Core to N+1, tick with N, and assert that outer index, real
  delta, real elapsed, and clock stamp all remain the frame-N tuple.
- Duplicate delivery: consume N twice and assert typed rejection plus unchanged World virtual time,
  fixed debt, fixed frame index, and callback count.
- Out-of-order delivery: consume N+1 then N and assert the same no-mutation contract.
- Skipped delivery: consume N then submit N+2 without discontinuity and assert typed rejection; a
  lifecycle-rebased successor may cross that gap explicitly.
- Multi-World delivery: two Levels consume N once each and retain independent consumption state.
- Upward projection: dynamic-session error tests preserve the exact typed clock-consumption receipt.

## Performance Gate

The production path must remove the `core.real_time()` mutex acquisition from every Level tick.
Release-only paired evidence will compare the old mutable-Core projection with immutable snapshot
projection at 1, 64, and 1024 Level contexts, using 10 warmups and 31 measured samples. Report P50
and P95, require exact output counts, and require the 1024-context snapshot P95 to be no greater
than 75% of the legacy mutex-backed P95. This is bounded local evidence, not an end-to-end frame
latency or cross-platform determinism claim.

The exact shared performance batch is `cargo +1.94.1 test -p zircon_runtime --lib --release
--locked --jobs 1 --no-default-features --features core-min runtime22_performance_ --
--include-ignored --nocapture --test-threads=1`.

## Exact Source Boundary

- Core snapshot owner: `zircon_runtime/src/core/runtime/time.rs`
- World clock owners: `zircon_runtime/src/scene/world_time/controller.rs` and
  `zircon_runtime/src/scene/world_time/snapshot.rs`
- Tick/error projection: `zircon_runtime/src/scene/level_system.rs`,
  `zircon_runtime/src/scene/module/world_driver.rs`, and
  `zircon_runtime/src/scene/fixed_step_failure.rs`
- Focused regressions: World-time controller, World-driver clock-domain tests, and dynamic runtime
  error projection tests

`core/runtime/time.rs` is currently in the Frameworks01 executable scope. Runtime22 must obtain an
exact coordinator transfer before editing it. Untracked World-time files must be claimed and
attributed explicitly; directory scope alone is not validation evidence.

## Current Progress

- Coordinator transfer `a305fb6c2cd44d4898951b430a8fc5d6` moved the three World-time owners and
  the World-driver regression owner into Runtime22; transfer
  `5794186e7f534e67b7450438f30a27ec` moved the dedicated World-time test owner.
- Duplicate, out-of-order, skipped-without-discontinuity, and explicit-discontinuity delivery
  regressions are written. The delayed immutable-tuple and independent multi-World regressions,
  dynamic typed-error projection, and release-only paired performance gate are also written.
- `WorldTimeController` now validates consumption before mutation, and `WorldDriver` projects real
  tick context plus ECS diagnostic frame identity from `WorldTimeSnapshot` without
  `core.real_time()`.
- Frameworks01 received exact transfer request `fe299db51db84157ac532e7a0da5e809`
  for `core/runtime/time.rs`; Runtime22 has not edited that foreign source.
- Managed Rust evidence remains pending. No test count or timing result is claimed by this record.

## Forbidden Workarounds

- Do not clamp, sort, or silently ignore duplicate/out-of-order snapshots.
- Do not reconstruct old real elapsed time from delta or the current Core clock.
- Do not add a second tick overload that accepts raw duration or latest-time fallback.
- Do not weaken fixed-step or virtual-delta acceptance while the M2 validation copy is pending.
