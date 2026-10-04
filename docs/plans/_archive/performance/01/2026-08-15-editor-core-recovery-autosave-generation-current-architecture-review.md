# Editor core recovery autosave-generation current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for production reachability, incremental dirty demand, payload memory admission,
  steady-state storage complexity and project-generation fencing; P1 for recovery startup scale and
  session heartbeat scheduling.
- Owners: Editor17 owns recovery/autosave semantics; Editor14 owns bounded background admission and
  save exclusion; Editor03 owns dirty generation; Editor16 owns project-session activation; Runtime11
  owns bounded durable file I/O; EditorUI08 owns retained tick integration; Render17 owns measurement.
- Accounting: keep `zircon_editor/src/core/recovery/**` in `pending.md`. Do not add it to `review.md`
  before current managed Cargo, crash/restart gates and F0/F4 WPR CPU/RSS/file-I/O/power evidence.
- Code disposition: no Rust source changed. Nine files are foreign modified and eleven present files
  or directories are foreign untracked current work. A one-line tick call would activate known P0
  scaling defects before their owner contracts are corrected.

## Exact scope

| scope | files | physical lines | tests | ignored | ordinal path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/recovery/**` | 20/20 | 5,009 | 54 | 0 | `57baefc04ab5e4d77cf9a4a1c91b3dee7a1d77f9b41cf5da72f8b8606078e781` |

The fingerprint streams every ordinal-sorted normalized workspace-relative path, NUL, raw file
bytes and NUL into SHA256. All 20 current Rust files were read in full. Production tracing included
dirty registry/toolkit projection, retained-host scheduling, UI and animation snapshot capture,
EditorJobSystem admission, project activation/session ownership and shutdown.

The 2026-07-30 3-file/824-line/10-test report is obsolete. The current tree has a bounded job
adapter, autosave service, recovery catalog, platform ownership lease, heartbeat/liveness layers and
split tests. Conversely, neither retained autosave polling nor restore-candidate flow is called by
production code today, and no production caller refreshes the session heartbeat.

## Current positive baseline

- `EditorAutosaveService` uses the unique `EditorJobSystem`; jobs are `Misc + Background`, use the
  foreground save mutex, capture bytes only after admission, and expose bounded completion pumping.
- Job admission is bounded by entries, estimated bytes and oldest age. Reservation happens before
  request/job/channel materialization, and a fairness cursor rotates a bounded selected window.
- Snapshot capture validates the Editor03 dirty generation immediately before serializing. It does
  not mark the source saved or mutate dirty facts.
- Session ownership uses an OS lease rather than PID or timestamp alone. Record parsing is strict,
  normal release reports durability failure, and residual state is preserved for explicit recovery.
- Snapshot publication uses same-directory temporary files, file flush, atomic replacement and
  parent-directory synchronization. Sequence reservation handles concurrent writers and stale crash
  markers; retention is bounded to three snapshots.
- Current tests cover admission races, mutex exclusion, cancellation, completion budgets, sequence
  collision/restart, metadata identity, recovery decisions and Windows/Unix lease behavior.

These are useful correctness foundations. They do not make the current production path reachable or
give it bounded CPU, memory and filesystem work in the dimensions below.

## Architecture verdict

Recovery currently has four different units of ownership: dirty generation, scheduler document ID,
job byte estimate and physical snapshot sequence. The first two are connected only by a due-time
full projection; the byte estimate describes a small request struct rather than the serialized
document; and the physical store reconstructs sequence/retention state by repeated directory scans.
There is no project-generation receipt that fences old work when project ownership changes.

The correct unit is an immutable `AutosaveGeneration` for one project generation and document dirty
generation. Editor03 change deltas create or supersede lightweight demand; Editor14 admits the
actual estimated/capped payload; Runtime11 writes it through one per-document durable lane; a small
manifest owns next sequence and fixed retention slot. UI tick only pumps bounded completions and
changed demand, never reconstructs all dirty/toolkit/path state.

## Structural bottlenecks

### P0: the production autosave path is unreachable

`poll_editor_autosave` exists only at its definition (`retained_host/app/autosave.rs:5`); the full
current Rust source has no call. Therefore no current product trace can show autosave CPU, memory or
I/O, and source-only tests cannot accept the feature. Adding the missing tick call alone is rejected:
it would activate the following full-scan, quadratic lookup and unbudgeted-payload behavior.

Restore catalog/`RestoreFlow` is likewise consumed only by recovery tests. `SessionGuard::claim` is
wired into project activation, but `refresh_heartbeat` has no production caller. The system is a
partially connected recovery architecture, not an optimized production workflow.

### P0: due scheduling reconstructs all dirty/toolkit/path state

`dirty_document_toolkits` snapshots all toolkits into a descriptor `BTreeMap`, then calls
`DirtyRegistry::changes_since(None)` and walks every dirty snapshot (`editor_manager_layout.rs:78-
108`). The retained autosave method resolves the physical source identity for every result and
materializes parallel `intents` and `documents` vectors before the bounded scheduler selects work
(`retained_host/app/autosave.rs:42-79`).

`AutosaveScheduler::plan_window` then scans all supplied states and inserts cloned IDs through two
bounded `BTreeSet`s (`autosave.rs:228-258`). Storage is bounded by window `W`, but due-time CPU is
`O(D log W)` for `D` dirty documents. Request construction searches the complete intents vector for
each selected ID (`retained_host/app/autosave.rs:80-88`), adding `O(W*D)` comparisons.

Editor03 already supplies a bounded `changes_since(cursor)` journal and immutable snapshots. The
recovery layer should maintain one autosave demand index from those deltas instead of creating a
second dirty authority or restarting from `None` at every interval.

### P0: admission bytes do not describe snapshot memory

The production estimate is `size_of::<AutosaveDocumentRequest>()` for every document
(`retained_host/app/autosave.rs:79`). The worker later calls toolkit capture, which returns a complete
`Vec<u8>` (`document_toolkit.rs:11-24`; `autosave_adapter.rs:415-432`). UI assets generate canonical
text while holding the UI-asset session lock; animations generate complete document bytes while
holding the animation session lock. With `W` running jobs, peak owned payload can approach the sum of
`W` document sizes even while the pending-byte budget reports only request structs.

Capture after admission is positive, but it does not make memory bounded. Admission must charge a
trustworthy document-generation estimate with a hard maximum or reserve a bounded streaming buffer;
the job must not own an unaccounted 1 GiB `Vec`.

### P0: every snapshot performs three directory scans and a metadata read/decode

For one admitted write, `next_sequence` enumerates the complete document directory
(`autosave.rs:356-380`), `reserve_sequence` calls `snapshot_sequence_exists` and enumerates it again
(`388-429,558-589`), and `rotate_document` enumerates it a third time while allocating owned names,
paths and `BTreeMap<u64, Vec<PathBuf>>` (`432-487`). `persist_source` reads and decodes
`recovery.json` every time (`autosave_catalog/catalog.rs:24-47`).

The algorithm is `O(E log E)` CPU/allocation plus three `read_dir` passes per snapshot for `E`
entries, even though the retention contract is exactly three. A durable per-document manifest or
fixed slot ring should load/reconcile once at project activation and make steady next-sequence,
metadata validation and rotation `O(1)` with zero directory scans.

The private `write_atomically`/`write_new_atomically` implementation also splits durability counters,
fault injection and streaming behavior from Runtime11. It must converge on the shared writer without
weakening flush, atomic replacement or crash-marker semantics.

### P0: project lifecycle has no autosave generation fence

Project synchronization stops admission and drops the old adapter, but started jobs may already be
capturing, allocating a sequence or writing. Cancellation is checked before and after capture, not
during sequence/metadata/write/rotation. Old snapshots stay in their old root, so this is not a
cross-project path escape, but completion and shutdown cannot state which project generation became
durable before the new project became authoritative.

Every demand, job, receipt and completion must carry the Editor16 project-session generation. Project
switch/close establishes a Runtime11 fence; stale generations may finish only according to an
explicit preserve-or-cancel policy and can never update current UI/service state.

### P1: recovery discovery is unbounded and fail-fast across the whole catalog

Startup enumerates every document directory; each document builds a `BTreeMap` of all snapshots only
to select the latest, then reads metadata and stats both source and snapshot
(`autosave_catalog/catalog.rs:50-110,148-189`). There is no entry, path, metadata-byte or wall-time
budget. One malformed directory or metadata error aborts the complete catalog, coupling one corrupt
document to all recovery candidates.

Use the manifest as the primary latest-snapshot index, verify in bounded pages, quarantine/report a
bad entry independently, and preserve a resumable cursor. Full disk reconcile belongs to explicit
repair/diagnostic work, not startup's synchronous critical path.

### P1: heartbeat API is synchronous but currently disconnected

`SessionGuard::refresh_heartbeat_at` clones/encodes and atomically replaces the lock record, including
flush and parent sync (`session_guard/guard.rs:125-135` plus `mutation.rs`). No production caller
exists, so there is no current frame hotspot, but wiring it to retained tick would create periodic
main-thread filesystem stalls. Editor16 should schedule a coalesced timer/demand on Runtime11 and
retain an exact receipt; interval selection must be measured against crash-detection policy.

## Required architecture hard cut

1. Editor03's dirty journal feeds a recovery-owned `AutosaveDemandIndex` by cursor. Each entry holds
   document ID, toolkit/source handle, dirty generation, project generation and an honest payload
   estimate. Stable ticks perform `O(0)` work; changes cost `O(C log D)` for `C` changed documents.
2. Due scheduling takes a bounded page directly from that index. Remove the all-toolkit descriptor
   map, all-dirty vectors, dual bounded sets and linear intent lookup. Fairness cursor and entry/age
   limits remain owned by Editor14.
3. Seal `AutosaveGeneration { project, document, dirty_generation }`. A newer dirty generation
   supersedes queued older work before capture. Capture/serialization remains under the same save
   exclusion but runs off UI; admission charges actual upper-bound bytes or a fixed streaming buffer.
4. Runtime11 owns one per-project/document durable lane with one running plus one latest pending
   generation. It exposes cancellation, fence, fault injection, write/flush/rename counters and exact
   terminal receipts. Recovery owns no private writer or thread.
5. Replace scan-derived sequence/rotation with a versioned, atomically durable manifest and fixed
   three-slot ring. Startup performs one bounded reconcile; steady writes perform zero directory
   scans and zero metadata reads. Stale reservations remain crash-recoverable.
6. Project switch/close and shutdown stop admission, fence the exact project generation, drain or
   explicitly cancel owned jobs, then release the session guard. Completion from stale generations
   cannot update the current service or status line.
7. Recovery discovery reads manifest pages under entry/path/metadata-byte/time budgets, isolates bad
   document entries and exposes a resumable repair path. Session heartbeat uses a coalesced Runtime11
   timer lane, never retained-frame synchronous I/O.
8. Only after source/static gates pass, connect retained tick. Its frame work is limited to bounded
   completion pumping and changed-demand consumption; no due-time all-document projection remains.

## Unreal primary-source comparison

- Unreal `PackageAutoSaver.cpp:1175-1218` updates `DirtyMapsForAutoSave`,
  `DirtyContentForAutoSave` and the user-restore set from package dirty-state callbacks, removing a
  package immediately when it becomes clean. `DoPackagesNeedAutoSave` then checks set counts in
  constant time (`1288-1300`). This is primary evidence for incremental dirty ownership instead of
  Zircon's due-time `changes_since(None)` reconstruction.
- Unreal chooses the next bounded backup slot as `(AutoSaveIndex + 1) % AutoSaveMaxBackups`
  (`PackageAutoSaver.cpp:350-357`) and passes that slot into map/content autosave (`369-383`). This is
  evidence for O(1) steady slot selection; Zircon still needs its stronger concurrent reservation,
  atomic durability and crash-reconcile guarantees.
- Unreal consumes the maintained dirty sets directly when autosave becomes eligible
  (`PackageAutoSaver.cpp:330-383`) and clears the relevant set after success. Zircon should retain
  generation-based supersession rather than copy Unreal's mutable package semantics, but should not
  rebuild all dirty state at each interval.
- Unreal performs some autosave work synchronously with a slow-task dialog in this source. It is not
  evidence that Zircon should move serialization or filesystem I/O to the UI thread. The useful
  reference is the maintained dirty set and bounded slot algorithm, not its threading model.

## Acceptance and measurement plan

| case | matrix | required result |
|---|---|---|
| dirty demand | documents 1/100/10K; changed per tick 0/1/100; due 0/1 | stable work=0; change work scales with `C`, not total `D`; due selection `O(W)`; full dirty/toolkit/path projection=0 |
| payload | docs 1/16; payload 1KiB/64MiB/1GiB; writers 1/4/16; capture stall 0/10ms/2s | queued payload=0; owned payload <= explicit byte/buffer cap; UI capture/serialization wall=0; same-document foreground save overlap=0 |
| storage | snapshots 3; directory entries/orphans 3/1K/100K; disk stall 0/10ms/2s | steady directory scans=0; metadata rereads=0; next slot/sequence O(1); final latest bytes and retention correct |
| lifecycle | project switch/close/crash at capture/write/flush/rename/manifest | stale-generation UI apply=0; every job terminal/fenced; guard release ordered; source digest, residual marker and recovery preserved |
| startup recovery | documents 0/100/10K; corrupt entries 0/1/10%; metadata 1KiB/1MiB | bounded page/bytes/time; one corrupt entry does not hide valid candidates; repair cursor resumes; synchronous open budget explicit |
| product | F0 project open/recovery and F4 idle/edit/autosave/close | current Cargo plus WPR CPU/thread/wake/lock/file-I/O p50/p95, allocation/RSS/package power and same-machine Unreal comparison GREEN |

Record dirty-journal entries/resyncs, demand count/age, selections/examinations, superseded generations,
estimated/actual/peak payload bytes, save-mutex wait/hold, sequence/manifest/reconcile work, directory
enumerations, write/flush/rename bytes/wall, project fences, stale completions, heartbeat work and
recovery page/corrupt counts. Algorithmic acceptance is stable `O(0)`, change `O(C log D)`, due
`O(W)`, steady slot/metadata `O(1)` and bounded payload memory. Source comparison alone does not
establish latency, power or parity.

RenderDoc is not applicable to this CPU/filesystem slice. WPR/xperf is required once the managed
editor launcher is available; the recorded approved-root separator defect currently blocks product
launch. No benchmark or trace artifact was written to C:.

## Per-file review

| file | current-source performance result |
|---|---|
| `autosave.rs` | Bounded interval/retention and crash reservations are positive. Due planning scans all input; each write performs three directory scans, metadata read and private atomic output. |
| `autosave_adapter.rs` | Admission reservation, lazy capture, fairness and completion budget are sound foundations. Estimated bytes do not cover payload; one batch single-flight lacks document-generation supersession/fence. |
| `autosave_catalog/catalog.rs` | Strict source identity is positive. Startup enumerates all documents/snapshots, builds BTreeMaps and fails the whole catalog on one bad entry. |
| `autosave_catalog/metadata.rs` | Small strict DTO; repeated steady read/decode is caused by store ownership, not this parser. Add manifest version/byte limits through Editor11. |
| `autosave_catalog/mod.rs` | Export/constants only; no independent hot work. |
| `autosave_catalog/source_path.rs` | Project-relative path validation is necessary and bounded by path length. Keep it in manifest/recovery admission. |
| `autosave_catalog/tests.rs` | Covers metadata and candidate semantics. Missing 10K/paging, corruption isolation and manifest-reconcile scale gates. |
| `autosave_service.rs` | Central service and bounded completion pumping are positive. State mutex spans orchestration; project generation/fence and ownership of shared job-system shutdown need hard contracts. |
| `mod.rs` | Export/test owner only; no independent runtime work. |
| `restore_flow.rs` | Deterministic maps/sets are acceptable for small startup sets. No candidate/resolution count or byte budget; production flow is not connected. |
| `session_guard.rs` | Module facade only; no independent hot work. |
| `session_guard/durability.rs` | Copy-sized durability result; no performance issue found. |
| `session_guard/error.rs` | Error DTOs only; no product hot work. |
| `session_guard/guard.rs` | OS ownership and explicit release are positive. Heartbeat is synchronous durable replacement and has no production scheduler. |
| `session_guard/liveness.rs` | Platform liveness is startup/recovery work, not a frame path. Keep PID evidence subordinate to the OS lease and instrument query latency/failure. |
| `session_guard/mutation.rs` | Correct durable lock mutation but duplicates per-write encode/stage/flush/replace work outside Runtime11 counters. |
| `session_guard/ownership_lease.rs` | Named mutex/flock provides constant-size OS arbitration; no polling loop found. Cross-process contention and abandoned-owner fixtures remain required. |
| `session_guard/record.rs` | Strict small record codec; heartbeat clones/encodes the record per refresh. Share bounded versioning/error limits with Editor11. |
| `tests.rs` | Strong store/guard/restore correctness coverage. Missing production reachability, project-generation fencing, steady scan counters and crash matrix under the shared writer. |
| `tests/autosave_adapter.rs` | Strong admission/race/fairness/completion tests. Estimated-byte fixtures model requests, not real 1KiB/1GiB payload ownership or streaming/RSS limits. |

## Static gates executed

- Read 20/20 current recovery Rust files in full and traced all production consumers, dirty/toolkit
  capture, job admission, project activation, shutdown and Unreal PackageAutoSaver sources.
- Ordinal current-source inventory is 20 files, 5,009 physical lines, 54 tests, zero ignored, with
  the fingerprint recorded above.
- The full current Rust source contains no production call to `poll_editor_autosave`,
  `refresh_heartbeat` or recovery-candidate/`RestoreFlow` consumption.
- `tools.tests.test_editor17_recovery_test_ownership_contract` ran 3 tests: 1 passed and 2 failed.
  `tests.rs` is 810 lines and `tests/autosave_adapter.rs` is 1,023 lines, exceeding the explicit
  800-line owner threshold. The deterministic current-source drift is recorded in
  `failure-2026-08-15-recovery-test-owner-threshold-drift.md`; no tests were deleted or weakened.
- Per-file `rustfmt --edition 2021 --check` passed 17/20. Foreign current `mod.rs`, `tests.rs` and
  `tests/autosave_adapter.rs` fail; the same failure record routes their owner cleanup.
- Managed Cargo, F0/F4 WPR and product file-I/O capture remain blocked by the recorded build-helper
  approved-root separator defect. RenderDoc was correctly excluded from this non-render slice.
- Protected plans/indexes were not modified. This static review is not an accepted milestone, so no
  commit or WeCom notification is due.
