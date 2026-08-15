---
related_code:
  - zircon_editor/src/core/hub_link
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/retained_host/app.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ProjectEditorRecords.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectEditorRecords.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-16
---

# Editor Hub link current-architecture review (2026-08-16)

## Status and scope freeze

- Result: `static_complete / dynamic_blocked`.
- Scope: `zircon_editor/src/core/hub_link/**`, **6/6 Rust files, 721 physical lines and
  3 inline tests**.
- Ordered path-and-raw-content SHA256:
  `e1eccf96c9abb19758af48d17bd7644079ae743ea90dd5f22cc7c16ec66671a0`.
- Every file was read in full. Production reachability was followed through project activation,
  startup restore, Welcome refresh/removal, retained-host ready publication and native attention.
- Code disposition: no Rust source changed. The complete module is foreign untracked work, and its
  owner plan is foreign dirty. This report freezes the observed snapshot without taking ownership.
- Accounting: keep the module in `pending.md`. It cannot enter `review.md` until current managed
  tests, two-process contention tests and F0/F1 WPR startup evidence pass.

The approved D/E/F editor build remains blocked in `tools/build-editor.ps1:130` before Cargo, so no
current editor executable exists for WPR/xperf. RenderDoc does not apply to this CPU/process-control
slice; it remains reserved for the renderer and first-presented-frame plans.

## Per-file review

| file | current-source performance result |
|---|---|
| `error.rs` | Typed focus-mailbox errors only; owned path/instance strings occur on failure. No hot-path issue. |
| `focus_signal.rs` | One atomic publish and atomic rename-claim per attention request. It does not poll. Multiple requests deliberately coalesce at one mailbox. Malformed/mismatched claims are retained without a count/age bound, so repeated bad input can accumulate diagnostic files. |
| `focus_watch.rs` | One non-recursive OS watcher survives for the active project. Directory events are filtered to the exact mailbox and only the first matching callback consumes it. This is the correct event-driven shape; dynamic evidence must count duplicate notify events, callback-thread latency and attention requests rather than replace it with frame polling. |
| `handshake.rs` | Ready/failure publication is a one-shot compact JSON atomic write after the retained host and focus watcher exist. No recurring work. The filesystem test correctly requires coordinator-managed `CARGO_TARGET_DIR`. |
| `mod.rs` | Protocol adapter wiring only. Liveness remains with `SessionGuard`, which avoids a second project-lock authority. |
| `recent_writeback.rs` | Every mutation takes a system-wide write lease, reads and validates the complete JSON document, mutates, validates again, pretty-serializes and atomically replaces it. The registry is bounded to eight rows, so its map/sort work is not the bottleneck. On Windows the caller can block in `WaitForSingleObject(INFINITE)`. There are no contention, abandonment, retry, shutdown or storage tests. |

## Current production chain and architecture verdict

Handshake and focus have the right baseline ownership:

`Hub launch token -> retained host initialization -> OS focus watcher -> ready atomic mailbox`.

The focus watcher is event-driven and project liveness stays exclusively in `SessionGuard`. No new
thread pool, polling timer or Hub-owned project lock is justified.

Recent-project state has the wrong transaction boundary. Successful project activation currently
runs:

`configure diagnostics -> apply plugin manifest -> acquire global recent lock indefinitely ->
read/decode/validate -> record/sort -> validate -> pretty encode/atomic write -> begin document
session -> commit SessionGuard`.

Therefore contention or a non-critical history-write failure can fail and roll back an otherwise
usable project. The same synchronous path serves Welcome removal and explicit refresh.

Automatic restore amplifies filesystem work. It reads the registry and, for every row, resolves the
path, checks existence/canonical identity and loads the manifest. After opening the selected project,
the open path writes the registry and `resolve_startup_session` reads and probes the complete list
again. `remember_prepared_project` and Welcome presentation also request fresh snapshots. The
eight-row cap bounds CPU collection work but does not bound cold, missing or remote-path latency.

The required authority chain is:

`ProjectAuthority/SessionGuard commit -> RecentProjectIntent -> bounded ordered projection lane ->
HubRecentProjectsGeneration -> affected-row health probe -> Welcome delta`.

Project availability and liveness are authoritative. Recent history is a recoverable projection:
its persistence failure must produce a diagnostic/retry receipt, not invalidate project activation.
The projection lane must keep cross-process read-merge-write serialization, but use a measured
deadline, finite queue/bytes/age, per-path coalescing and bounded shutdown flush. The immutable
in-memory generation updates once from the accepted intent, so the editor does not reread its own
write. Startup parses the bounded registry once; direct open of the newest candidate reuses the
authoritative project-open validation, while remaining row health is refreshed off the UI thread.

## Proposed PERF-MVP-643: non-blocking Hub history projection

Editor10/14/16 must implement one service, not a second job system or project authority:

1. separate project-session commit from recent-history persistence; successful project open remains
   successful when history persistence is delayed or rejected;
2. enqueue typed record/remove intents into one ordered lane, coalesced by canonical project key and
   bounded by entries, retained bytes and age;
3. perform cross-process read-merge-write under a measured finite lease deadline with explicit
   retry/backoff, abandonment and terminal receipts;
4. publish one immutable recent-project generation per accepted logical change and let UI consume
   borrowed snapshots/deltas without rereading the file;
5. parse the startup registry at most once per file generation, reuse project-open validation for
   the chosen restore candidate and schedule remaining health probes as affected-row work;
6. keep handshake and focus event-driven; add duplicate-event, bad-claim retention and callback
   latency counters without moving them into the frame loop.

## Reference-engine evidence and adaptation boundary

- Unreal `ProjectEditorRecords.h:35-55` defines `QueueUpdate` as the safe record-update entry: acquire
  a system-wide lock, load, mutate and save in a worker task.
- `ProjectEditorRecords.cpp:96-123` dispatches that work through TaskGraph and chains each update
  after the previous `AsyncUpdateTask`, preserving order without blocking the caller.
- `ProjectEditorRecords.cpp:126-135` waits only at explicit teardown. Its one-minute critical-section
  lease is evidence for a finite lock contract, not a Zircon latency target; Zircon must choose its
  deadline from two-process measurements and keep teardown bounded.
- `SProjectBrowser.cpp:807-894` marks project discovery with a CPU scope, snapshots recent metadata
  once and uses it while building/sorting the browser model. Zircon should retain the bounded shared
  document but must not repeat path/manifest probes for every snapshot request.

The Unreal record is an engine-association projection rather than Zircon's exact Hub DTO. The useful
evidence is the ordered asynchronous read-modify-write owner and explicit teardown boundary. Zircon
must reuse its Editor14 jobs, typed Hub protocol and `SessionGuard`; copying Unreal's global static
task or lock duration would be an unsupported design change.

## Measurement and acceptance gates

| gate | matrix | required result |
|---|---|---|
| startup snapshot | rows `0/1/8`; local warm/cold, missing and delayed paths | registry read/decode <=1/file generation; chosen valid project manifest load not duplicated; remaining probes off UI thread; Welcome uses generation/delta |
| write contention | writers `1/2/16`; hold `0/10/100/1000ms`; abandoned owner | project commit main-thread lock/I/O wait 0; finite deadline; ordered coalesced intents; retry/result entries, bytes and age bounded |
| focus/handshake | focus signals `1/100/10K`; duplicate directory events; malformed input | frame polling 0; one logical attention per coalesced signal; bad-claim files count/age bounded; ready only after watcher; callback p95 recorded |
| product | at least 31 comparable cold/warm F0/F1 runs after build repair | WPR/xperf CPU sampling, CSwitch, waits, file I/O, main-thread time, RSS and package power; compare a matched local Unreal project/browser flow with the same storage and power plan |

Static review is not acceptance. Current managed Cargo, deterministic delayed-filesystem fixtures,
two-process lease contention, product reachability and WPR/power evidence remain mandatory.

## Static gates executed

- Recomputed current inventory: 6 files, 721 physical lines, 3 inline tests and the unchanged
  ordered raw fingerprint
  `e1eccf96c9abb19758af48d17bd7644079ae743ea90dd5f22cc7c16ec66671a0`.
- `rustfmt --edition 2021 --check`: **6/6 GREEN**.
- Scoped `git diff --check`: GREEN. Both new documents have zero trailing-whitespace findings.
- Front-matter references: 15/15 paths exist.
- Repository documentation gate: RED with 671 existing violations across 241 of 2,502 documents;
  violations owned by the two new documents: **0**.
- Session coordinator plan audit and the `codex-performance-audit-20260814` heartbeat: GREEN.
- Managed Cargo, delayed-filesystem/two-process tests, product build, WPR/xperf and power: not run
  because the recorded approved-root build defect remains unresolved.
