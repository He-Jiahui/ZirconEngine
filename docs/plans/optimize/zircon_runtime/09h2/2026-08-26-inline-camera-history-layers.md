---
title: Runtime09H2 Inline Camera History Layers
category: zircon_runtime
report_id: Runtime09H2-inline-camera-history-layers-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-inline-history-layers-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 inline camera history layers

## Scope

- Parent scope: the Runtime09H2 per-camera temporal history identity, specifically projecting culling and Volume layer sets into `ViewportCameraHistoryKey` on every frame submission.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`; source blob `9dd0c15c0925f4e8b1b3dd63fe1d267980e1978a`.
- Owners: `ViewportCameraHistoryLayerKey`, its focused source and Rust contracts, the standalone allocation/timing model, and this record.
- This slice preserves exact ordered layer identity, wide layer indices, key equality/hash behavior, and shared storage for wide sets. It does not close exposure-specific history invalidation, real frame delta time, output transfer correctness, GPU effects, or the remaining Runtime09H2 acceptance gates.

## Change

- Layer keys with up to four entries now store their ordered `RenderLayer` values in a four-slot array plus an explicit length.
- Empty sets and a set containing layer zero remain distinct because equality and hashing include the explicit length.
- Layer indices above the scene schema v1 mask width remain exact `u32` values; no lossy mask conversion is introduced.
- Sets with more than four entries retain an `Arc<[RenderLayer]>` fallback, and cloned history keys continue to share that backing storage.
- Direct Rust contracts prove the common three-layer path is inline and that two five-layer fallback keys share storage after cloning.

`ViewportCameraHistoryKey::from_camera` creates separate culling and Volume layer keys each frame. Typical one-to-three-layer cameras now perform no layer-key heap allocation; unusual sets beyond four layers retain the previous shared-slice behavior.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_inline_history_layers_performance_contract -v` initially failed 4/4 because the inline representation, allocation-free common conversion, overflow-only fallback, and direct Rust contracts were absent.
- GREEN: the same focused source contract passes 4/4 after the inline representation is implemented.
- A local batch of every Runtime09H2 performance source contract passes 44/44.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating shared-slice/inline sample pairs. Each sample constructs 65,536 camera history keys with a two-layer culling key and a one-layer Volume key. Every pair produces checksum `11562718363690483712` for both representations. Four sequential local runs passed the acceptance thresholds; the table records the final run.

| Metric | Two shared slices | Four-slot inline keys | Change |
|---|---:|---:|---:|
| P50 | 10.2326 ms | 0.2823 ms | -97.241% |
| P95 | 16.5971 ms | 0.8265 ms | -95.020% |
| allocations / 65,536 camera keys | 131,072 | 0 | -100.000% |

The other three runs produced P50 reductions of 97.251%, 96.954%, and 96.870%, P95 reductions of 99.101%, 97.881%, and 99.113%, and the same complete allocation elimination. These timings isolate CPU history-layer-key materialization and do not claim complete history validation or frame time.

## Async validation

One coordinator batch must run the four focused Python source contracts, all six camera-history-key Rust tests in one Cargo filter, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 6/6 Rust tests to pass, identical checksum `11562718363690483712` for both representations, zero common-path allocations versus `131,072` legacy allocations, and P50/P95 reductions of at least 90%. The Cargo validation remains required even while the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` prevents workspace compile-time input closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as CPU history-layer-key evidence for 65,536 camera keys.
