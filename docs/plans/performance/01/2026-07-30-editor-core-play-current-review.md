# Editor core play current-source review

## Status

- Result: `source_review_complete / static_gate_pending / dynamic_pending`.
- Review date: 2026-07-30.
- Primary owners: Editor04 for play lifecycle, snapshot and pending edits; Editor14/Runtime11 for bounded CPU, I/O and pipe work; Runtime04/Runtime10 for the attached runtime world and snapshot generation; Plugins01 for native activation.
- Accounting: keep `zircon_editor/src/core/play/**` in `pending.md`. Do not add it to `review.md` before current-source managed Cargo, deterministic scene/lock/queue/output counters and F4 product evidence are GREEN.
- Code disposition: no Rust source was changed. The deleted legacy `bridge.rs`, tracked `mod.rs` change and all current untracked play modules/tests were reviewed and preserved. Four source files were actively leased by another session during final documentation, so this review used a source-bound current fingerprint and only leased document paths.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/play/**` | 37/37 | 3,958 | 34 | `044bda115e431f16f9e71d28efa74b385879c4710f1c35d42583f57a71a3123f` |

The fingerprint streams each native workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. All 37 files were read in full. During review, another owner extended `tests.rs`; the added host attachment/selection/undo test was reread and the inventory was rebound only after two stable hash passes. Production reachability was followed through retained-host startup, app runtime-gateway attachment, menu Play/Stop, retained backend polling, runtime-event consumers and pending-decision application.

## Per-file review

| file | current-source performance result |
|---|---|
| `backend/contract.rs` | Synchronous start/stop/poll trait makes every backend call foreign work on the controller caller. A future ticketed backend must change this owner rather than adding an outer UI queue. |
| `backend/mod.rs` | Module wiring and exports only. |
| `backend/noop.rs` | Constant-time embedded/default backend. It reports attachable and relies on the app/startup-attached runtime gateway; no process or I/O occurs. |
| `backend/report.rs` | Owned diagnostics DTOs. Per-frame running reports can allocate a vector supplied by the backend, but the no-op path returns empty. |
| `controller.rs` | One transition gate preserves ordering but remains held across plugin activate/deactivate and backend start/stop/poll, including snapshot I/O/process work for a process backend. Edit routing also holds it while pending admission serializes/scans payloads. Mode publication correctly occurs after releasing the gate. |
| `edit_policy/decision.rs` | Small decision enum only. |
| `edit_policy/mod.rs` | Module wiring and exports only. |
| `edit_policy/policy.rs` | Constant-time target routing; no queue or payload work. |
| `edit_policy/target.rs` | Compact target identity only. |
| `edit_policy/tests.rs` | Two matrix tests cover edit/play target routing, not performance. |
| `edit_protection.rs` | State and queue locks are separate, but `route` keeps the protection-state mutex while `enqueue` serializes and scans a potentially wide invocation; the controller transition gate is also held by its caller. Resolution callbacks run without these locks, with a small state guard preventing concurrent Play. |
| `error.rs` | Typed errors and formatting only. |
| `live_link.rs` | Stable gateway handle and atomic instance IDs avoid world copies. Attach/detach keep the attachment write lock while replacing the gateway generation; this must remain a short publication operation. |
| `mod.rs` | Module wiring and current hard-cut exports only. |
| `mode.rs` | Building mode owns a cloned `PlayStartRequest`; snapshot contents are `Arc<str>`, while paths and request metadata are copied. This is transition-frequency rather than frame-frequency work. |
| `pending_edits/intent.rs` | Invocation payload is shared through `Arc`, so failure reports and retries do not deep-clone JSON. Replacement drops the previous payload while the queue lock is held. |
| `pending_edits/mod.rs` | Module wiring and exports only. |
| `pending_edits/queue.rs` | Current queue is bounded to 4,096 entries, 4 MiB estimated payload and 30 minutes, supports latest/bounded retention, compact 128-row pages and 128-entry/2 ms apply budgets. Remaining costs are full `serde_json::to_vec` allocation solely to measure every enqueue, repeated O(N) age/cohort/summary scans and cursor paging from the front. One synchronous apply callback can exceed the elapsed budget, and expired work blocks new admission until explicit resolution. |
| `pending_edits/resolution.rs` | Compact summary/report DTOs; failure intents share their invocation payload. Applied/failure ID vectors remain bounded by the selected apply batch. |
| `pending_edits/tests.rs` | Eleven tests cover coalescing, FIFO/retry, bounded eviction, entry/payload/age rejection, policy wiring and resolution exclusion. No 4K/4 MiB algorithmic counters or slow-callback UI latency test exists. |
| `plugin_activation/contract.rs` | Synchronous foreign activation boundary. It permits DLL/project I/O and therefore cannot execute under the controller transition gate in the final design. |
| `plugin_activation/mod.rs` | Module wiring and exports only. |
| `plugin_activation/native.rs` | Successful deactivate moves the runtime snapshot and only restores ownership on failure, removing the prior full blob clone. Activate still performs project plugin load, runtime-mode entry, diagnostics aggregation/sort/dedup under nested activation and controller transition gates. |
| `plugin_activation/noop.rs` | Constant-time test/fallback activation. |
| `plugin_activation/report.rs` | Owned diagnostics/matrix report only. |
| `process_backend/child.rs` | Process polling is nonblocking and terminal cleanup occurs after the active mutex is released. Stop/reap and output-reader joins remain synchronous; an error consumes the `PlayChild`, potentially dropping handles/scene ownership without proving the child was reaped. |
| `process_backend/command.rs` | Builds one process command. Diagnostic argument rendering clones/lossily converts every argument and joins one string at start only. |
| `process_backend/mod.rs` | `start` holds the active mutex across snapshot materialization, argument formatting and process spawn. `stop` moves the child out before synchronous kill/wait. No product path currently installs `ProcessPlayBackend`; this remains a pre-integration gate. |
| `process_backend/output.rs` | Queue count is bounded at 1,024 and live drain at 64 lines, with nonblocking drop counts. `read_until('\n')` leaves one line and total queue bytes unbounded; every Play creates two private reader threads, and terminal finish joins both without a deadline. |
| `process_backend/tests.rs` | One argument contract test; no output, process failure, stop/reap or cleanup behavior is exercised. |
| `request.rs` | Start DTO shares scene snapshots via `Arc<str>` and clones paths at construction. |
| `snapshot/mod.rs` | Module wiring and exports only. |
| `snapshot/source.rs` | `from_world` synchronously builds a full `DynamicScene` and pretty JSON, creating multiple scene-sized owners. The production menu calls it while holding the workbench shell mutex before the controller request. |
| `snapshot/store.rs` | Snapshot materialization synchronously creates directories, writes all bytes, calls `sync_all` and renames. Process start holds its active mutex across all of this. Cleanup recursively deletes the owned directory and can run during stop/drop. |
| `snapshot/tests.rs` | Two tests cover snapshot roundtrip/cleanup and persisted-file ownership; no large-scene, syscall, cancellation or partial-write budget. |
| `tests.rs` | Fourteen lifecycle/gateway tests cover mode transitions, rollback, crash, publication and edit selection/undo preservation across play attachment. They use no slow foreign callback, scale or concurrent stop/poll test. |
| `transition_report.rs` | Transition DTO owns diagnostics; unchanged process polls can return up to the bounded line drain. |

## Corrected and remaining tasks

### PERF-MVP-550: current product still serializes the scene synchronously

The retained menu dispatcher takes the workbench shell mutex, clones the current scene, calls `PlaySceneSource::from_world` and only then requests Play. That performs full World to DynamicScene to pretty JSON work even for the default embedded/no-op backend. If the process backend is later installed, its `start` additionally holds `active` across directory creation, write, `sync_all`, rename and spawn. Publish a world-generation play artifact through the shared CPU/I/O scheduler, release the shell lock, and commit only a matching generation. Embedded play should pass a typed/immutable artifact; process play should materialize the same artifact once.

### PERF-MVP-551: admission is bounded; measurement and lookup remain expensive

The former unbounded queue/full snapshot finding is stale. Current source has global entry/byte/age limits, operation-owned lossless/latest/bounded retention, shared payloads, compact cursor pages and interactive count/time apply. Remaining admission serializes the full invocation to a temporary `Vec<u8>` solely to get its length while two outer locks are held. Cohort lookup/count/eviction, age validation and summary are linear over both deques; many distinct cohorts can approach O(N^2) at the 4,096 cap. Carry a validated retained-byte estimate with the deferred invocation, index typed cohorts and oldest age, and execute pending operations through a resumable ticket so one slow callback cannot exceed a UI frame budget.

### PERF-MVP-552: line count is bounded; byte and lifecycle budgets are not

The 1,024-line queue, 64-line live drain and drop counter are current. A newline-free child stream can still grow one `Vec` without bound before enqueue, while 1,024 large strings make total queue memory unbounded. Two private blocking threads are created for every process session. Move pipe decoding to the Runtime11 blocking-I/O owner with max line/queue bytes, truncation and oldest-age metrics; retain child/reap authority on every stop error and join readers with a bounded terminal protocol. The process backend is not currently product-wired, so these are integration gates rather than observed default-path timings.

### PERF-MVP-553: transition ownership must not equal lock ownership

The old foreign-work finding remains current. Controller requests and retained polling hold `transition_gate` across plugin DLL load/restore and backend work, while menu Play also holds the outer workbench shell mutex across scene serialization and the entire transition. Replace the lock-held transaction with a generation/token state machine: publish Starting/Stopping/Polling under a short lock, perform foreign tickets outside, then generation-check commit/rollback. Stop/cancel must signal the active ticket without waiting under UI locks.

## Acceptance plan

- Start: scenes `1/1K/100K entities` and serialized bytes `1KiB/64MiB/1GiB`, unchanged/changed/cancel/supersede. Count World/scene/JSON owners, cloned bytes, shell/transition/active lock wait/hold, serialize/write/fsync/spawn wall and RSS. UI-thread serialization/I/O and lock hold proportional to bytes must be zero.
- Pending edits: entries `1/1K/4,096`, payload `64B/1MiB/4MiB`, retention lossless/latest/bounded and age `0/30min`. Count temporary serialized bytes, cohort/age visits, summary/page visits, queue actual bytes and apply callback/frame time. Admission must be near O(1), actual memory hard-bounded and each frame obey count plus elapsed budget even for slow operations.
- Output/process: lines `1/1K/1M`, line bytes `64B/1MiB/1GiB`, poll `30/120Hz`, normal/error/kill-failure/reader-stall. Count reader resources, buffered bytes/age/drop/truncate, format work, stop/reap/join/cleanup latency. No child, reader or snapshot may outlive terminal ownership.
- Transition: foreign latency `0/10ms/10s`, concurrent play/stop/poll/route `1/16`. Record state generation, stale completions, lock wait/hold and cancellation latency. Foreign wall must not count as controller/workbench lock hold and activation/backend order must remain exact.
- Run current-source managed play, pending-edit, snapshot and host lifecycle tests; then F4 embedded start/stop/crash/pending apply-discard and, once explicitly wired, process start/stop/output. RenderDoc first-frame capture belongs to the render-owner F4 gate; no process backend or reproducible product capture was available in this CPU/control review.

## Reference check

- Unreal `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp` stores `FRequestPlaySessionParams` for the next tick and exposes cancellation before `StartQueuedPlaySessionRequest`. Zircon should similarly separate request publication from expensive start execution while retaining a single lifecycle authority.
- Godot `dev/godot/editor/run/editor_run_bar.cpp` centralizes run mode, stops an existing run before starting a replacement and delegates process ownership to `EditorRun`. Zircon's controller should preserve one transition authority but delegate blocking work to its existing scheduler/process owner.
- Bevy `dev/bevy/crates/bevy_tasks/src/usages.rs` distinguishes cross-frame CPU work (`AsyncComputeTaskPool`) from I/O-intensive work (`IoTaskPool`). Zircon should map scene serialization and snapshot/pipe I/O onto Runtime11's corresponding budgets, not create per-Play pools.

## Static gates executed

- Read all current 37/37 Rust files and the listed production caller chains, including the concurrently added host attachment test.
- The final source-bound `rustfmt --edition 2021 --check` is RED only in concurrently extended `tests.rs` (three assertion layouts at lines 121, 169 and 180). The source belongs to another active session, so this review records the gate without changing it.
- `git diff --check -- zircon_editor/src/core/play` passed; Git only reported the existing LF-to-CRLF checkout warning for `mod.rs`.
- Source inventory is 37 files, 3,958 physical lines and 34 inline tests at fingerprint `044bda115e431f16f9e71d28efa74b385879c4710f1c35d42583f57a71a3123f`, stable across two final passes.
- `review.md` remained unchanged. No managed Cargo, allocation/lock/RSS scale run, WPR F4 product trace, process run, RenderDoc capture or independent dynamic review ran.
