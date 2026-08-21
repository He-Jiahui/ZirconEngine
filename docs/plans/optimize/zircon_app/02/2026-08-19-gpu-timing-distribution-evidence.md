# App02 GPU Timing Distribution Evidence

- Date: 2026-08-19
- Session: `app02-gpu-timing-distribution-r1-01a00797-20260819`
- Plan finding: `PBR-P1-23`
- Status: passed

## Scope

The opt-in PBR viewer GPU report no longer treats the first Ready screenshot
frame as a benchmark result. The existing asynchronous WGPU timestamp ring is
retained; the viewer now discards the screenshot frame, consumes five
consecutive post-Ready warmup reports, then retains 31 consecutive measured
reports. The normal viewer path remains unchanged when
`--gpu-timing-report` is absent.

The evidence schema is hard-cut from
`zircon_shader_pbr_viewer_gpu_timing_evidence_v1` to `v2`. The Rust producer,
standalone Python validator, and cold/warm profile summarizer move together.

## Contract

- Timestamp period is projected from the existing WGPU queue calibration as
  raw `f32` bits, nanoseconds per tick, and derived tick frequency.
- Every accepted report must have the next frame generation, the same
  timestamp period, non-empty unique pass names, and identical pass coverage.
- `Deferred`, capacity exhaustion, generation gaps, pass drift, invalid
  calibration, missing samples, and timeout fail closed.
- All 31 samples are retained. No outlier is removed.
- The producer reports nearest-rank `min`, `median`, `p95`, and `max` for total
  GPU time and every pass.
- The validator recomputes every aggregate from the raw samples and rejects a
  mismatch before the profile summarizer can consume it.
- The validator requires the exact field set implied by the declared pass
  coverage and 31 sample ordinals; extra samples or undeclared pass aggregates
  cannot be silently ignored.
- The profile summarizer now aggregates each run's stable-frame pass median,
  rather than aggregating first-Ready single-frame values.

## Deterministic Delta

| Metric | Before | After | Delta |
|---|---:|---:|---:|
| post-Ready warmup reports | 0 | 5 | +5 |
| retained measured reports per run | 1 | 31 | +30 / 31x evidence |
| distribution statistics per total/pass | 1 raw value | min/median/p95/max + 31 raw values | auditable distribution |
| explicit queue timestamp calibration | 0 fields | bits + period + frequency | fail-closed calibration |
| silent generation/pass discontinuity acceptance | possible | 0 | eliminated by exact continuity gate |
| extra frames without `--gpu-timing-report` | 0 | 0 | unchanged |

The additional 36 post-Ready reports are diagnostic work requested only by the
GPU timing option. This milestone improves measurement correctness; it does not
claim the renderer became faster.

## Validation

Validation evidence:

1. `rustfmt` passed for all five changed Rust files.
2. The GPU evidence validator and profile summarizer suites passed together:
   30 tests, 30 passed, 0 failed in 16.966 seconds. The added regression proves
   that `sample.031` and an aggregate outside `pass_coverage` both fail closed.
3. Coordinator run `ab871833fede4d098f70adca29083cdb` reached Cargo. The
   focused `zr_rhi_wgpu` timer group passed: 5 tests, 5 passed, 0 failed.
   The next `zircon_runtime` GPU timing command stopped during the shared lib
   test harness build with 54 pre-convergence errors, so the PBR viewer and
   Python consumer stages did not run. This receipt contributes no App02
   runtime or GPU latency result and is not attributed to the timing change.
4. The first ordinary validation copy `7b45c862ebe34f159172265a2fcc7152` was
   rejected before Cargo because it did not contain a workspace `Cargo.toml`;
   its run `4686ffc7196d4fa88eb817cdf8673e11` is retained as setup evidence
   only. The replacement Cargo-closure copy is `af0eec31c70f4706a6e1923d74e860b4`.
5. Current-source run `4dbde46954f94d67a546e9b39131b9fd` compiled the
   complete viewer test harness and ran 128 tests. It passed 127 and exposed
   one stale source-shape assertion in
   `screenshot_export_writes_versioned_ibl_provenance`: production still
   emitted `pipeline_creation()`, but Rustfmt had wrapped the receiver onto the
   preceding line. The assertion now matches the stable
   `.pipeline_creation(),` fragment, consistent with the adjacent pipeline
   readiness and shader-source assertions. No production behavior changed;
   the corrected current-source batch is pending.
6. Main continuation copy `bad081f77bc3478cadfe1fe5dd218d8b` compiled the
   real-adapter viewer after its WGPU timer, runtime artifact, 128 viewer, and
   30 Python consumer tests passed. The viewer then failed before Ready with
   `render graph pass 'uber' reads resource 'bloom-texture' before any producer
   writes it`. The first repair synchronized an already-authored post-process
   stack with Bloom feature degradation and removed the disabled effect's output
   and ordering dependencies. Focused tests locked that stack-bearing path.
7. Replacement copy `5f3a484e53174172be7da95f47241c53` again passed the
   WGPU timer `5/5`, Runtime GPU/artifact `2/2`, viewer `128/128`, and Python
   consumer `30/30` groups before the real adapter exposed the same graph error.
   The remaining path was the bootstrap compile before any post-process stack is
   available: `filter_no_stack_post_process_resources` retained the optional
   Bloom read on `uber` even when compile options had already disabled the Bloom
   feature and its producer. The compiler now removes that input according to
   the feature gate, and a no-stack Forward+ regression locks successful graph
   compilation with no Bloom producer. Rust 1.94.1 formatting and scoped diff
   checking pass; the repaired managed batch and real GPU distribution remain
   pending.
8. Repaired closure copy `98b3a87c7d1c4f338e17d203e636aad3` passed the selected
   Runtime artifact regressions, all 128 viewer tests, and all 30 Python
   consumer tests. The real adapter still failed before `ready.png` with the
   same unproduced `bloom-texture`, so this copy has no accepted GPU result.
   The no-stack repair used `RenderPipelineCompileOptions::permits_feature`,
   which only described policy permission and did not prove the filtered
   renderer still contained a Bloom provider. The real frame path carried a
   post-process stack with Bloom resources while profile filtering had removed
   the Bloom feature. Compilation now passes the actual enabled renderer
   feature set into descriptor filtering and removes Bloom resources whenever
   that set lacks the provider. Regressions cover both explicit compile-option
   removal and a pipeline asset with no Bloom feature. A new immutable copy is
   required; the terminal removed copy is not reused.
9. Replacement copy `39ce22ca381b43b3bbcd38b30b1c3847`, run
   `498e9a342ea8469e98ad1d6f136c4508`, passed WGPU timer `5/5`, selected
   Runtime artifact `2/2`, viewer `128/128`, and Python consumer `30/30` before
   the real adapter reproduced the same `uber` / `bloom-texture` failure. This
   proved the remaining defect was not a missing-provider filter. When Bloom
   was active, its producer and consumer were in the same stage, but the
   consumer read had no `RenderFeatureResourceVersion`; feature registration
   order could therefore author `uber` before `bloom-extract`. Descriptor
   filtering now binds active Bloom reads explicitly to `bloom-extract` in
   both stack and no-stack paths. Regressions require the compiled dependency
   and pass order, and budget degradation is reapplied after the runtime
   rebuilds the post-process stack. Rustfmt and scoped diff checks pass. The
   next immutable copy must still produce the real screenshot and 31-sample
   GPU distribution before this record can be accepted.
10. Main continuation copy `7cfabbf9098f4412988d0d837ec99cd7` passed the
    post-process group `19/19`, all four App02 Rust groups including viewer
    `128/128`, and both Python consumer groups `9/9` plus `21/21`. The real
    adapter initialized successfully and the executable returned normally, but
    no screenshot was produced because the migrated `WgpuRenderFramework`
    always selected `render_frame_with_pipeline...`. The environment-only PBR
    startup profile deliberately owns only direct-render resources and rejected
    that compiled scene-graph execution. The framework now queries the
    renderer profile and selects the existing direct renderer for extract,
    runtime-frame, and surface-present submissions while preserving the latest
    target for the framework capture API. A source regression requires all
    three submission shapes to retain both the compiled path and the direct
    profile route. This failed receipt contains no accepted GPU distribution;
    the repair still requires a fresh immutable grouped run.

Actual GPU P50/P95 values require a fresh current-source viewer run on a named
adapter. They must not be inferred from the synthetic unit-test fixture.

A post-implementation source recheck found that the screenshot frame armed one
timestamp request, while redraws made for the remaining warmup/measured frames
did not arm another request. The 5 + 31 distribution would therefore exhaust
its finite resolve budget instead of completing on a real adapter. Production
now snapshots `gpu_timing_evidence_pending()` before borrowing the scene and
arms timing on every pending redraw. This avoids a whole-`self` borrow while the
scene field is mutably borrowed and leaves ordinary redraws unchanged when the
option is absent. The source contract fixes the snapshot-before-scene ordering.
No GPU result is claimed until a real viewer run passes.

The focused batch will be rerun after the mainline compile-convergence commit;
the rerun must include the same three Rust groups and 30 Python tests before
this record can be completed. It is grouped with Runtime02 and Runtime06 in the
single follow-up script `zircon-validation-optimize-followup-batch.ps1`
(`02a3dac8837e3c0193ef2a4713c0902dffbe65be1861847d830004269e1f5174`),
which has zero PowerShell parser errors and executes nine Cargo groups, the
30 Python tests, and one real-adapter viewer run in one coordinator job. The
App02 child script hash is
`30b6b8818c650a1c0947dc10dfb60a9377e95dd936e84a2d749b2079ce69c71d`.

Unified run `5258f82da9e041f1aca557eebfab2ccb` for job
`8cd4be20e6ab4fde858100c88379ea7c` reached the current source. The WGPU timer
group passed again with 5/5 tests, but the next `zircon_runtime` GPU-timing
group stopped after 2,694.140 seconds on 54 shared test-harness compile
errors. The viewer and Python consumer stages did not run, so this receipt
contains no GPU latency or performance result.

11. Cargo-aware current-source job `1ccefd55b24740558cba5b07fa6f823d`, run
    `4479497a25944e20bd16c620a4e3d451`, subsequently passed the App02
    real-adapter stage with exit code 0 in 1,139.013 seconds. The host adapter
    was `NVIDIA GeForce RTX 3060 Laptop GPU`, driver `32.0.15.9186`, selected
    with `high_performance_no_fallback` on `wgpu(vulkan)`. The viewer retained
    all 31 measured frames after five warmups, with measured generations 7
    through 37 and no outlier removal. The generated screenshot SHA-256 was
    `8ee5e7146b7d63727ba378fc682f5f3e3679d98fd8b6db3df1b5380cc1625707`.
    The validation copy has since been removed, so the evidence was frozen
    from the coordinator terminal receipt into
    `.codex/state/session-coordinator/cargo-runs/app02-gpu-terminal-evidence-1ccefd55.txt`
    (SHA-256
    `8f88c70fbbc7de45d35a8217dc72f78ba1aceb2bceb80eae524996569fe397ac`).
    A replay validator recomputes the full distribution from those raw rows;
    the main continuation will run that validator together with the remaining
    Runtime06/04 gates rather than repeat this 19-minute GPU stage.

## Accepted Real-Adapter Distribution

| Scope | Min (us) | Median (us) | P95 (us) | Max (us) |
|---|---:|---:|---:|---:|
| total GPU frame | 102 | 109 | 205 | 218 |
| direct GPU scene upload | 3 | 5 | 5 | 5 |
| direct output transfer | 16 | 17 | 37 | 38 |
| direct overlays | 3 | 4 | 4 | 4 |
| direct scene content | 78 | 83 | 161 | 174 |

These are nearest-rank aggregates over the 31 raw samples in the frozen
coordinator evidence. They establish the requested named-adapter timing
distribution; they are not a claim of renderer speedup because this milestone
changes measurement quality rather than the rendering algorithm.

Coordinator Main replay run `e3880656eefa4064aaa5920b37a1cb4d` accepted the
frozen App02 evidence together with Runtime02, Runtime06, and Runtime04. It
pins source job `197e37fe25f94d00915fcd890b03724d`, source run
`1562528434194a17879de2abbc2dbebf`, 19 source Cargo groups, 30 Python tests,
one real GPU run, and the 205 us total-frame P95.

## Remaining PBR-P1-23 Work

This milestone removes the one-shot timing weakness, but does not close the
whole finding. A later milestone must bind the sampled frames to a visual
stability oracle/golden policy and add same-scene, same-quality paired runs on
locked hardware before any comparison with Unreal or HDRP is valid.
