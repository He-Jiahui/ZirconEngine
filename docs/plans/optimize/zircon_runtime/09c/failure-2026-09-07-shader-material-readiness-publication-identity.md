---
handoff_kind: failure
status: open
created_at: 2026-09-07
summary_slug: shader-material-readiness-publication-identity
plan_link_mode: child_record_only
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/09c
related_code:
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_shader.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/default_pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/material_readiness.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests/prewarm_publication.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_manifest.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_manifest/runtime_identity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_pipeline_prewarm.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/readiness_diagnostics/last_good.rs
tests:
  - managed Windows resource readiness publication identity regressions
  - managed Windows shader source, material cache identity, pipeline key and last-good regressions
  - managed Runtime target-client production-library check
  - managed Editor target-editor-host check with dev-dynamic linking
---

# Runtime09C: Shader And Material Readiness Publication Identity

## 来源执行者

- Origin Session: `failure-roll-01a07160-frameworks01`.
- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：[Runtime/Editor production-library compilation](../../../zircon_runtime/frameworks/01/failure-2026-09-06-editor-runtime-import-contract.md).
- 修复责任计划：`docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md`
- 交接原因：09C owns shader/material artifact and in-memory PSO cache identity.
- Fixing Session: `failure-roll-01a07160-runtime09c`.

## 失败现象与复现证据

Managed job `20cb9378ea4f4d8eae2ff4b35464f95a` reports six E0599 errors:
`resource_streamer_ensure_material.rs:239,812,830,860` and
`resource_streamer_ensure_shader_source.rs:87,124` call the removed public
`ResourceReadinessGeneration::dependency_revision` accessor. Original log:
`.codex/tmp/app08-runtime-client-lib-20260906.log:4502`, sealed digest
`349d73696525d60d95a58f36373016c13113039aca2253e4b05fa9ce2e54ea4e`.
The same calls remain in current source at handoff creation.

Original commands, to repeat through managed Windows validation:

```powershell
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -Features target-client -NoDefaultFeatures -RuntimeProductDll -SkipTest -LinkMode static
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -Features zircon_runtime/target-editor-host -NoDefaultFeatures -LibTests -CheckOnly -LinkMode dev-dynamic
```

## 最低共享层根因

The resource owner intentionally replaced diagnostic-counter identity with retained
`ResourceReadinessRowIdentity` and `ResourceReadinessGenerationIdentity`. Shader
preparation, material candidate caches and PipelineKey still retain numeric
dependency revisions. Restoring counter accessors violates the resource boundary
guard `test_projection_diagnostic_counters_are_not_public_identities`.
Frameworks01 snapshots 2917-2920 were withdrawn and must not be reused.

Existing resource row identity retains the immutable publication, distinguishes
managers and remove/re-add lifetimes, and reuses unchanged rows across unrelated
publications. Runtime cache consumers must use that contract. Persistent shader
variant and disk artifact identities remain content based. The local Unreal
reference `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/Shader.h:634`
retains shader resource code through `TRefCountPtr` separately from shader hashes.

## 架构修复验收

- Migrate preparation producers, consumers and tests to retained row identity.
- Reject a missing publication for a resolved resource; never manufacture zero
  or a raw pointer integer as a valid cache identity.
- Cover root revision changes, dependency publication changes, unchanged rows,
  different managers, and remove/re-add while the old identity remains retained.
- Preserve last-good material publication and unchanged persistent variant keys.
- Run lower resource identity regressions, shader/material/PipelineKey consumers,
  the original commands and the upward Frameworks01/App08 gate with matching
  source snapshots. Existing performance acceptance remains required.

## 禁止临时方案

- Do not restore public diagnostic counters, compatibility aliases or a second
  identity authority. Do not treat a missing row as a cache hit.
- Do not close from source inspection, syntax checks or a queued receipt.
- Do not touch the user-excluded `zr_vm` dependency to obtain admission.

## 修复结果与回传

Open: source repaired and independently reviewed; managed acceptance pending.
The independent [09D residency consumer](../09d/failure-2026-09-07-residency-readiness-publication-identity.md)
has its own owner and lifecycle. No fixed artifact, commit or notification exists
for this lifecycle yet.

Repair source snapshot 2950, request `aceca1b540764071822c97d3824c179f`, contains
all 16 related files. The original 12-file baseline is snapshot 2942; the
prewarm entrypoint/helper baseline is 2947, and the new GPU test was absent.
The first review of 2943 reported C0/I1/M0 because the prewarm producer still
used the default empty identity. This was repaired before the final review.

Final independent review by task `01a07160-5337-7570-a507-ed6decf2d32b` on 2950
reported C0/I0/M0, with all 16 hashes checked before and after review. The binder
uses the actual ResourceStreamer publication, validates root revision and WGSL /
template contents, rejects drift, and assigns the same in-memory identity used
by drawing. A cold-load publication change permits one new source preparation;
it never relabels old shader source with a newer row.

Focused new GPU regression:
`runtime_prewarm_reuses_draw_publication_and_invalidates_only_changed_dependencies`.
It requires a real offscreen adapter, checks no extra PSO on prewarm-to-ensure,
unrelated publication reuse, dependency invalidation with unchanged root
revision, and rejection of stale revision / different canonical content.
Resource-manager identity fixtures cover retained clone, changed dependency,
unrelated publication, different manager, remove/re-add and persistent key
stability. All selected Rust files passed read-only rustfmt parsing and
`git diff --check`; Rust compilation, GPU behavior and performance remain
unexecuted, so these are not acceptance evidence.

The [handoff plan-identifier parser return](fixed-2026-09-07-handoff-letter-suffixed-plan-identity.md)
is the unique moved Tooling13 artifact for 09c/09d registration validation; its
coordinator closeout remains separate. The previous link to the deleted fixing
artifact is corrected without changing either lifecycle. No repeated Cargo
admission was requested while it would inspect the excluded external dependency.

On 2026-09-08 the 2950 source manifest was checked against the worktree before
assembling further validation inputs. Fifteen paths still match. Only
`resource_streamer_ensure_material.rs` differs: the existing worktree has
`let mut runtime = MaterialRuntime` where 2950 has `let runtime`.
Its current hash is
`53c8e85a5be04f0174619dad0dd01401a23add82744183bca3a92d34d41207f4`;
coordinator attribution still binds the prior
`109db73201ffaba92fdfe5fc3e8b37dc98dd78ec6e1b8e9a9dc754a513c7f7c8`.
No live lease owns this file at that boundary. The new bytes were preserved,
not silently attributed or covered by the earlier independent review. Their
provenance and consumer contract must be resolved before final source acceptance.

## 2026-09-08 managed compiler feedback and test-module repair

Runtime09D's client library compilation used the 2950 consumer files, except for
the explicitly recorded read-only mutable-runtime support above, in sealed input
`151f3a6ca148c08305ea907a7b9e828abf77f4753d2e061fe24bd648e53696a2`.
Job `5a4eaf2b8bcd4749adb83d495b255bb3` failed in Cargo check with 304 errors,
no linked test binary and zero tests executed. Full log and embedded receipt:
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime09d-client-support-20260908/results/residency-client-lower.log`.

The first diagnostic is E0583 at `ensure_pipeline/tests.rs:36`.
`ensure_pipeline.rs` loads the test owner through
`#[path = "ensure_pipeline/tests.rs"]`; its unqualified child declaration then
searches `ensure_pipeline/prewarm_publication.rs`, while the canonical child is
`ensure_pipeline/tests/prewarm_publication.rs`. The test owner now uses
`#[path = "tests/prewarm_publication.rs"]`, matching the adjacent
`shader_source/tests.rs` pattern. No test is removed, renamed or ignored.

Pre-edit hash still matched 2950. Lease request
`252ad948974a4c5abf6d835aa87979ab`, attribution
`9c58c3fab573485fb2b32ed54d35e6c5` and source snapshot 3010
(`febeb606ecc64cf0b86354ff518cacda`) bind the one-line change. New hash:
`aaa1bbec45b550b07da6f21e12b32501a603a17d36c381147af5d7c51c2841a7`.
Rustfmt parsing, including the resolved child module, and scoped diff checks
passed. The original 2950 review does not cover this later fix; compilation,
actual GPU execution and an incremental independent review remain required.

The same run reports seven retired-accessor calls in owned `last_good.rs`:
four `material_revision` calls and three `material_draw_generation` calls.
The current material path has staged, published and previous-published bundles;
`PublishedMaterialDrawProxy` selects one complete published bundle, while the
test-only `material` and uniform accessors prefer a staged candidate. The
last-good tests therefore need the current publication boundary as well as
accessor migration. Merely restoring singular getters would not prove the
published bundle's behavior. Unrelated UI/Text/graphics diagnostics keep their
own source ownership.

Source snapshot 3017, request `d4770355cafb4dffb23130ddc6c6685c`, now contains
the prewarm path correction and the migrated `last_good.rs`, hash
`f1fd1983d816e49e409b721d2657efb984247b750c29abfc2ea26289a587860c`.
The two existing ResourceStreamer regressions prepare a candidate, assert that
it is not yet published, and call the production candidate-publication boundary.
Every last-good runtime/generation/uniform observation then comes from one
`PublishedMaterialDrawProxy`. Rejection preserves the published generation and
both uniform bindings. Dependency recovery leaves that bundle visible while
staged, then retains it in the previous-published slot after publication. Source
revision assertions use the existing resource owner and still reject synthetic
material or root-shader revisions.

This is the ResourceStreamer publication-owner test boundary. It does not
simulate successful PSO admission or replace the separate prewarm/draw GPU
regression and upward rendering acceptance. The repaired test file passed
Rustfmt and the scoped whitespace check; it has not compiled or executed yet.
Incremental independent review remains required for both files in 3017.

That review, bound to source `3017` and record `3019`, was queued once to
`优化协调器验证效率` (`01a07063-6f03-7803-a12d-13ea015ca645`) in message
`01a07ec5-9d07-7010-8feb-f2c2ce5742f7`. A separate source-review conclusion
was requested for this lifecycle. The dispatch is not a review or dynamic pass.
