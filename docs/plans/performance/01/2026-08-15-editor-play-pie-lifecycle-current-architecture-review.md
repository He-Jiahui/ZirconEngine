---
related_code:
  - zircon_editor/src/core/play
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_runtime/src/scene/world/world.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
  - dev/godot/editor/run/editor_run.cpp
  - dev/godot/editor/run/editor_run_bar.cpp
  - dev/bevy/crates/bevy_tasks/src/usages.rs
tests:
  - 37 of 37 current Rust files reconciled and reviewed
  - 5128 physical lines and 52 inline tests
  - path plus physical-line-count plus per-file SHA-256 manifest fingerprint 4e26dbb2648ec10b882e606795368f8b04c7548a45a3a4ab57d3e67c0d7eac20
  - managed current-source Cargo and product WPR/xperf/RenderDoc/energy remain blocked by the non-runnable editor baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-15
---

# Editor Play/PIE lifecycle current architecture review (2026-08-15)

## Scope freeze and method

This review freezes `zircon_editor/src/core/play/**` at **37/37 Rust files, 5,128 physical lines
and 52 inline tests**. The manifest fingerprint is
`4e26dbb2648ec10b882e606795368f8b04c7548a45a3a4ab57d3e67c0d7eac20`; it is SHA-256 over sorted
`path|physical-lines|file-sha256` rows joined with LF. Two consecutive final inventory passes
produced the same fingerprint.

| Current module | Files | Physical lines | Tests | Static verdict |
|---|---:|---:|---:|---|
| root lifecycle and edit protection | 9 | 1,796 | 15 | one synchronous controller transaction is treated as both lifecycle authority and lock authority; terminal failures can leave the reported mode inconsistent with process reality |
| `backend/**` | 4 | 72 | 0 | synchronous start/stop/poll contracts permit arbitrary foreign work on the UI caller |
| `edit_policy/**` | 5 | 125 | 2 | compact constant-time policy types; retain behind the new session authority |
| `pending_edits/**` | 5 | 1,427 | 19 | capacity is bounded, but admission serializes payloads and repeatedly scans cohorts while outer lifecycle/protection locks remain held |
| `plugin_activation/**` | 5 | 200 | 1 | native activation is product-wired and nests another transition lock around plugin discovery/load/enter/exit work |
| `process_backend/**` | 5 | 1,244 | 12 | product-wired process start performs snapshot I/O and spawn synchronously; failure paths consume child/tree ownership before terminal proof |
| `snapshot/**` | 4 | 264 | 3 | full World projection/pretty JSON plus create/write/fsync/rename/delete remain synchronous lifecycle work |

Every current `.rs` file was read and assigned to the matrix below. Product reachability was traced
through editor event dispatch, menu Play/Stop, editor startup, retained-host tick/poll, runtime event
consumer startup and `World::clone`. Modified files were reviewed as current workspace source, not
as HEAD. Several source files contain foreign uncommitted work, so this pass made no Rust edit.

The approved-root defect in `tools/build-editor.ps1:130` still rejects valid D/E/F output roots
before Cargo. The current Pester result is 9 pass/6 fail from 15 tests. The latest managed editor and
runtime attempts therefore did not produce a current product executable. WPR 10.0.26100.8972,
xperf 10.0.26100.4188 and RenderDoc 1.44 are installed, but tracing an old binary would not validate
this source. Dynamic start latency, lock hold, RSS, power and GPU timings are `not_measured`; this
root must stay out of `review.md`.

## Architecture verdict

The P0 bottleneck is a structurally synchronous transaction assembled under the editor shell lock:

`Workbench shell lock -> World deep clone -> DynamicScene projection -> pretty JSON -> controller
transition lock -> native plugin discovery/load/enter -> edit protection -> snapshot mkdir/write/
fsync/rename -> process spawn/job attachment -> two output-reader threads`

This is not fixed by optimizing JSON formatting, increasing queue sizes or adding an outer worker
around the existing controller. Play state is split across the workbench state, controller mode,
plugin snapshot, edit protection, process backend, output readers, snapshot directory, runtime event
consumers and play-domain gateway. Several of those owners can reach terminal state while another
still reports `Playing`.

The required hard-cut chain is:

`WorldCommitGeneration -> PlayArtifactGeneration -> PlaySessionRequestGeneration ->
PlayPreparationTicket -> PluginRuntimeGeneration -> ProcessSessionGeneration ->
RuntimeConsumerGeneration -> PlayPresentationGeneration`

`PlaySessionAuthority` is the only lifecycle truth. It owns one monotonically increasing session
generation and explicit `Requested`, `Preparing`, `Starting`, `Running`, `Stopping`, `Terminal` and
`Failed` phases. CPU projection, I/O materialization, plugin transition and process supervision are
dependency-ordered tickets outside UI locks. Each completion commits only when its session and
input generations still match. Stop/cancel records intent in O(1), retains every cleanup resource
until a positive terminal receipt and exposes a degraded/failed phase when cleanup must be retried;
it never calls a stopped process `Playing`.

The UI event may capture only stable project/document/world generation handles while holding the
shell lock. It releases that lock before requesting the session. A process receives the same
immutable play artifact that embedded Play consumes; process materialization is one scheduled I/O
stage, not a second authority. No blocking I/O, plugin call, process wait, reader join or recursive
delete is permitted in a UI callback, transition lock or `Drop`.

## Current structures to preserve

- `PlayDomainLink` publishes a stable gateway handle and uses atomics for instance identity; its
  attach/detach operations can remain short generation swaps.
- mode-transition publication already occurs after the controller transition lock is released.
- process polling releases the active-child mutex before terminal finishing.
- Windows process spawn assigns the child to a process tree before normal execution resumes.
- output capture now bounds queue entries at 1,024, queue bytes at 4 MiB, line bytes at 64 KiB,
  live drain at 64 lines/256 KiB/2 ms and records dropped/truncated counts.
- pending edits bound total entries at 4,096, payload bytes at 4 MiB and age at 30 minutes; payload
  ownership is shared and apply callbacks execute outside the queue mutex.
- native plugin deactivation moves its snapshot and restores it on failure instead of cloning the
  snapshot blob.
- edit-policy target decisions are small and deterministic.

## P0 findings

### P0.1 UI lock contains the complete Play preparation and start path

`editor_event_execution/dispatch.rs:18-20` takes the workbench shell lock before dispatching the
menu action. `menu_action.rs:208-220` then requests `project_scene`, builds `PlaySceneSource` and
calls `request_play` without releasing it. `AuthoringWorld::try_snapshot` calls `Clone::clone` on
the live World. Current `World::clone` copies entity/kind/dynamic-component/type/resource/event/
message/observer/command state, builds persistent component snapshots and reconstructs entity and
component-storage projections. `PlaySceneSource::from_world` immediately constructs another
`DynamicScene`, then allocates pretty JSON.

The product path installs `ProcessPlayBackend::for_current_install` in
`editor_host_startup.rs:63-70`. Its synchronous `start` keeps the backend `active` mutex while
snapshot materialization performs `create_dir_all`, `write_all`, `sync_all` and `rename`, then
formats arguments and spawns the child. Native plugin activation is also installed by retained-host
startup and performs project plugin loading and runtime-mode entry first. Consequently a large
scene, slow storage, plugin loader or process creation stalls all workbench access.

Required correction: authoring mutations publish an immutable `WorldCommitGeneration`; a
generation-keyed play artifact is prepared once through Runtime11 CPU/I/O lanes. Enter Play only
publishes a request containing stable handles. The lifecycle authority advances after generation-
checked preparation receipts. Delete `project_scene -> World::clone -> from_world` from the UI
event path and delete snapshot materialization from synchronous backend `start`.

### P0.2 binary mode cannot represent partial or retryable terminal work

`request_stop` stops the backend and then deactivates plugins. If deactivation fails, `?` returns
before edit protection ends or mode changes to Edit. With the product process backend the process
has already been removed and stopped, so the controller reports `Playing` with no active process.
The same defect occurs after `poll` reports `Exited`: backend ownership is terminal before plugin
deactivation, yet a deactivation failure leaves the mode as `Playing`. Existing tests accept this
state for a no-op backend, masking the product inconsistency. UI error strings that say the runtime
remains active for retry are therefore not always true.

Required correction: phase state and resource receipts are separate. Process terminal, plugin
restore, runtime-consumer detach, edit-protection release and presentation restore each produce a
receipt owned by the same session generation. A failed cleanup advances to `Failed/CleanupPending`
with the exact retained resources. UI mode is a projection of that state, not the rollback
mechanism. Add exhaustive fault injection at every boundary and every ordering of stop/poll/cancel.

### P0.3 stop and terminal failure consume process ownership prematurely

`ProcessPlayBackend::stop` takes the child out of `active` before calling `PlayChild::stop`.
`PlayChild::stop(self)` consumes the child and first consumes its `ProcessTreeLease`. If tree
termination or `wait` fails, the error returns after backend ownership has already disappeared.
Rust `Child` drop does not prove termination or reaping; reader join handles can detach, while
`MaterializedPlayScene::drop` recursively removes the snapshot directory. In terminal poll,
`finish(self)` similarly consumes ownership; a failed tree termination skips reader joins but still
cleans the scene and returns an outcome.

Required correction: a `ProcessSessionTicket` retains process, tree, pipes and snapshot leases
until a positive reaped/closed/cleaned terminal receipt. Termination is retryable and deadline-
driven, with escalation recorded explicitly. Resource cleanup is an I/O ticket. `Drop` may only
enqueue/idempotently signal cleanup; it cannot block or erase the last observable owner.

### P0.4 lifecycle locks wrap synchronous foreign contracts

Controller `request_play`, build completion, stop and poll hold `transition_gate` across backend and
plugin calls. Edit routing also holds it while admission can serialize and scan. Native plugin
activation adds its own transition mutex around discovery, DLL loading, runtime entry/exit,
diagnostic aggregation and snapshot publication. Backend and plugin traits are synchronous, so the
lock contract admits unbounded foreign latency by construction.

Required correction: replace synchronous `activate/start/stop/poll` transitions with ticket
submission plus generation-checked completion events. Short state commits remain serialized by the
authority; work does not. Plugin affinity requirements must be explicit in the ticket rather than
implicitly satisfied by the UI caller.

## P1 findings

### P1.1 pending edit admission is bounded but not near O(1)

Every enqueue serializes the full invocation into a temporary `Vec<u8>` only to calculate retained
bytes. Latest/bounded paths then scan retry and pending deques for `find_cohort`, `cohort_count` and
`oldest_cohort`; summary/age work scans again. At the 4,096-entry cap, many distinct cohorts can
drive repeated O(N) admission and O(N squared) batch behavior. `PlayEditProtection::route` retains
its state mutex and the controller retains `transition_gate` while this happens. Apply has a
128-entry/2-ms outer budget, but one synchronous callback may exceed the budget because elapsed
time is checked only between callbacks.

Required correction: the operation owner supplies a validated retained-byte estimate or shared
serialized payload; queue state maintains typed cohort indexes and an O(1) oldest-age frontier;
pagination uses stable slots/cursors. Slow operations execute as cancellable job tickets and commit
results through the edit authority.

### P1.2 inactive product polling performs lock work every retained tick

The retained-host tick always calls `pump_runtime_event_consumers`, which always calls
`play_sessions.poll_backend`. In Edit mode this still acquires the transition mutex and reads mode
state before returning unchanged. During Play, each tick also polls the process and may spend the
bounded 2-ms/256-KiB output drain budget before other UI work.

Required correction: a session-generation wake/readiness source schedules process completion and
output deltas. At minimum, an atomic active-generation fast gate must make inactive ticks zero-lock;
the final design should use Runtime11's shared process/blocking-I/O owner rather than frame polling.
Measure before claiming the current inactive lock is a dominant wall-time hotspot.

### P1.3 output memory is bounded, but per-session threads and joins remain unbounded in time

The previous unbounded-line and unbounded-byte findings are fixed in current source. Remaining
costs are two private blocking threads per Play session, string rendering on the UI poll path and
reader `join` without a deadline. The output pump belongs on a shared blocking-I/O/process service
with per-session byte/age quotas and bounded terminal receipts. Do not remove the current line,
queue, byte, drain, truncation and drop limits during that migration.

### P1.4 snapshot durability policy is coupled to interactive start and cleanup

The process path always `sync_all`s an ephemeral snapshot before spawn and recursively deletes the
directory from both explicit cleanup and `Drop`. Whether an ephemeral Play artifact requires a
durable media flush is a transaction/durability decision, not a default UI-start requirement.
Runtime04 durable project commits and ephemeral Play materialization need separate policies and
metrics. Test fixtures must use approved D/E/F roots, not implicit `std::env::temp_dir` on C:.

## Complete per-file reconciliation

| Files | Review result |
|---|---|
| `controller.rs`, `mode.rs`, `request.rs`, `transition_report.rs` | synchronous transition authority, partial-terminal defect, request/mode allocation and diagnostic publication reviewed |
| `edit_protection.rs`, `edit_policy/{decision,mod,policy,target,tests}.rs` | lock scope, deterministic target policy, decision/prompt transitions and policy tests reviewed |
| `live_link.rs`, `error.rs`, `mod.rs`, `tests.rs` | gateway generation, errors/exports and all lifecycle/rollback/current test expectations reviewed |
| `backend/{contract,mod,noop,report}.rs` | synchronous trait boundary, no-op semantics and owned report allocation reviewed |
| `pending_edits/{intent,mod,queue,resolution,tests}.rs` | payload ownership, limits, cohort/age/page algorithms, retry/apply behavior and 19 tests reviewed |
| `plugin_activation/{contract,mod,native,noop,report}.rs` | synchronous boundary, nested locks, project loading, snapshot move/restore and report ownership reviewed |
| `process_backend/{child,command,mod,output,tests}.rs` | product start/poll/stop/drop, command arguments, process-tree/reap authority, pipe bounds/threads and 12 tests reviewed |
| `snapshot/{mod,source,store,tests}.rs` | persisted/snapshot sources, World projection/JSON, atomic materialization, fsync/cleanup and three tests reviewed |

DTO-only, error-only, module-export and test-support files are intentionally grouped in the table;
none was excluded from the 37-file manifest or source read.

## Hard-cut implementation plan

### A1. Define one session authority before adding concurrency

Owner: Editor04, with Plan02 M4 contract review. Define session/request/resource generation IDs,
phase graph, cancel/stop semantics, terminal receipt schema and presentation projection. Model every
failure edge. Delete binary-mode rollback assumptions and text claims that are not derived from
resource truth.

### A2. Publish immutable play artifacts from World generations

Owners: Runtime03/Runtime10/Runtime11 and Editor04. Publish generation-keyed scene artifacts from
the authoring World without cloning the complete runtime authority. Separate persistent authoring
data from runtime-only resources/events/queues. CPU projection and encoding run as dependency
tasks; unchanged generations reuse artifacts. UI requests carry handles only.

### A3. Move plugin and process preparation to explicit tickets

Owners: Plugins01, Runtime11 and Editor04. Native plugin discovery/load/enter/exit uses the stable
VM ABI/capability/state-migration contract and declared affinity. Snapshot materialization, spawn,
pipe read, poll, terminate, reap and cleanup use shared I/O/process tickets. Every ticket reports
generation, timings, bytes and retained resources.

### A4. Commit short transitions and hard-delete old authorities

Owner: Editor04. Commit task completions only under a short session-authority lock and only for the
matching generation. Drive runtime consumer/gateway/edit protection/presentation from that commit.
Delete synchronous backend/plugin transition traits, UI `World::clone` snapshot start, backend-owned
materialization, frame-polled inactive state and blocking cleanup in `Drop`. No alias, dual path or
compatibility shim survives.

### A5. Reindex pending edits and preserve current bounds

Owners: Editor04/Editor14. Add stable cohort and age indexes, owner-supplied byte accounting and
cancellable apply tickets. Preserve current count/byte/age/line/drain bounds and add invariant
counters before replacing containers.

## Measurement and acceptance

No absolute millisecond or watt claim is accepted from source inspection. Baselines must use the
same Windows machine, power mode, project, build profile and capture interval. Unreal/Godot source
establishes ownership and lifecycle design evidence; comparable product binaries and projects are
required before numerical engine-to-engine claims.

| Scenario | Required counters/traces | Acceptance |
|---|---|---|
| Enter Play, scenes 1/1K/100K entities and artifacts 1 KiB/64 MiB/1 GiB | World clone/projection/JSON owners and bytes; UI shell/session/backend/plugin lock wait/hold; CPU task span; write/fsync/spawn wall; RSS | UI-thread scene clone/serialization/file I/O/plugin load/spawn = 0; shell/session lock hold is O(1) generation publication; unchanged artifact work = O(1); changed work is proportional to changed authoring data plus emitted artifact bytes |
| Stop/crash/cancel/supersede with 0/10 ms/10 s injected stage latency and every stage failing | session phase/generation, retained process/tree/pipe/snapshot/plugin resources, cancellation/termination/reap/cleanup latency, stale completions | no stale completion commits; no resource loses its owner before terminal receipt; no state reports `Playing` after process terminal; UI lock wait excludes foreign latency |
| Edit mode and Play mode for 1/100K retained ticks at 30/60/120 Hz | poll calls, mutex acquisitions, process syscalls, output bytes/age/drop/truncate, UI tick CPU | inactive session poll locks/syscalls = 0; active work is wake/delta driven and bounded by declared per-frame apply budget |
| Pending edits 1/1K/4,096, payload 64 B/1 MiB/4 MiB | temporary serialized bytes, cohort/age/page visits, actual retained bytes, callback/ticket time | admission/index work amortized O(1); temporary full-payload encoding solely for size = 0; memory/count/age bounds never exceeded; one slow operation cannot consume the UI frame |
| Process output 1/1K/1M lines and 64 B/64 KiB/1 GiB newline-free input | threads/tasks, max decoded line, queued/deferred bytes, drops/truncation, drain and join/terminal latency | existing 64-KiB line, 1,024-line, 4-MiB queue and 2-ms/256-KiB drain ceilings preserved; private threads/session = 0 after Runtime11 integration; terminal wait is deadline-bounded |

Validation order after the approved-root blocker is fixed:

1. managed focused unit/property/fault-injection tests for the session phase graph, process ticket,
   snapshot store, pending indexes and stale generations;
2. managed current-source editor/runtime Cargo gates and product F4 Play start/stop/crash/cancel;
3. WPR/xperf CPU, disk I/O, context-switch, wait and working-set captures written only to D/E/F;
4. Windows energy/CPU package sampling over controlled idle/Edit/Play intervals;
5. RenderDoc first stable Play frame only after process launch and render are reproducible. It can
   correlate first-frame GPU work but cannot prove these CPU/lifecycle defects are fixed;
6. same-hardware comparison against declared reference workloads, with raw trace paths, warm-up,
   sample count and percentiles recorded. Never substitute remembered Unreal timings.

## Local reference evidence

- Unreal `PlayLevel.cpp:1002-1035` stores a Play request for the next tick and exposes explicit
  cancellation. `:1116-1128` clears the queued request after every attempt, preventing repeated
  starts. `:1138-1226` owns previous-session shutdown, save validation and destination branching
  inside one session flow; `:1419-1423` queues stop intent. Unreal still performs heavy work on its
  editor tick in places, so the transferable rule is explicit request/session authority and staged,
  profiled ownership, not a false claim that Unreal makes every stage asynchronous.
- Godot `editor_run.cpp:51-195` keeps process IDs and commits `STATUS_PLAY` after creation;
  `:222-231` centralizes process termination and status reset. `editor_run_bar.cpp:320-374` orders
  save/build/run and `:462-477` centralizes stop/debugger/UI reset. This corroborates one process
  owner and explicit stage order, not a numerical performance target.
- Bevy `bevy_tasks/src/usages.rs:52-75` distinguishes frame-critical compute, cross-frame compute
  and I/O task pools. Zircon's existing Runtime11 design should apply the same workload
  classification while preserving Zircon's single TaskGraph authority.

## Static disposition

- Static review is complete for the frozen 37/37-file manifest.
- No source optimization was made because the required correction crosses protected Editor04,
  Runtime03/11 and Plugins01 boundaries and current Play files contain foreign edits.
- The stale 2026-07-30 review is superseded for current-source decisions: process Play is now
  product-wired and output line/byte bounds are now present.
- Dynamic acceptance is blocked by the managed editor baseline. `pending.md` and `review.md` were
  not modified by this session; owner routing is recorded separately.
