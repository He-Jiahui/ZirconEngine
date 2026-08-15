---
related_code:
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/core/play/process_backend
  - zircon_editor/src/ui/host/export_process_support
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/output_tail.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformProcess.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/MonitoredProcess.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/MonitoredProcess.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/PlatformProcess.h
  - dev/bevy/crates/bevy_tasks/src/usages.rs
tests:
  - 10 of 10 current Rust files reconciled and reviewed
  - 2927 physical lines and 16 inline tests
  - path plus physical-line-count plus per-file SHA-256 manifest fingerprint 871709381a80334aabe34b65ee347b21b8470c7fb14f06193c081f31e21aca2e
  - managed current-source Cargo and product WPR/xperf/energy remain blocked by the non-runnable editor baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-15
---

# Editor process supervision/output current review (2026-08-15)

## Scope freeze and method

This review freezes the shared editor process owner and its two current product consumers at
**10/10 Rust files, 2,927 physical lines and 16 inline tests**. The manifest fingerprint is
`871709381a80334aabe34b65ee347b21b8470c7fb14f06193c081f31e21aca2e`; it is SHA-256 over sorted
`path|physical-lines|file-sha256` rows joined with LF.

| Current slice | Files | Lines | Tests | Static verdict |
|---|---:|---:|---:|---|
| `core/process.rs` | 1 | 658 | 4 | Play has a persistent Windows Job Object, but spawn resumes by scanning all system threads and termination consumes the lease before terminal proof |
| `ui/host/export_process_support/**` | 4 | 565 | 4 | 64-KiB file reads and typed errors exist; temporary capture uses the host temp directory and terminal drain can aggregate all remaining output |
| `ui/host/export_cargo_process.rs` | 1 | 284 | 2 | process polling occupies a worker and complete stdout/stderr are duplicated from disk into Vec and String |
| wizard execution/output slice | 4 | 1,420 | 6 | output tail is now O(1) and bounded, but process output is written to temporary capture and then written again to durable artifacts while completion remains synchronous |

The reviewed wizard files are `execution.rs`, `execution/core_pipeline.rs`,
`execution/output_capture.rs` and `output_tail.rs`. All current source was read in full. Product
callers were reconciled through Play process start/stop and both Cargo/export wizard process loops.
Current foreign edits in `process.rs`, wizard execution/output capture and output tail were included
in the fingerprint and preserved. This pass made no Rust edit.

The managed editor baseline remains blocked by `tools/build-editor.ps1:130`, which rejects valid
D/E/F output roots before Cargo. WPR/xperf cannot produce current-source product evidence until that
owner fixes the blocker. Dynamic spawn, cancellation, output, RSS and energy values are therefore
`not_measured`; none of these files are accepted into `review.md`.

## Architecture verdict

Zircon currently has two process-tree authorities:

- Play creates a suspended Windows process, attaches a new kill-on-close Job Object, finds a thread
  through a system-wide Toolhelp snapshot, resumes it and retains `ProcessTreeLease`;
- export/Cargo starts a normal process, configures a Unix process group only, and on Windows spawns
  `taskkill /PID ... /T /F` later. It retains only `Child` plus an armed drop guard.

Output has two more authorities: temporary capture files used as pseudo-pipes and durable wizard
artifacts. Cargo copies the temporary files into complete in-memory vectors and then complete
strings. Wizard copies the same bytes from temporary files into durable files while hashing,
decoding, retaining a tail and emitting UI deltas.

The required hard-cut chain is:

`ProcessRequestGeneration -> PlatformSpawnTicket -> ProcessSessionGeneration ->
BoundedOutputDelta + CanonicalOutputArtifact -> TerminationReceipt -> ReapReceipt ->
PipeCloseReceipt -> ArtifactCleanupReceipt`

One Runtime11-owned `ProcessSupervisor` owns process, primary thread, tree/job/group, stdout/stderr,
canonical output artifacts, cancellation deadline and terminal receipts. Editor Play, Cargo and
wizard export hold typed session handles. Platform spawn returns all native handles atomically;
Windows assignment uses the primary thread handle returned by `CreateProcessW`, not a global thread
enumeration. Process work uses a shared blocking process/I/O lane or OS readiness/completion, not a
sleeping general-purpose worker and not private per-session threads.

## Current structures to preserve

- Play assigns a Windows Job Object before the suspended child is resumed, preventing normal child
  execution before tree ownership exists.
- the Job Object is configured with kill-on-close rather than relying only on parent PID traversal;
- Unix commands use a dedicated process group and treat `ESRCH` as already terminal;
- export cleanup retains typed primary and cleanup termination errors;
- output capture yields each live read at 64 KiB and supports a one-worker nested-join test;
- wizard output persists complete stdout/stderr with byte count and BLAKE3 digest;
- wizard line decoding limits decoded line chunks to 16 KiB;
- current foreign work changed the 512-line tail to `VecDeque`, removing repeated front shifts, and
  bounds terminal vector projections at the output boundary;
- cancellation is checked before process launch and during process monitoring.

## Findings

### P0.1 process/tree/pipe ownership is split and terminal APIs are not retryable

`ProcessTreeLease::terminate(self)` and Windows `JobObject::terminate(self)` consume the only tree
owner. Play takes its `PlayChild` out of backend state before invoking this path. If termination or
reap fails, no retryable session owner remains even though the process, descendants or inherited
pipes can still be live. Export has the opposite accidental retry mechanism: an error path calls
`terminate_process_tree`, then the armed guard may invoke it again from `Drop`; this is not a typed
terminal state and can block an arbitrary dropping thread.

`ProcessTreeTermination.succeeded` means a termination operation was accepted, not that child reap,
descendant exit, pipe EOF and artifact cleanup have all completed. Consumers separately call
`wait`, and only for the direct child. Required correction: process state retains each native
resource until explicit, independently observable receipts complete. Failed termination remains a
retryable `Terminating/Failed` session; dropping a handle submits idempotent cleanup but never waits.

### P0.2 terminal output drain defeats the 64-KiB live budget

`read_available` allocates at most 64 KiB per stream per call, which is useful. However,
`final_output_drain` loops to EOF while extending one `CapturedOutputChunk`. If a fast child writes
large output and exits before the monitor catches up, completion allocates all remaining stdout and
stderr before returning. The live chunk size is therefore not a terminal working-set bound.

Cargo then appends chunks into complete `stdout`/`stderr` vectors and converts each to a new String.
Wizard writes chunks into durable logs, but terminal drain still materializes the remaining bytes
first. Required correction: the canonical output sink owns streaming writes and hash state; live
and terminal draining use the same bounded chunk callback. Results contain locator, digest, byte/
line/drop counts and a bounded tail, never the complete payload.

### P0.3 wizard output traverses and writes the same bytes twice

`create_output_capture` redirects the child to two temporary regular files. Poll reads those files
and `ExportWizardOutputCapture::record` writes the same bytes to two durable artifact files while
hashing and decoding. This doubles write traffic and later adds read traffic. `finish` flushes and
`sync_all`s each stream and the manifest. Core reports then clone tail vectors into pretty JSON and
perform another staged `sync_all`/replace.

Required correction: create the declared canonical artifact once and give its writer to the child
when the platform permits. A bounded tail/hash decoder observes the same stream without a second
full byte copy. If teeing is required, it is one supervisor-owned streaming stage with measured
backpressure and a single durability policy.

### P0.4 process monitoring parks general-purpose workers

Cargo polling sleeps 100 ms per iteration; wizard polling sleeps 25 ms. These loops run inside
editor job execution and repeatedly submit nested reads/poll work. Every active long-lived child
therefore occupies a worker while mostly sleeping and wakes at 10 or 40 Hz. This competes with
scene, import, preview and other jobs and scales with process count.

Required correction: Runtime11 provides a process/blocking-I/O lane with OS wait/readiness or a
small shared supervisor. Session completions publish typed deltas. General CPU workers do not sleep,
wait on process handles or perform terminal drain.

### P1.1 Windows Play spawn is O(total system threads)

The Play command is created with `CREATE_SUSPENDED`, but Rust `std::process::Child` exposes no
primary thread handle. `resume_initial_thread` therefore calls `CreateToolhelp32Snapshot` for all
threads, scans until it finds the process ID, opens that thread and resumes it. Spawn latency and
system work depend on unrelated processes and thread count.

Required correction: the Windows platform spawn implementation owns `PROCESS_INFORMATION` from
`CreateProcessW`, assigns its process handle to the Job Object and resumes the returned primary
thread handle directly. Measure spawn against 1/1K/10K unrelated system threads; unrelated thread
visits must be zero.

### P1.2 Windows export termination launches another synchronous process

Export's Windows configuration is a no-op at spawn. Cancellation later invokes `Command::new(
"taskkill").output()`, incurring PATH lookup, process creation, captured output and synchronous wait
for every attempt. Tree containment starts only at cancellation, so it cannot provide the same
ownership guarantee as Play's pre-resume Job Object.

Required correction: all editor child processes use the same platform spawn/session owner. Normal
cancellation calls the retained native job/group handle; external `taskkill` is not the product
steady path. A separately declared emergency diagnostic tool may exist only as an observed fallback.

### P1.3 product and test artifacts default to C:

`create_output_capture` and multiple process tests call `std::env::temp_dir()`. On the current
Windows environment this normally resolves to C:, violating the explicit artifact-location rule
and making performance evidence depend on an uncontrolled volume. Product APIs must receive an
approved project/build/cache artifact root. Tests must allocate under an approved D/E/F validation
root supplied by the harness.

### P1.4 current tests overstate structural coverage

Two of four `core/process.rs` tests only inspect source strings. The functional Windows test covers
one suspended child and the export guard covers one parent/descendant path, but there is no
termination failure injection, retry, root-exits-first, detached descendant, inherited-pipe stall,
concurrent process storm, terminal output storm, worker occupancy or approved-root assertion.

## Complete per-file reconciliation

| Files | Current review result |
|---|---|
| `core/process.rs` | typed errors, process group/Job Object, suspended attach/resume, taskkill fallback, consume-on-terminate semantics and four tests reviewed |
| `export_process_support/{child_guard,error,mod,output_capture}.rs` | Drop cleanup, typed cleanup cause, API exports, temp-file capture, nested join, 64-KiB reads, terminal drain and four tests reviewed |
| `export_cargo_process.rs` | pre-cancel, spawn, 100-ms monitor, termination/reap, full Vec/String accumulation and two tests reviewed |
| wizard `execution.rs` | process loop, 25-ms polling, error cleanup, output/durable capture, stage result and report projection reviewed |
| wizard `execution/core_pipeline.rs` | resume report read/pretty write/fsync/replace, command DTO duplication and tail rejoin reviewed |
| wizard `execution/output_capture.rs` | second full output write, BLAKE3, 16-KiB decoder, 512-line deque tail, fsync/manifest and three tests reviewed |
| wizard `output_tail.rs` | current O(1) deque eviction, terminal Vec adapter and three tests reviewed |

## Hard-cut implementation plan

### B1. Freeze the cross-product process session contract

Owners: Runtime11 and Plan02 M1, reviewed by Editor04/14/15. Define `ProcessSpec`, session/
generation IDs, platform spawn result, tree policy, output policy, cancel/deadline/escalation and
the complete terminal receipt. One session must support Play, Cargo and wizard without product-
specific process ownership.

### B2. Implement platform-owned spawn and termination

Owner: Runtime11 platform/process lane. Windows directly retains process, primary thread and Job
Object handles through create/assign/resume. Unix retains process-group/session identity. Remove
system-thread discovery and normal `taskkill` spawning. Make termination retryable and reaping/
pipe closure separately observable.

### B3. Replace temporary relay files with one output artifact stream

Owners: Runtime11 and Editor15. Stream stdout/stderr to the canonical artifact/hash/tail sink in
bounded chunks. Publish UI deltas under a count/byte/time budget. Cargo and wizard results carry
locators/digests/tails. Delete complete Vec/String results, terminal aggregation and duplicate full
temporary-to-durable writes.

### B4. Hard-cut callers and delete blocking cleanup

Owners: Editor04/14/15. Migrate Play, Cargo and wizard to typed process tickets. Remove sleep-poll
loops, `ExportProcessChildGuard` blocking Drop, synchronous process/tree helpers and private Play
reader threads. Compatibility wrappers and dual process authorities do not survive.

## Measurement and acceptance

| Scenario | Evidence | Acceptance |
|---|---|---|
| spawn 1/100 sequential and concurrent with 1/1K/10K unrelated Windows threads | WPR/xperf process/thread create, Toolhelp calls, worker queue/occupancy, p50/p95/p99 | unrelated thread visits = 0; spawn work O(1) in unrelated system thread count; no general CPU worker is parked for child lifetime |
| output 1 B/64 MiB/1 GiB, slow/fast producer, fast exit, no newline | peak RSS, allocation high-water, bytes read/written, artifact bytes/digest, tail/drop/truncate, UI delta queue | peak in-memory output is a declared constant plus bounded chunks/tail; terminal does not aggregate remainder; exactly one canonical full-output write per stream |
| cancel/kill failure/reap failure/pipe stall/root-exits-first/descendant tree | session generation/phases and every resource receipt; cancel and escalation latency | resource owner retained until terminal; retry works; stale receipt cannot close a new session; Drop wait/I/O/process spawn = 0; direct and descendant processes/pipes terminate or remain explicitly cleanup-pending |
| idle 0/1/100 process sessions for 60 s | worker active/sleep/park, wake rate, context switches, CPU package energy | inactive sessions wake = 0; live process monitoring does not consume general worker capacity; wake rate is readiness-driven rather than 10/40 Hz per process |
| approved-root product and tests | all opened/written artifact paths | output, fixtures and traces stay on approved D/E/F roots; C writes = 0 |

Managed Cargo, F4 Play, export product tests and WPR/xperf/energy captures run only after the build
root blocker is fixed. RenderDoc is not an acceptance tool for this CPU/process owner; it is relevant
only to the first stable Play render frame owned by the render plans.

## Local reference evidence

- Unreal centralizes typed creation in `FWindowsPlatformProcess::CreateProc` and
  `FProcessStartInfo` (`WindowsPlatformProcess.cpp:699-726`). Its Windows implementation owns
  `PROCESS_INFORMATION`, explicit inherited-handle lists and process handles rather than asking
  each consumer to assemble a separate subprocess protocol.
- Unreal `FMonitoredProcess` (`MonitoredProcess.cpp:102-133`, `:181-269`) owns process handle, pipes,
  cancel state, output and completion/cancel delegates in one lifecycle object. This supports one
  session authority and ordered pipe/process cleanup.
- Unreal also uses per-process monitor threads and its `TerminateProc(..., KillTree)` traverses a
  process snapshot (`WindowsPlatformProcess.cpp:939-980`). Those are reference limitations, not
  algorithms to copy. Zircon's shared Runtime11 task/process owner must avoid private threads,
  system-wide polling and unbounded output.
- Bevy's task usages distinguish cross-frame CPU from I/O work. Zircon should apply that workload
  classification through its own single TaskGraph rather than occupy compute workers with process
  sleep-poll loops.

## Static disposition

- Static current-source review is complete for the frozen 10/10-file manifest.
- No source edit was made: the correction spans shared Runtime11 and protected Editor04/14/15
  boundaries, and four reviewed files contain foreign work.
- The July 17 host review remains useful for history but its 5-file export summary is superseded by
  this current 10-file process/output reconciliation.
- Findings strengthen existing PERF-MVP-080/091/639; no duplicate performance ID is created.

