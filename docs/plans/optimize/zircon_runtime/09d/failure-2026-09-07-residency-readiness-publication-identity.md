---
handoff_kind: failure
status: open
created_at: 2026-09-07
summary_slug: residency-readiness-publication-identity
plan_link_mode: child_record_only
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/09d
related_code:
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/contract.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/manager.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/manager/ticket_issuance.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/manager/device_recovery.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/work_queue.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_blocks/contract.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_blocks/semantic_load.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/gpu_upload/plan.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/gpu_upload/submit.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor/contract.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/gpu_residency.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/gpu_maintenance.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor/owner.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/tests.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/tests/device_recovery.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/tests/work_queue.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/tests/semantic_executor.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/tests/publication_identity.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs
tests:
  - managed Windows resource readiness publication identity regressions
  - managed Windows render asset residency ticket and completion regressions
  - managed Runtime target-client production-library check
  - managed Editor target-editor-host check with dev-dynamic linking
---

# Runtime09D: Residency Readiness Publication Identity

## 来源执行者

- Origin Session: `failure-roll-01a07160-frameworks01`.
- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：[Runtime/Editor production-library compilation](../../../zircon_runtime/frameworks/01/failure-2026-09-06-editor-runtime-import-contract.md).
- 修复责任计划：`docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md`
- 交接原因：09D owns residency request, completion and device/resource lifetime identity.

## 失败现象与复现证据

Managed job `20cb9378ea4f4d8eae2ff4b35464f95a` reports E0599 at
`render_asset_residency/manager.rs:647,656`: calls to
`ResourceReadinessGeneration::dependency_revision` and `sequence` no longer exist.
Original log `.codex/tmp/app08-runtime-client-lib-20260906.log:4489`, digest
`349d73696525d60d95a58f36373016c13113039aca2253e4b05fa9ce2e54ea4e`.
Current source retains these calls and Copy tickets with numeric identities.

Original commands, to repeat through managed Windows validation:

```powershell
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -Features target-client -NoDefaultFeatures -RuntimeProductDll -SkipTest -LinkMode static
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -Features zircon_runtime/target-editor-host -NoDefaultFeatures -LibTests -CheckOnly -LinkMode dev-dynamic
```

## 最低共享层根因

Resource readiness now exposes retained immutable publication identities. Residency
TicketSeed, request/completion tickets and reconciliation still compare diagnostic
sequence and dependency counters. Counter identity can collide across resource
manager lifetimes and cannot preserve the referenced publication. The resource
owner's public-counter prohibition is intentional; Frameworks01 snapshots
2917-2920 that attempted accessor restoration were withdrawn.

## 架构修复验收

- Retain the appropriate resource readiness publication in residency tickets and
  migrate ownership/borrowing through request, completion, release and reconciliation.
- Preserve resource, asset revision, demand, device epoch, scope and route checks.
- Cover stale completion rejection, manager replacement, remove/re-add, dependency
  changes and unchanged-resource behavior when unrelated rows are republished.
- Run resource identity, residency lower and direct consumer regressions, then
  original Runtime/Editor and upward Frameworks01/App08 managed acceptance.

## 禁止临时方案

- Do not expose diagnostic counters, cast raw addresses to persistent IDs, or
  duplicate resource identity authority to preserve Copy on tickets.
- Do not omit stale completion checks or count static inspection as dynamic proof.
- Do not touch the user-excluded `zr_vm` dependency to obtain admission.

## 修复结果与回传

Open: source repaired in snapshot 2959 (request
`d683092a56fe40868592e8b435c93424`), compared with pre-edit snapshot 2958
(`de45127fbc3a4c45acee6c693930d117`). Fixing Session:
`failure-roll-01a07160-runtime09d`.

Tickets and seeds retain `ResourceReadinessRowIdentity`; transition methods
borrow tickets while asynchronous request, completion, release and GPU tracking
owners retain clones. The shared resource identity implementation in
`zircon_runtime/crates/zr_resource/src/readiness_generation.rs` is unchanged.
Reconciliation preserves resource, root revision, demand, device, scope and route
checks; mixed catalog/readiness revisions fail before mutation or ticket-ID
consumption. Executor cancellation and preparation completion compare the full
publication-bearing ticket.

Four real ResourceManager regressions cover unrelated publications for pending
and resident tickets, dependency changes with unchanged root revision and stale
upload completion, manager replacement and remove/re-add, and mixed snapshot
atomicity. The executor regression rejects a same-ID cancellation carrying a
different publication. Source-hash changes force real publication updates;
identical-record upserts are not used as change evidence.

All 20 source paths parse with rustfmt. `git diff --check` passes. The 15 tests in
`tools.tests.test_frameworks_01_resource_crate_boundary` passed in 7.895 seconds.
These are boundary/syntax checks, not Rust dynamic acceptance. Snapshot 2959 is
reviewed by the existing independent task
`01a07160-5337-7570-a507-ed6decf2d32b`: Critical 0, Important 0, Moderate 0.
The reviewer checked the source fingerprints before and after review; all 20
paths matched snapshot 2959. Managed Runtime/Editor dynamic acceptance remains
pending. Source leases were released after freezing the snapshot.

The independent [09C shader/material consumer](../09c/failure-2026-09-07-shader-material-readiness-publication-identity.md)
is being accepted separately. No fixed artifact, commit or notification exists
for this lifecycle yet.

## 2026-09-08 lower shared resource validation

Before preparing managed inputs, all 20 current source hashes were compared with
snapshot 2959 and matched. No residency source changed after its independent review.
The checked support chain is ResourceManager publication and row identity,
residency seed/ticket admission, asynchronous completion and reconciliation, then
ResourceStreamer and Runtime/Editor consumers.

The sealed input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime09d-publication-2959-20260908`
contains 10,950 files, digest
`e9d1ac77491046430dd8e009deeffd2786a11f0173a78dfdbe57376e8946efa3`.
It derives from the complete Runtime test input
`5dba8bc7d7ed12b0c2d688e4400110b233fc88ab2c10499ce59d66300f717096`
by overlaying exactly the 20 frozen residency source/test files. Shared resource
identity and external inputs remain the sealed parent's exact bytes; no live
`zr_vm` checkout was inspected or edited.

Managed Windows job `78e2ee15dfd9417093843f1adb788c3b` executed:

```powershell
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zr_resource -LibTests -NoDefaultFeatures -SkipBuild -LinkMode static -SourceSnapshot E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime09d-publication-2959-20260908/inputs/source -SourceSnapshotDigest e9d1ac77491046430dd8e009deeffd2786a11f0173a78dfdbe57376e8946efa3
```

Cargo retained `--locked`. The complete shared resource library reported
224 passed / 0 failed / 10 ignored / 0 filtered out, harness time 0.66 seconds.
Entry and embedded managed receipt both report exit 0. Queue time was 11.969
seconds, source sync 25.906 seconds, check 40.867 seconds, compile/link 45.002
seconds and test stage 1.895 seconds. The sealed source manifest was reverified
after the run. Full log and durable embedded receipt:
`results/resource-lower-library.log` under that input directory.

This is dynamic lower-layer evidence only. The Runtime residency module is not
compiled by the `zr_resource` package command. Its regression tests, the original
Runtime client and Editor commands, upward acceptance, coordinator evidence
binding and independent closeout review still remain before return and commit.

## 2026-09-08 client compilation and support boundaries

The next input, `runtime09d-client-support-20260908` under the same benchmark
root, seals 10,952 files with digest
`151f3a6ca148c08305ea907a7b9e828abf77f4753d2e061fe24bd648e53696a2`.
The 20 residency files remain exactly 2959. Seventy-nine additional frozen
support files come from Frameworks01 snapshots 2901, 2904, 2914, 2924, 2927,
2931, 2934, Render11 snapshot 2905 and 09C snapshot 2950, with later snapshots
winning overlapping paths. Withdrawn snapshots 2917-2920 are excluded.
`support-provenance.json` records the final hash and source snapshot per path.

Two existing consumer corrections are sealed as read-only support snapshot 3008:
`resource_streamer_ensure_material.rs` hash
`53c8e85a5be04f0174619dad0dd01401a23add82744183bca3a92d34d41207f4`
and `render_pass_execution_context/gpu/native.rs` hash
`b422c550f1aae7ed00f3da7416ab8787c2994aedf627c247ebb934ffd62405a6`.
Their post-review provenance remains pending; this Session did not edit or
attribute those bytes or include them in its commit scope.

Managed job `5a4eaf2b8bcd4749adb83d495b255bb3` ran the Windows validator with
`-Package zircon_runtime -Features target-client -NoDefaultFeatures -LibTests
-TestFilter graphics::scene::resources::render_asset_residency -SkipBuild
-LinkMode static`, plus that exact source path and digest. Cargo check failed
with 304 errors and exit 101; wrapper/receipt exit 1, no compile/link stage and
zero tests executed. Queue time was 11.776 seconds, source sync 23.549 seconds
and check 347.958 seconds. Full log and embedded receipt are retained at
`results/residency-client-lower.log`; the input manifest still matches after exit.

The failure includes six missing Editor fixture inputs, unrelated UI/Text and
graphics test/production migrations, a 09C prewarm test-module path error, and
seven 09C last-good test calls to retired singular material accessors. The
prewarm module declaration is repaired separately in 09C snapshot 3010; its
behavior test has not run. Restoring retired material accessors is not used to
hide the current staged/published material contract.

The residency consumer also calls the two-argument geometry replay commit
contract, while the old sealed parent has its one-argument implementation.
Current `resource_streamer/geometry_replay.rs` already implements both completion
flags, hash `0de7343d3abae45db255957bc90844358820a6cde8e14b18df167646f2bafd67`.
It has no coordinator attribution at this boundary, so its exact support
provenance must be resolved before use. The caller must retain the replay
completion check; dropping the second argument would lose the truncation guard.

The production-only Runtime client check finished against the same immutable
input as managed job `14ab8efb2c8d40b08a1f7f4c882e2a1e`, under operational
Session `validate-matrix:failure-roll-01a07160-frameworks01`. Its command uses
`-Package zircon_runtime -Features target-client -NoDefaultFeatures -SkipTest
-CheckOnly -LinkMode static`, plus the same source path and digest. Cargo check
reported 9 errors, exit 101; wrapper/receipt exit 1 and zero tests. Queue time
was 19.007 seconds, source sync 27.958 seconds and check 117.214 seconds. Log and
embedded receipt: `results/runtime-client-production-check.log`. The source
manifest was reverified after exit.

The non-VM errors are the missing geometry replay support above, the Render11
environment-frame support closure, and four UI/Text migration diagnostics.
Current rich-text and paragraph source already contain relevant corrections;
the inline-widget change also requires its current provider-aware callers.
Those borrowed support changes must be frozen with their full producer and
consumer closure before another client check. The remaining three diagnostics
are in script VM/host consumers and remain outside the user's current scope.
Neither failed check is dynamic residency evidence or permits return or closeout.
