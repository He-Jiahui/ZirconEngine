# Runtime22 Virtual Delta and Clock-Domain Optimization Record

- Date: 2026-08-20
- Owner: `optimize-runtime22-virtual-delta-01a012f4-20260818`
- Source plan: `docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- Status: implementation and deterministic performance-gate definition complete; managed revalidation pending

## Problem

`WorldDriver` previously passed raw real time to every non-fixed scene system. Pausing or scaling
virtual time therefore did not govern gameplay callbacks. A paused frame could also drain fixed
overstep left by an earlier capped frame, allowing fixed callbacks and fixed-clock advancement to
leak through the pause. Diagnostic systems had no explicit way to opt into real time.

## Change

- `RuntimeTimeAdvance` now carries real delta, virtual delta, virtual pause state, and the fixed
  step plan as one authoritative handoff.
- Paused virtual time neither accumulates nor drains fixed overstep. Existing debt, fixed elapsed
  time, and fixed frame index remain unchanged until virtual time resumes.
- Native, builtin, and runtime scene systems default to the `Virtual` clock domain and do not run
  while virtual time is paused. Pending derived-state work is retained for the resumed frame.
- Systems explicitly registered in the `Real` domain continue with the unclamped real delta.
  Scaled virtual systems receive the scaled virtual delta.
- Runtime and plugin-SDK builders reject `Real` for `FixedFirst`, `FixedUpdate`, and
  `FixedPostUpdate`; fixed-loop stages remain governed only by the drained fixed-step plan.

## Deterministic Performance Evidence

| Paused-frame workload | Before | After | Reduction |
|---|---:|---:|---:|
| default virtual callbacks | scheduled with real/zero-like delta | `0` | `100%` |
| explicit real-time diagnostic callbacks | no clock-domain contract | `1` with `16 ms` real delta | semantic split |
| pre-existing fixed overstep | could drain while paused | preserved without elapsed/frame advance | `100%` leaked fixed steps removed |

The resumed scale case requires `16 ms` real delta to produce `8 ms` virtual delta at `0.5x`.
This is a deterministic work-removal and clock-correctness gate, not a wall-clock latency claim.
The exact output row is `PERF_RESULT runtime22_clock_domain` with
`paused_virtual_callbacks=0`, `paused_real_callbacks=1`,
`paused_virtual_work_reduction_percent=100`, `scaled_virtual_delta_ms=8`, and
`scaled_real_delta_ms=16`.

## Acceptance

- `core_runtime_virtual_pause_scale_and_clamp_feed_fixed_time`
- `core_runtime_virtual_pause_preserves_existing_fixed_overstep`
- `world_driver_pauses_virtual_systems_and_runs_explicit_real_time_systems`
- `runtime_registration_rejects_real_clock_domain_for_fixed_stages`
- The Runtime22 clock-domain child covers runtime tests, world-driver tests, public plugin-SDK
  registration defaults, typed fixed-stage rejection, and the performance row above.
- Runtime22 and Runtime01 share the single five-Cargo-group parent validator
  `zircon-validation-runtime01-runtime22-batch.ps1`, SHA-256
  `BDFA5754B2C85973AECBAD3A5039D4E23EBFEDB7889E3C8EFBECAF6BFC2FE9B6`.
- The earlier ticket `c3b54e583afb476e980e529d5a6b47e7` compiled against a stale Main baseline
  and is not acceptance evidence. The combined batch will be rematerialized from post-Main HEAD.

## Remaining Scope

This record closes pause/scale clock routing and fixed-debt preservation. Replay serialization,
deterministic RNG ownership, timer scheduling, cross-world time domains, long-run drift, and the
rest of Runtime22 remain open under the parent plan.
