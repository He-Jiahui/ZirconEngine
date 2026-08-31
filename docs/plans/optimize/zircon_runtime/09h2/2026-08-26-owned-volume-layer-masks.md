---
title: Runtime09H2 Owned Volume Layer Masks
category: zircon_runtime
report_id: Runtime09H2-owned-volume-layer-masks-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-owned-volume-layer-masks-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 owned Volume layer masks

## Scope

- Parent scope: the Runtime09H2 scene-to-render Volume extraction path, specifically transferring a volume's `RenderLayerSet` to post-process and local-fog outputs.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: `World::collect_post_process_volumes`, `fog_volume_from_extract`, their focused source and Rust contracts, the standalone allocation/timing model, and this record.
- This slice removes redundant layer-mask ownership copies. It does not close versioned profile persistence, unknown plugin payloads, unsupported shapes, overlay ownership, resource readiness, GPU effects, or the remaining Runtime09H2 acceptance gates.

## Change

- The newly constructed `RenderLayerSet` is moved directly into `PostProcessVolumeExtract`; the extraction call no longer clones it unconditionally.
- Post-process-only volumes retain that owned mask with no copy.
- Local-fog-only volumes move the mask out of the temporary extract and leave an empty set behind; the extract is then discarded.
- Volumes consumed by both post-process and local fog perform the one required clone so both persistent outputs keep independent ownership.
- `fog_volume_from_extract` now consumes the caller-selected layer mask instead of cloning from the extract internally.
- A direct Rust contract proves the helper uses the supplied mask while leaving the extract's distinct mask unchanged.

For a modeled collection of 256 local volumes, the old path allocated twice for 128 post-only volumes and three times for each of 64 fog-only and 64 dual-output volumes. The owned path allocates once for post-only and fog-only volumes and twice only for dual-output volumes, reducing layer-mask allocations from 640 to 320.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_owned_volume_layer_masks_performance_contract -v` initially failed 4/4 because the entry move, conditional dual-output clone, consuming fog-helper parameter, and direct Rust contract were absent.
- GREEN: the same focused source contract passes 4/4 after the ownership transfer and fallback contracts are implemented.
- A local batch of every Runtime09H2 performance source contract passes 36/36.
- `rustfmt --edition 2021 --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/owned sample pairs, with 4,096 collections of 256 volumes per sample. Each collection contains 128 post-only, 64 fog-only, and 64 dual-output volumes. Every pair must produce rolling checksum `14268945649176395776`. Four local runs passed the acceptance thresholds; the table records the final run.

| Metric | Unconditional layer-mask clones | Owned output transfer | Change |
|---|---:|---:|---:|
| P50 | 173.0447 ms | 87.6872 ms | -49.327% |
| P95 | 239.7466 ms | 151.3577 ms | -36.868% |
| layer-mask allocations / collection | 640 | 320 | -50.000% |

The other three runs produced P50 reductions of 47.813%, 52.773%, and 46.249%, P95 reductions of 51.853%, 51.060%, and 42.353%, and the same 50% allocation reduction. These timings isolate CPU layer-mask ownership transfer and do not claim complete Volume evaluation or frame time.

## Async validation

One coordinator batch must run the four focused Python source contracts, the direct helper Rust contract, all 14 scene post-process extraction Rust tests, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 15/15 Rust tests to pass, checksum `14268945649176395776`, exactly 320 owned-path allocations versus 640 legacy allocations, a 50% allocation reduction, and P50/P95 reductions of at least 35%. The Cargo validation remains required even while the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` prevents workspace compile-time input closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as CPU layer-mask ownership evidence for one 256-volume collection.
