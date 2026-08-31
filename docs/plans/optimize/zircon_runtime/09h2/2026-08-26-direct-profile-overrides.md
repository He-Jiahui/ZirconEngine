---
title: Runtime09H2 Direct Profile Override Projection
category: zircon_runtime
report_id: Runtime09H2-direct-profile-overrides-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-direct-profile-overrides-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 direct profile override projection

## Scope

- Parent scope: the Runtime09H2 Volume extraction CPU path, specifically fixed-profile projection into typed component overrides.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: `VolumeComponentOverride::from_profile`, its effect-stack builders, source/performance contract, focused Rust tests, and this record.
- This slice removes transient projection allocations only. It does not close versioned profile persistence, unknown plugin payloads, unsupported shapes, overlay ownership, resource readiness, GPU effects, or the remaining Runtime09H2 acceptance gates.

## Change

- `from_profile` derives the exact outer override count from the four optional profile groups and preallocates the result vector once.
- The 11 effect-stack builders now construct final `VolumeComponentOverride` values directly from fixed arrays.
- The former intermediate `EffectStackOverride { values: Vec<VolumeParamValue> }` and its second `into_override` collection are removed.
- A named `EFFECT_STACK_PROFILE_OVERRIDE_COUNT` is guarded against the 11 direct pushes so future profile expansion cannot silently reintroduce outer-vector growth.
- A focused Rust test freezes the existing component order from Bloom and Color Grading through all 11 effect-stack overrides.

For a complete 14-component profile, the implementation retains one final values vector per component and one outer vector. It removes all 11 effect-stack staging vectors plus outer growth reallocations.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_direct_profile_overrides_performance_contract -v` initially failed 4/4 because the outer vector was not reserved, effect builders staged owned values, the count was implicit, and no complete order contract existed.
- GREEN: the source contract passes 4/4 after direct projection and the stable-order Rust test are implemented.
- One intermediate false negative looked for the count constant in `from_profile` instead of its dedicated `profile_override_count` helper; the guard was corrected without changing the implementation.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and `git diff --check` pass for the owned files.
- The standalone model is compiled with `rustc +1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/direct sample pairs, with 16,384 complete 14-override projections per sample. It covers 49 typed values, exact override count and rolling checksum parity, and a process-local allocation counter. Three final local runs passed the acceptance thresholds; the table records the latest run.

| Metric | Staged effect values + growing outer Vec | Direct final values + reserved outer Vec | Change |
|---|---:|---:|---:|
| P50 | 56.4824 ms | 31.0569 ms | -45.015% |
| P95 | 83.8548 ms | 73.3280 ms | -12.554% |
| allocations / complete profile | 28 | 15 | -46.429% |

The other two final runs produced P50 reductions of 41.508% and 40.741%, P95 reductions of 57.340% and 58.783%, and the same 46.429% allocation reduction. These timings isolate CPU profile-to-override projection; they do not claim complete Volume evaluation or frame time.

## Async validation

One coordinator batch must run the four Python source contracts, all four focused Volume extract Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 4/4 Rust tests to pass, exact rolling checksum parity, allocation reduction of at least 40%, P50 reduction of at least 30%, and P95 reduction of at least 10%. The Cargo validation remains required even while a foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` blocks workspace closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as complete 14-component CPU profile-projection evidence.
