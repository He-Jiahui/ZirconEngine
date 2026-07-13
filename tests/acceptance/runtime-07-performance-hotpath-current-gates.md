---
related_code:
  - zircon_runtime/src/scene/ecs/query
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
output_records:
  - docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md
  - docs/plans/zircon_runtime/runtime/07/2026-07-11-runtime07-durable-performance-evidence-and-resource-gate.md
status: in_progress_durable_evidence_guard_28_passed_profiling_trace_accepted_fps_zrvm_blocked
---

# Runtime 07 Performance Hotpath Current Gates

Date: 2026-07-11

## Current managed-binary evidence

- `ecs_query`: 58/58 passed.
- Broad `extract`: 287 passed and 23 failed. The failures are 7 active Render
  frame/pipeline contracts, 15 active Runtime UI render/text contracts, and the
  Runtime 05 mesh geometry-order contract. No ECS-query test failed.

## Profiling build evidence

- Command: `cargo build -p zircon_runtime --lib --profile profiling --features
  profiling,profiling-chrome --locked --jobs 2`.
- The cold managed-lane build completed and produced the optimized runtime
  `rlib`, `cdylib`, and PDB. Its artifact window was 2026-07-11 14:10:51 to
  14:57:38 (about 46 minutes 47 seconds).
- A second current-source build completed with exit code 0. Its artifact window
  was 15:00:35 to 15:32:17 (1902.799 seconds). Concurrent runtime-interface
  edits changed the fingerprint, so this is a second optimized compile, not a
  no-op cache timing.
- Both runs show the bottleneck after dependency compilation: release-level
  optimization of the large `zircon_runtime` crate's 16 codegen units, followed
  by `rlib`/`cdylib` emission. The second run began runtime code generation at
  15:02 and did not finish the runtime artifacts until 15:32.

## Trace execution gate

- Target command: `cargo test -p zircon_runtime --lib
  direct_runtime_frame_submit_exports_perfetto_trace_artifacts --profile
  profiling --features profiling,profiling-chrome --locked --jobs 2 --
  --nocapture --test-threads=1`.
- The command spent 37 minutes compiling the optimized lib-test binary and did
  not reach test execution. It was stopped when the managed D: lane fell to
  about 1.09 GiB free; the job is recorded with exit 124 and the lane was
  released. No trace artifact or pass result is claimed.
- The profiling library lane was then cleaned after preserving the build facts.
  Locked proc-macro DLLs left about 0.026 GiB behind; no unrelated Cargo or
  rustc process was terminated.

## Current resource and evidence-owner check

- The historical 10fps/RenderDoc facts are now routed through Runtime 07's
  numbered output archive. The retired `20260611-0416` session note is no longer
  a plan or module-document evidence dependency.
- On 2026-07-11 the previously recorded local ZrVM `lib/Debug` and `bin/Debug`
  directories were both absent, so the authoritative Vampire FPS command still
  cannot produce the required two comparable samples.
- All visible managed drives were below the repository's 50 GiB free-space
  threshold (C 13.75, D 25.13, E 10.70, F 6.05 GiB at the check), and three
  unrelated Cargo root lanes were active. No new Runtime 07 Cargo/profiling
  build was started and no external process was terminated.
- The standalone current-source `performance_hotspots` suite first exposed 12
  stale parent/status-mirror routes (15/27). Concrete evidence now comes from
  Runtime 07/15 numbered archives, six live-session inputs are removed, and a
  recursive no-session-evidence guard is mounted. The final suite passes 28/28
  and its direct structure audit reports `risks=[]`.

## Decision

The Runtime 07 ECS query hotpath gate and M0.2 profiling build gate are
accepted. Runtime 07 stays `in_progress`: the broad extract gate is not green,
the required two-run FPS baseline is still blocked by the unavailable current
ZrVM link library, and the frame-trace test has not reached execution under the
current disk/optimized-test-binary budget. This record does not convert the
external Render/UI failures into Runtime 07 ownership and does not treat a
compile-only trace attempt as trace evidence.

## Current artifact-reuse follow-up

- ZrVM currently has no `build/` directory, the required link-directory
  environment variable is unset, and managed target roots contain no current
  `zr_vm_rust_binding` import library/runtime DLL pair. The language-server
  extension's standalone old DLL is not accepted as FPS evidence.
- The exact trace test is guarded by both `profiling` and `profiling-chrome`.
  Five existing ordinary Runtime lib-test binaries were inspected with
  `--list`; none includes it, so no feature-incompatible binary is counted as
  M0.3 execution.
- The latest resource check reports C/D/E/F free space of
  12.94/16.48/12.56/2.49 GiB and 12 active external Cargo/rustc processes. No
  new build was started and no external process was terminated.
- A coordinator cleanup plan identified seven old released lanes and protected
  active/retained lanes, but execution was rejected as
  `maintenance_unauthorized`. No manual deletion or privilege bypass occurred.

## Third consecutive resource-gate result

The current check reports C/D/E/F free space of 12.71/5.97/13.49/0.08 GiB,
13 Cargo/rustc processes, four active/leased managed lanes, and still no ZrVM
build/link product. Because artifact reuse and authorized cleanup alternatives
have both been exhausted, M0.1 and M0.3 cannot proceed until external state or
maintenance authority changes.

## Current-source trace acceptance

Resource recovery subsequently allowed the exact profiling-chrome lib-test to
run in its managed D: target lane. The fixed current-source snapshot completed
the optimized build in 67m59s and passed 1/1 in 12.30 seconds (7571 filtered).
The test generated native and Perfetto timelines plus hotspots and summary in a
temporary directory, read them, asserted the four required anchors, and cleaned
the directory by its existing contract. M0.3 is accepted; the old resource gate
above is retained as chronology, not current status.

Runtime 07 remains `in_progress` solely on the M0.1 authoritative two-run
Vampire FPS baseline and deviation table, which still requires a current ZrVM
import library/runtime DLL pair.

## Current ZrVM binding provenance follow-up

A recursive source-tree audit found a matching `.lib/.dll` pair below
`E:\Git\zr_vm\.codex\tmp\aot-clean-verify-20260622-121531`, but both files were
generated on 2026-06-11. Its copied CMake cache still names the absent
`E:/Git/zr_vm/build-msvc` directories, while the current clean ZrVM checkout is
the 2026-07-09 commit `2eb70efa143c44c9acc91e002f9f054f54e9f588`.
Consequently this is stale isolated verification output, not a current binding
that can satisfy M0.1.

The resumed resource check reported C/D/E/F free space of
11.64/26.85/55.85/1.17 GiB, six Cargo processes, three rustc processes, and
other sessions' active/orphaned/leased coordinator lanes. No external process
or lane was modified. M0.1 remains open for the exact command's two FPS values
and `<20%` deviation result; no stale binary is relabeled as acceptance.
