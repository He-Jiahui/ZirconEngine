---
title: Runtime09H2 Inline Volume Defaults
category: zircon_runtime
report_id: Runtime09H2-inline-volume-defaults-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-inline-default-values-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 inline Volume defaults

## Scope

- Parent scope: the Runtime09H2 Volume evaluation CPU path, specifically resetting registered descriptors into `RenderResolvedPostProcessSettings` defaults.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: `VolumeComponentDescriptor::apply_defaults`, its focused source and Rust contracts, the standalone allocation/timing model, and this record.
- This slice removes transient default-value vectors for the 15 builtins only. It does not close versioned profile persistence, unknown plugin payloads, unsupported shapes, overlay ownership, resource readiness, GPU effects, or the remaining Runtime09H2 acceptance gates.

## Change

- A named `BUILTIN_VOLUME_PARAM_INLINE_CAPACITY` fixes the builtin inline budget at the current maximum of nine parameters, owned by the Exposure descriptor.
- `apply_defaults` fills a fixed stack array and passes only the descriptor-length slice when its parameter count fits the builtin budget.
- Descriptors with more than nine parameters retain the existing `default_values` vector fallback, so plugin extensibility and the public owned-values API are unchanged.
- A direct Rust contract proves every registered builtin fits the inline capacity.
- A 12-parameter plugin contract proves the fallback forwards all defaults in order without truncation.

`VolumeComponentRegistry::default_resolved_post_process_settings` applies all 15 builtin descriptors. Before this change that reset allocated one vector per descriptor; the builtin path now performs no default-value heap allocations.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_inline_volume_defaults_performance_contract -v` initially failed 4/4 because the capacity, stack path, long-plugin fallback branch, and direct Rust contracts were absent.
- GREEN: the same source contract passes 4/4 after the inline path and fallback contracts are implemented.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/inline sample pairs, with 65,536 complete 15-descriptor default applications per sample. Its builtin parameter counts are `[7, 7, 2, 3, 9, 6, 3, 5, 3, 3, 2, 2, 2, 3, 1]`. Every pair must produce rolling checksum `17186103771395915776`; a separate 12-parameter plugin fallback must produce checksum `16226415085948753238` with one allocation. Four local runs passed the acceptance thresholds; the table records the final run.

| Metric | Per-descriptor default Vec | Nine-slot builtin stack buffer | Change |
|---|---:|---:|---:|
| P50 | 79.7404 ms | 15.6742 ms | -80.343% |
| P95 | 171.5433 ms | 25.6105 ms | -85.071% |
| allocations / 15-builtin reset | 15 | 0 | -100.000% |

The other three runs produced P50 reductions of 80.014%, 82.105%, and 83.555%, P95 reductions of 77.912%, 84.566%, and 85.354%, and the same 100% builtin allocation reduction. These timings isolate CPU default-value materialization and application; they do not claim complete Volume evaluation or frame time.

## Async validation

One coordinator batch must run the four Python source contracts, all six focused `render_volume_component` Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 6/6 Rust tests to pass, exact builtin and plugin checksums, zero allocations for the 15-builtin reset, exactly one allocation for the long-plugin fallback, and P50/P95 reductions of at least 70%. The Cargo validation remains required even while the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` prevents workspace compile-time input closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as CPU default-materialization evidence for one complete 15-builtin reset.
