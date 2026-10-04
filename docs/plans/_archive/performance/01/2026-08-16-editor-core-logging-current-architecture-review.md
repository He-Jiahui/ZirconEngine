---
related_code:
  - zircon_editor/src/core/logging
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/script_build/diagnostics_sink.rs
  - zircon_editor/src/ui/activity/view.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceFile.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Private/Presentation/MessageLogListingViewModel.cpp
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Private/UserInterface/SMessageLogListing.cpp
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-16
---

# Editor core logging current-architecture review (2026-08-16)

## Status and scope freeze

- Result: `static_complete / dynamic_blocked`.
- Scope: `zircon_editor/src/core/logging/**`, **13/13 Rust files, 1,889 physical lines and
  16 inline tests**.
- Ordered path-and-raw-content SHA256:
  `946a635610fa1756e280c8dfae32433d67c70f504258e1596d6f2d3ac80a99e0`.
- Every current file was read in full. Production flow was followed through EditorContext message
  publication, project-scoped file setup, script diagnostics, editor/chrome/console snapshots,
  retained-host errors and jump actions.
- Code disposition: no Rust source changed. `service.rs`, `store.rs` and `tests.rs` are foreign dirty
  work adding clear/resync behavior. The root defects cross Editor02/13/14/17, EditorUI08 and
  Runtime11; changing the rolling sink alone would preserve the wrong synchronous contract.
- Accounting: keep the module in `pending.md`. It cannot enter `review.md` until managed current
  tests, scale/allocation counters and F0/F4 WPR file-I/O/wait/RSS/power evidence pass.

The approved D/E/F editor build remains blocked in `tools/build-editor.ps1:130` before Cargo. The
module tests also create fixtures through `std::env::temp_dir()`, which is not approved under the
no-C-artifact rule; they were not executed. RenderDoc is not applicable to this CPU/I/O/UI-data
slice.

## Per-file review

| file | current-source performance result |
|---|---|
| `config.rs` | Entry and logical-byte limits bound both memory store and event queue. The budget counts payload estimates, not retained struct/Arc/queue overhead or writer buffers, so RSS still needs a measured hard bound. |
| `entry.rs` | Message size is capped at 8 KiB and shared as `Arc<str>`. Construction first owns a `String`; byte estimates omit allocation/control-block overhead. |
| `error.rs` | Typed failures only. Persistence errors are converted to owned strings per failed emit in the service. |
| `filter.rs` | Six-channel `BTreeSet` membership is bounded. Filter cost becomes expensive only because consumers repeatedly scan and clone the complete store. |
| `jump.rs` | Typed shared jump targets avoid a second path owner. Construction and display formatting allocate only when created/rendered. |
| `mod.rs` | Module wiring only. |
| `record.rs` | Disk formatting converts source/jump to strings, performs three whole-string replacement passes for each field and allocates the final line. It currently runs on the producer thread for every persisted record. |
| `rolling_file.rs` | Every record calls `create_dir_all`, gets system time, formats the line, takes a state mutex, calls `metadata`, opens the file, writes and flushes. Segment discovery can repeat metadata while advancing. The file handle and tracked byte count are not retained. |
| `service.rs` | One global `emission` mutex covers sequence/store mutation and all rolling-file work. Thus all logging threads serialize behind filesystem latency. The first emitter also synchronously drains the event queue and invokes the sink until empty; this is a bounded reentrant dispatcher, not asynchronous delivery. Clear/resync discontinuity is now explicit and useful. |
| `severity.rs` | Compact ordered enum only. |
| `source.rs` | Six typed channels and shared plugin IDs are appropriate. Source display formatting occurs again for file/UI materialization. |
| `store.rs` | Count/estimated-byte eviction and monotonic sequences are sound. `snapshot` scans under the mutex and clones every match; sequence lookup linearly scans up to 2,048 rows. There is no immutable generation, cursor, range query or visible window. |
| `tests.rs` | Covers retention/filter/jump, rolling semantics, sequence/file order, reentrancy, queue byte/entry backpressure, resync retry, clear and I/O failure. It does not measure per-record filesystem calls/flushes, producer wait, allocation/RSS, million-record bursts or stable UI rebuilds. Fixtures use the system temp directory. |

## Current production chain and architecture verdict

The producer path is:

`producer formats LogEntry -> emission mutex -> ring push/record clone -> create-dir/time/line
format/metadata/open/write/flush -> event queue -> producer drains sink -> message bus JSON publish`.

Disk work is fully serialized inside the emission lock. Script build projects diagnostics in a loop,
so a diagnostic burst performs one synchronous flush per diagnostic. A slow filesystem blocks every
producer regardless of source; a slow event sink blocks whichever thread became dispatcher.

The event projection is currently disconnected from product UI invalidation. Current-source search
finds `EditorTopic::log()` subscriber registrations only in `core/context/builder.rs` tests. The
production retained UI reads `EditorLogService` directly, while every emit still clones the topic,
allocates a JSON value and publishes a bus message.

The read side is also a structural hotspot. `editor_snapshot`, `chrome_snapshot` and direct console
access each call `activity_log_console_output`, which:

1. locks and scans the complete retained ring;
2. clones all matching `LogRecord`s;
3. clones them again into `ActivityLogView`;
4. escapes and formats every row into a temporary `Vec<String>` and joins one complete text blob;
5. allocates parallel level and jump-sequence arrays.

This happens whenever an unrelated caller asks for the broad editor/chrome snapshot, not only when
the log generation, filter or visible console window changes.

The required chain is:

`LogIngressRange -> LogStoreGeneration -> bounded diagnostics-writer batches ->
LogPersistenceReceipt -> FilteredLogWindowGeneration -> RetainedConsoleRowDelta`.

The memory ring remains the immediate authority. Producer admission assigns ordered ranges and does
only bounded memory work; disk formatting/I/O and UI formatting are separate consumers. The writer
keeps the active segment open, tracks its bytes, batches writes and flushes at explicit byte/age,
error/fatal and shutdown boundaries. It must use one recursion-safe Runtime11 diagnostics I/O lane,
not submit logging back through a job path that itself logs.

The UI consumes immutable record handles by generation/cursor and materializes only visible plus
overscan rows. A filter change may scan the retained generation once; a stable unrelated chrome
snapshot performs zero log scan/clone/format work. Sequence lookup uses the retained range/index,
not a linear search. The unused per-record JSON publication must either gain one exact typed
generation subscriber or be deleted during the cutover.

## Proposed PERF-MVP-644: bounded logging ingress, writer and visible generation

1. publish monotonic admitted ranges through a count/byte/age-bounded ingress with deterministic
   discontinuity/drop receipts and reserved error/fatal behavior;
2. keep ring mutation short and independent of disk/sink latency; expose immutable generation,
   cursor/range and O(1) retained-sequence lookup;
3. move escaping/encoding and file I/O to one recursion-safe diagnostics writer lane, keep the file
   open, track segment bytes and batch writes/flushes under explicit durability rules;
4. add batch ingress for script/compiler/import/process bursts so one producer batch does not acquire
   locks and request flush per row;
5. replace complete console strings with filter-generation plus visible/overscan row handles and
   affected-range invalidation; broad editor/chrome reads reference the cached console generation;
6. connect the log change signal to exactly one retained consumer or remove the dead message-bus
   JSON projection; never maintain both as competing UI authorities;
7. make shutdown/fatal flush a measured, bounded terminal phase and expose admitted/dropped bytes,
   producer wait, writer queue age, batch/write/flush counts, UI visits and RSS.

## Reference-engine evidence and adaptation boundary

- Unreal `OutputDeviceFile.cpp:476-532` lazily creates one writer and retains its archive plus an
  asynchronous writer instead of reopening the file per record.
- `OutputDeviceFile.cpp:559-595` formats into that async writer. Per-line flush is an explicit
  `-FORCELOGFLUSH` mode, not the normal path; `Flush()` is a named boundary at `:419-429`.
- `OutputDeviceRedirector.cpp:500-529` owns buffered-item draining and distinguishes asynchronous
  from waiting flush. `:532-545` makes the drain a profiled operation. This supports a distinct
  output owner and measurable flush boundary rather than producer-thread file I/O.
- Unreal MessageLog keeps filtered message handles in a view model (`MessageLogListingViewModel.cpp:
  31-68`, `:180-187`). `SMessageLogListing.cpp:38-53` binds them to `SListView`, which generates
  rows on demand (`:294-300`) instead of rebuilding one complete console string for every broad UI
  snapshot.

Zircon should not copy Unreal's globals, exact buffer policy or MessageLog full filter refresh.
It should retain typed sources/jumps, finite byte budgets and immutable generation semantics while
adapting the persistent async writer and virtual row ownership to Runtime11 and retained UI.

## Measurement and acceptance gates

| gate | matrix | required result |
|---|---|---|
| ingress/writer | producers `1/16/64`, records `1/1K/1M`, payload `64B/8KiB`, disk stall `0/10ms/1s` | producer disk/flush/sink work 0; wait independent of disk latency until explicit bounded overflow policy; queues/bytes/age/RSS bounded; order/discontinuities deterministic |
| filesystem | segment sizes `1KiB/32MiB`, day/rollover, normal/fatal/shutdown | active open <=1; create-dir/metadata/open near segment generation, not record count; writes batched; normal flush not per record; bounded terminal flush and failure receipt |
| store/UI | retained `1/2,048/100K`, changed `0/1/1%`, filters 1/6, visible `20/100` at 60 Hz | stable scan/clone/format/join 0; changed work near affected plus visible rows; sequence lookup O(1); broad chrome reads share console generation; UI queue bounded |
| product | at least 31 comparable cold/warm F0/F4 and diagnostic-burst runs after build repair | WPR/xperf CPU sampling, CSwitch, mutex waits, file I/O/flush counts, allocations/RSS and package power; compare matched local Unreal logging with equal payload/rate/storage/power plan |

Static review is not milestone acceptance. Current managed tests, scale counters, product
reachability, WPR/power and an independent current-source review remain mandatory.

## Static gates executed

- Recomputed current inventory: 13 files, 1,889 physical lines, 16 inline tests and the unchanged
  ordered raw fingerprint
  `946a635610fa1756e280c8dfae32433d67c70f504258e1596d6f2d3ac80a99e0`.
- `rustfmt --edition 2021 --check`: **13/13 GREEN**.
- Scoped `git diff --check`: GREEN; only existing checkout line-ending warnings were emitted.
  Both new documents have zero trailing-whitespace findings.
- Front-matter references: 21/21 paths exist.
- Repository documentation gate: RED with 671 existing violations across 241 of 2,505 documents;
  violations owned by the two new documents: **0**.
- Session coordinator plan audit and the `codex-performance-audit-20260814` heartbeat: GREEN.
- Managed Cargo/tests, scale/allocation/RSS counters, product build, WPR/xperf and power: not run
  because the approved-root build defect and system-temp fixture placement remain unresolved.
