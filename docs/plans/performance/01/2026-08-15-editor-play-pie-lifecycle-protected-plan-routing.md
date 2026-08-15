---
related_code:
  - zircon_editor/src/core/play
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_runtime/src/scene/world/world.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-15
---

# Editor Play/PIE lifecycle protected-plan routing evidence (2026-08-15)

## Coordinator decision

The current-source review is complete in
`2026-08-15-editor-play-pie-lifecycle-current-architecture-review.md`: 37/37 Rust files, 5,128
physical lines, 52 inline tests and manifest
`4e26dbb2648ec10b882e606795368f8b04c7548a45a3a4ab57d3e67c0d7eac20`.

Performance01 is authorized only for `docs/plans/performance/01/**`. The numbered runtime/editor/
plugin plans and foreign-dirty `performance/pending.md` and `performance/review.md` remain protected
by their owners. This record requests exact merges without bypassing those boundaries. It is not a
terminal failure handoff: non-validation architecture review continues while the product build
baseline is repaired.

## Required owner merges

### Performance main plan

Add `PERF-MVP-639` as P0 after PERF-MVP-638 and link it to PERF-MVP-627/631/632, Plan02 M1/M4,
Editor04/14, Runtime03/11 and Plugins01:

| ID | Priority | Current root cause | Required hard cut | Acceptance summary |
|---|---|---|---|---|
| PERF-MVP-639 | P0 | the product Enter Play event holds the workbench shell lock across a full `World::clone`, projection, pretty JSON, native plugin load/enter, snapshot create/write/fsync/rename and process spawn; controller/plugin/backend locks wrap synchronous foreign work; binary mode can report `Playing` after the process is terminal; stop/finish consume process-tree/pipe/snapshot ownership before positive terminal proof | Plan02 M4 plus Editor04/14, Runtime03/11 and Plugins01 hard-cut one generation-based `PlaySessionAuthority`, immutable `PlayArtifactGeneration`, dependency-ordered CPU/I/O/plugin/process tickets, explicit Requested/Preparing/Starting/Running/Stopping/Terminal/Failed phases and retained terminal cleanup receipts. Delete UI World clone/serialization, synchronous backend/plugin transitions, backend-owned materialization, inactive frame polling and blocking cleanup in Drop | frozen 37/37 static manifest retained pending. UI heavy work and foreign-latency lock hold = 0; unchanged preparation O(1); no stale completion or false Playing state; all process/plugin/pipe/snapshot resources retain authority until terminal; current Cargo, fault injection, 1/1K/100K scale, F4 WPR/xperf, RSS/energy and relevant first-frame RenderDoc correlation pass |

Do not implement this as another queue around `request_play`, a larger transition mutex or a second
Play state machine. The owner/generation/receipt contract lands before concurrency.

### Plan02 M1 and M4

M1 must expose the shared TaskGraph ticket/affinity/cancellation/completion and blocking-I/O/process
contracts needed by Play preparation. It must not create editor-private worker pools.

M4 must name `PlaySessionAuthority` as the sole Play lifecycle truth and define the complete phase,
generation, artifact, plugin-runtime, process-session, runtime-consumer and presentation contracts.
It must hard-delete synchronous Play transition traits and duplicate workbench/controller/backend
truth after dependencies land. No aliases, dual writes or compatibility shims survive.

### Performance pending/review indexes

Add one concise pending entry:

- `zircon_editor/src/core/play/**`: 37/37 static reviewed, 5,128 physical lines, 52 inline tests,
  manifest `4e26dbb2648ec10b882e606795368f8b04c7548a45a3a4ab57d3e67c0d7eac20`; architecture/fault/
  scale/product tracing pending.

Keep the module out of `review.md` until A1-A5, current-source managed Cargo, exhaustive lifecycle
fault tests, scene/pending/output scale counters, F4 process start/stop/crash/cancel, WPR/xperf,
RSS/energy and relevant RenderDoc correlation pass. Then move the module entry atomically from
pending to review. Do not add 37 per-file index rows.

### Editor04 Play-in-editor owner

Merge the following dependency-ordered work into the existing Editor04 lifecycle plan:

1. define `PlaySessionAuthority`, generation IDs, phases and exact terminal resource receipts;
2. change UI Play/Stop to publish O(1) request/cancel intent after releasing the shell lock;
3. consume immutable `PlayArtifactGeneration` handles instead of cloning/serializing World;
4. drive plugin/process/runtime-consumer/edit-protection/presentation changes from session commits;
5. retain resource truth through every partial start/stop/crash/cancel failure;
6. replace inactive frame polling with generation wake/delta delivery;
7. hard-delete controller binary rollback, synchronous backend/plugin traits and duplicate modes;
8. add deterministic stage fault injection and stale-completion tests before product wiring closes.

The current test expectation that a deactivation failure remains `Playing` is valid only for a
backend that is still running. Split it into resource-truth cases; a terminal process must project
`Failed/CleanupPending`, never `Playing`.

### Editor14 jobs owner

Provide cancellable Play preparation/apply/progress tickets through the shared Runtime11 TaskGraph.
Record queue wait, run wall, bytes, stage generation, cancellation latency and stale completion.
Pending-edit callbacks that can exceed a frame become resumable tickets; do not run them under
Play transition or protection locks.

### Runtime03 and scene-generation owners

Define the persistent authoring-data boundary used by `PlayArtifactGeneration`. The artifact path
must not clone runtime-only resources, events, messages, observers, command queues or rebuilt
indexes from `World::clone`. Publish changed authoring data by generation and reuse unchanged
artifacts. Coordinate this with existing World inspection/render extraction generation work rather
than adding another full-scene projection authority.

### Runtime11 task-system owner

Add or confirm shared lanes/contracts for cross-frame CPU scene projection, blocking snapshot I/O,
process spawn/supervision, bounded pipe decode and cleanup. Tickets carry affinity, deadline,
cancellation and completion generation. Do not create two threads per Play session or poll inactive
sessions every UI tick.

### Plugins01 owner

Move project discovery/load/enter/exit behind the stable VM ABI/capability/state-migration contract
and a session-generation plugin ticket. Keep plugin code/objects out of the UI lock and do not pass
Rust objects across dynamic library boundaries. A failed restore retains exact plugin-generation
authority and produces a cleanup receipt; it must not force the process lifecycle to lie.

### Runtime04 asset/project owner

Provide the durable project generation and artifact locator consumed by Play. Ephemeral Play
snapshot materialization has a separate durability policy from a project commit; do not impose
unconditional `sync_all` on the UI start path or publish a second project snapshot authority.

## Dependency and deletion gate

Required landing order:

`Plan02 M1 TaskGraph contracts -> Runtime03 authoring generation -> Play artifact producer ->
Plugins01/process tickets -> Editor04 session authority -> Editor14 presentation/pending jobs ->
old-path deletion -> dynamic acceptance`

The milestone is not complete while any product path performs one of the following:

- clones the complete World or emits pretty scene JSON under the workbench lock;
- loads/restores plugins, writes/fsyncs snapshots, spawns/waits processes or joins readers under a
  UI/session/backend transition lock;
- consumes the last process/tree/pipe/snapshot owner before a terminal receipt;
- reports `Playing` when the process is terminal;
- polls an inactive backend every retained tick;
- performs blocking recursive cleanup in `Drop`;
- keeps a compatibility alias or dual lifecycle authority.

## Validation routing

After the approved-root build defect is fixed, Editor04 owns managed focused and F4 product tests;
Runtime11 owns scheduler/process/I/O counters; Performance01 owns WPR/xperf, RSS/energy and
same-hardware reporting; render owners own the relevant first-stable-frame RenderDoc capture. All
artifacts must be written to approved D/E/F roots. Numerical comparison must declare workload,
hardware, power mode, build, warm-up, sample count and percentiles; local reference source alone is
not a timing baseline.

No commit or WeCom milestone notification is requested by this routing record. Those occur only
after the hard-cut milestone has accepted current-source dynamic evidence.
