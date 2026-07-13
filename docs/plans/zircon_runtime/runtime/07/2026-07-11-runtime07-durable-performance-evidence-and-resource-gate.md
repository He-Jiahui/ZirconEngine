---
related_code:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - zircon_runtime/src/graphics/tests/render_profiling.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots
implementation_files:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/render/index.md
tests:
  - tests/acceptance/runtime-07-performance-hotpath-current-gates.md
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
doc_type: milestone-detail
status: in_progress_durable_evidence_guard_28_passed_profiling_trace_accepted_fps_current_source_compile_blocked
---

# Runtime 07 durable performance evidence and resource gate

Date: 2026-07-11

## Evidence-owner hard cut

The historical 10fps RenderDoc facts—230 draws, 231 pre-draw buffer copies and
31 render passes—remain valid routing evidence, but their permanent owner is
now Runtime 07's numbered output archive. The retired
`20260611-0416-rendering-10fps-analysis.md` session note is removed from the
Runtime 07 plan and hotspot module-document inputs. Active sessions remain
coordination and lease state only.

The execution prerequisite now reads numbered Runtime 07 records first and
uses the coordinator for current graphics/runtime ownership. No missing note is
restored, no fallback path is added, and no historical performance assertion is
silently promoted to a current FPS result.

Six performance-hotspot guard source owners also stopped loading the live
Runtime implementation session note. Concrete historical/status/count anchors
now resolve from Runtime 07 and Runtime 15 numbered archives. Parent plans and
module documents remain routing/current-summary owners instead of being
repopulated with duplicated historical rows. A new guard rejects the entire
`.codex/sessions/` path family in both the top-level `performance_hotspots.rs`
owner and every Rust source under the folder-backed `performance_hotspots/`
tree.

The current standalone performance-hotspots suite initially exposed 12 stale
parent/status-mirror failures (15/27). After repairing the lowest shared
evidence routes and removing obsolete inputs, the final current-source suite
compiles without warnings and passes 28/28, including the new durable-evidence
family guard. Direct `performance_hotpath_boundary_audit` reports source 46,
test owner 91, empty doc/Cargo anchor lists, mirror present and `risks=[]`.

## Current executable resource gate

- The two current-source profiling library builds remain accepted. The second
  completed in 1902.799 seconds.
- The optimized trace lib-test attempt never reached execution and produced no
  trace artifact; it was stopped when its managed lane fell to about 1.09 GiB
  free.
- The previously recorded local ZrVM library and runtime DLL directories,
  `E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug` and
  `E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug`, were both absent during
  this current-state check.
- Free space at the check was C 13.75 GiB, D 25.13 GiB, E 10.70 GiB and F 6.05
  GiB. Every visible managed drive was below the repository's 50 GiB Cargo
  threshold, while three unrelated Cargo root lanes were active.

Therefore no new profiling, trace or Vampire FPS Cargo command was started and
no unrelated Cargo/rustc process was terminated. Compile-only evidence is not
reported as trace execution, and historical 10fps evidence is not reported as
the required two-run current baseline.

## Status decision

Status remains
`in_progress_durable_evidence_guard_28_passed_profiling_build_accepted_trace_fps_resource_blocked`.
M0.2 profiling build is accepted; M0.1 two-run Vampire FPS and M0.3 trace
artifact execution remain open. This record closes only durable evidence
routing and the current resource-state audit.

## Current artifact-reuse audit

A follow-up current-state audit ruled out two apparent no-build shortcuts:

- The ZrVM repository has no `build/` directory and
  `ZR_VM_RUST_BINDING_LIB_DIR` is unset. Exact searches under the managed D/E/F
  Cargo target roots, the repository target directory and the local Cargo tree
  found no `zr_vm_rust_binding.lib` or `zr_vm_rust_binding.dll`. The only DLL
  found in the ZrVM source tree belongs to the language-server extension and
  has no accompanying import library. It is not promoted to a current runtime
  link artifact.
- `direct_runtime_frame_submit_exports_perfetto_trace_artifacts` is compiled
  only with both `profiling` and `profiling-chrome`. Five extant ordinary
  Runtime lib-test executables were queried through `--list`; none contains the
  trace test, so a debug binary cannot be substituted for the unfinished
  profiling lib-test link.

At this check free space was C 12.94 GiB, D 16.48 GiB, E 12.56 GiB and F 2.49
GiB, with 12 external Cargo/rustc processes active. No Cargo process was
started or stopped. M0.1 and M0.3 remain open for their exact prescribed
commands; this audit only proves that neither can be honestly completed by
relabeling stale or feature-incompatible artifacts.

The coordinator cleanup planner found seven released lanes older than one
hour, while correctly denying active leases, live processes and retention
paths. Applying that exact plan was rejected with
`maintenance_unauthorized`; no directory was removed manually and no
maintenance capability was bypassed. The resource gate therefore remains
unchanged.

## Third consecutive blocked-state audit

The third consecutive current-state check still found no ZrVM build, library,
runtime directory or configured binding link path. Free space deteriorated to
C 12.71 GiB, D 5.97 GiB, E 13.49 GiB and F 0.08 GiB, with 13 Cargo/rustc
processes and four coordinator-managed active or leased lanes.

The no-build alternatives have been exhausted: the old language-server DLL has
no import library, ordinary test binaries do not contain the profiling trace
test, and coordinator-managed cleanup requires a separate maintenance
capability that this Session does not possess. Runtime 07 remains incomplete;
progress now requires an external-state change or explicit new maintenance
authority. No process or target directory was modified during this audit.

## Resource recovery and current-source trace acceptance

Disk capacity later recovered enough to run the exact M0.3 command in a
coordinator-managed lane:

`cargo test -p zircon_runtime --lib direct_runtime_frame_submit_exports_perfetto_trace_artifacts --profile profiling --features profiling,profiling-chrome --locked --jobs 2 --target-dir D:\cargo-targets\runtime07-trace-20260711 -- --nocapture --test-threads=1`

An earlier compatible-lock build snapshot reached execution after 98m55s but
failed because `preview-sky` loaded transient `scene-depth` before a producer.
The mesh pass descriptor changed while that optimized binary was compiling, so
the result was retained as a mixed-snapshot diagnostic rather than reported as
a current-source regression. The unchanged-lock current-source rebuild was
then pinned by SHA-256 for `Cargo.lock`, `mesh.rs`, `pass_authoring.rs`, and
`render_profiling.rs`. It finished the optimized lib-test build in 67m59s and
passed 1/1 in 12.30 seconds with 7571 tests filtered out.

The passing test generated and read `timeline.zrtrace.json`,
`timeline.perfetto.json`, `hotspots.json`, and `summary.md` in its temporary
output tree. It verified both trace formats contain `submit_runtime_frame`,
`render_frame_with_pipeline`, `DepthPrepass`, and `depth-prepass`, then removed
the temporary tree as required by the existing test contract. This is accepted
execution evidence, not a claim that those temporary files were preserved.

M0.2 and M0.3 are now accepted. M0.1 remains open because the current ZrVM
import library/runtime DLL pair is still absent, so the two authoritative
Vampire FPS samples and the required deviation calculation cannot be produced.
Runtime 07 therefore remains `in_progress`.

## M0.1 current-binding provenance audit

A broader search later found an import-library/runtime-DLL pair under
`E:\Git\zr_vm\.codex\tmp\aot-clean-verify-20260622-121531`. It does not change
the M0.1 decision. Both files were generated on 2026-06-11, and the copied
`CMakeCache.txt` points back to the now-absent `E:/Git/zr_vm/build-msvc` source
and binary directories. The current clean ZrVM checkout is instead at
`2eb70efa143c44c9acc91e002f9f054f54e9f588`, dated 2026-07-09. The isolated
pair therefore has neither current-source provenance nor a reproducible current
build directory and is rejected as an authoritative FPS input.

At this resumed check C/D/E/F free space was approximately
11.64/26.85/55.85/1.17 GiB. D remained below the repository's 50 GiB Cargo
threshold. E had recovered above it, but coordinator state still contained
other sessions' running, orphaned and leased Cargo lanes, while six Cargo and
three rustc processes were visible. No external process was terminated, no
released lane was manually deleted, and no new full ZrVM/Runtime build was
started into that contention. M0.1 remains `in_progress` until the exact Runtime
command produces two numeric FPS samples whose deviation is below 20%.

## Current ZrVM recovery and Render 18 first-writer blocker

The current ZrVM checkout at
`2eb70efa143c44c9acc91e002f9f054f54e9f588` was built outside Cargo under
`D:\zrvm-builds\runtime07-m0-current-2eb70efa`. The current import library has
SHA-256
`1E6DB5B049F62E57177974FFA0F50B5D7C3897062B34B4773431ADD2759694A9` and the
runtime DLL has SHA-256
`8C5450B73DF4DA3BDA7E1FFEFB634370CB666A1B14B1414F9BD4B335EA86BFEA`.
The binding build also exposed two missing AOT compiler translation units in
the ZrVM Rust-binding CMake target; the current external CMake owner was
corrected without adding a Zircon compatibility layer.

Seven remaining Vampire model sidecars were still valid version-6 records
while the other 52 sidecars had already moved to version 7. They were migrated
to `format_version = 7` and `source_digest`; the current project audit reports
59 version-7 sidecars, zero version-6 sidecars and zero `source_hash` keys. A
direct diagnostic run then completed in 96.76 seconds and printed
`fps_current=34.67983575629786` with
`frame_ms_current=28.835199999999997`. This diagnostic predates the formal
stable-snapshot pair and is not counted as an authoritative M0.1 sample.

The exact prescribed Cargo command was subsequently attempted against the
current binding. One build timed out during concurrent recompilation, one
failed on concurrently edited asset-migration enum exhaustiveness, and the
latest completed build ran the target for 89.97 seconds before failing at
render submit:

`render graph pass \`opaque-mesh\` loads transient attachment \`scene-color\` before any producer writes it`

Directly rerunning that same test executable reproduced the identical error in
79.41 seconds. This proves the failure for that compiled snapshot, but it does
not prove the same root cause for the continuously changing current source.
Source review confirms that camera-stack attachment policy is applied during
GPU execution after graph compilation, so it cannot itself cause this graph
compiler diagnostic. A fresh current-source rebuild on 2026-07-12 compiled the
production Runtime and then stopped while compiling the shared lib-test target
on one concurrent Asset migration test error (`AssetReference::guid()` no
longer exists). It therefore did not execute the focused graph contract. The
Render 18 and Asset files are owned by active sessions, so Runtime 07 has not
added a competing workaround or weakened either contract.

M0.1 therefore has zero formal samples and remains `in_progress`. The immediate
current-source blocker is the concurrently changing Asset migration lib-test
source (successive rebuilds reached changing `guid()`, transaction recovery
match, crash-window helper visibility and `PathBuf` typing errors); the
Render 18 scene-color result must be re-established from a fresh binary after
that compile gate clears. Acceptance still requires two successful
executions of the exact Cargo command, unchanged source snapshots around each
run, numeric `vampire_runtime_perf` output and deviation below 20%.

An independent production-only graph probe was also attempted outside the
repository worktree to avoid the shared lib-test target. It still observed the
same live Asset migration churn while compiling `zircon_runtime`: transaction
schema visibility and a newly added `AssetImportOutcome::reference_repairs`
field were inconsistent in that snapshot. The probe therefore produced no
graph verdict and no repository source change.

## Current-source graph verdict and FPS progress

After the Asset migration source stabilized, the same production-only probe
completed successfully against current source. The compiled forward-plus graph
reported `opaque-mesh` before `preview-sky`, so the old mixed-snapshot
`opaque-mesh` load-before-producer diagnostic is no longer a current production
blocker. The focused lib-test also reached execution, but its assertion now
fails because it still requires `depth-prepass` to write `gbuffer-normal`;
current source no longer declares that access. This is a stale Render 18 test
expectation, not a production graph compile failure, and Runtime 07 has not
changed another active plan's source to mask it.

The exact M0.1 command then completed successfully once and produced the first
current-source numeric sample:

- sample 1: `30.894424483213513 FPS`, `32.368300000000005 ms`, 116 mesh draws;
- command: `cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features backend-zr-vm --locked -- --nocapture --test-threads=1`;
- date: 2026-07-12;
- result: 1/1 passed in 83.89 seconds after the current lib-test build.

A rebuilt current test executable subsequently passed the same target at
`29.641661948702144 FPS` and `33.7363 ms`, yielding a mean-relative deviation
of `4.138895%`. That execution is useful stability evidence, but it is not
promoted to the second formal Cargo sample: active Shader 06 and Render 18
sessions changed Runtime source during the enclosing exact Cargo attempt, and
the second exact attempt itself stopped on a transient Shader 06 test-module
compile inconsistency before those edits converged. M0.1 therefore has one
accepted exact-command sample and still requires one more exact-command pass
after the shared Runtime source reaches a stable window. No foreign source was
reverted or patched by Runtime 07.

## M0.1 final two-run acceptance

The exact prescribed Cargo command completed again on 2026-07-12 after the
Render 18 compiler fixes converged:

`cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features backend-zr-vm --locked -- --nocapture --test-threads=1`

The accepted formal pair is:

- sample 1: `30.894424483213513 FPS`, `32.368300000000005 ms`, 116 mesh draws;
- sample 2: `33.98320549984198 FPS`, `29.426299999999998 ms`, 116 mesh draws.

Both exact commands passed 1/1. Sample 2 finished the test body in 82.76
seconds. The mean-relative FPS deviation is `9.521868%`, below the required
`<20%` threshold. The current ZrVM import library and runtime DLL provenance
remains the SHA-256 pair recorded above.

An intervening exact-command pass also produced `39.22630044992567 FPS` and
`25.493100000000002 ms`, but the shared Runtime source continued changing
during that run. It is retained as diagnostic evidence and is not substituted
for the accepted formal pair. Runtime 07 M0.1, M0.2 and M0.3 are now complete.

## M1 current completion evidence

The ECS/query and extract counters now complete their production publication
path. `EcsFramePerformanceDiagnostics::publish(...)` writes the completed
World-local query/change aggregate to the runtime diagnostic store after every
`WorldDriver` tick, including zero-valued idle frames. The exact Vampire test
also prints the current query, change-detection and extract values next to the
FPS/render work sample, so the same authoritative path can be used for future
regression diagnosis.

Current fixed-scenario counter evidence is:

- QueryState: 128 entities, 8 repeated runs, 8 hits, 1 initial miss and 1
  initial rebuild; unchanged runs add no rebuild.
- Change detection: 6 stale mark checks, 0 added matches and 0 changed matches.
- Extract cache: unchanged captures record rebuilds `[1, 0]`, hits `[0, 1]`,
  misses `[1, 0]`, with stable non-zero output bytes; a scene mutation records
  rebuilds `[1, 1]`.

The current-source package compilation window was repeatedly contaminated by
live Shader 06 and Physics 03 edits. A fresh exact focused build reached
foreign compile errors (Realtime IBL test imports/lifetime and newly extended
ColliderShape matches) before it could execute the new publication test. No
foreign owner source was changed by Runtime 07. The last successfully built
current Runtime test executable independently passed the plan's two existing
filter gates: `frame_extract_rebuild` 2/2 and `ecs_query` 58/58. The standalone
current-source Runtime 07 performance guard passed 28/28 after adding the
publication anchors. Therefore M1.1/M1.2/M1.3 behavior and durable evidence are
complete, while the full-package current-source Cargo rerun remains a shared
workspace validation blocker rather than an unimplemented Runtime 07 design
item.

Final closeout audits on 2026-07-12 passed the standalone Runtime plan-status
suite 48/48, the Runtime 07 performance-hotspot guard 28/28, and direct
`performance_hotpath_boundary` / `runtime_plan_status_boundary` audits with
`risks = []`. Rust formatting, Python syntax checks, and scoped
`git diff --check` also passed. The repository-wide output-record audit still
reports 20 pre-existing violations in foreign Editor/Render plan families;
none targets Runtime 07. The applicable Runtime 07 failure graph is clear,
while unrelated active failure handoffs and foreign Cargo lanes remain visible
as shared-worktree diagnostics only.

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | Runtime 07 authoritative closeout | completed | 2026-07-12 | `frame_spans_trace_accepted_completed`、`scoped_counter_points_runtime_published_completed`、`named_assertions_behavior_accepted_completed` 与 `authoritative_inventory_completed` 全部闭合。 |
| M3 | M3-T testing | passed | 2026-07-12 | plan-status 48/48、performance-hotspot 28/28、两项 direct boundary audit `risks = []`、rustfmt/Python/diff-check 通过；共享源码全包编译仍由活动外部 owner 占用且不构成 Runtime 07 设计缺口。 |
