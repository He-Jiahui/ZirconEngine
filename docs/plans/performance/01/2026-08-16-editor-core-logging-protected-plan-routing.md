---
related_code:
  - zircon_editor/src/core/logging
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/script_build/diagnostics_sink.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceFile.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Private/Presentation/MessageLogListingViewModel.cpp
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Private/UserInterface/SMessageLogListing.cpp
---

# Protected plan routing: editor core logging

## Reason for routing

Performance01, `pending.md`, `review.md`, Editor13 and Editor17 are protected/foreign dirty in this
session. Three logging source files are also foreign dirty. This record requests current-source plan
corrections without overwriting those owners. Canonical evidence is
`2026-08-16-editor-core-logging-current-architecture-review.md`.

## Requested Performance01 correction

Record `zircon_editor/src/core/logging/**` as **13/13 Rust files, 1,889 physical lines, 16 tests**
with ordered path-and-raw-content fingerprint
`946a635610fa1756e280c8dfae32433d67c70f504258e1596d6f2d3ac80a99e0`.

Retain these current facts:

- count/estimated-byte ring and event limits, typed source/jump identity and clear/resync
  discontinuity are useful foundations;
- every persisted emit still performs directory check, time, complete line formatting, metadata,
  open, write and flush under one global emission mutex;
- the first emitter synchronously drains and calls the event sink; script diagnostic bursts emit and
  flush one row at a time;
- current production UI reads the log service directly. `EditorTopic::log()` subscriber
  registrations were found only in tests, while every emit still allocates/publishes JSON;
- each editor/chrome/console snapshot scans and clones the full retained store, clones records into
  views again and rebuilds a complete joined text plus parallel arrays;
- logical payload estimates do not bound Arc/record/queue/writer overhead or measured RSS;
- tests use the system temp directory and are not approved under the no-C-artifact rule.

## Proposed PERF-MVP-644

| id | priority | current diagnosis | required cutover | acceptance |
|---|---|---|---|---|
| PERF-MVP-644 | P0 | All producers serialize behind per-record file reopen/metadata/write/flush; the producer that starts event dispatch drains the sink synchronously. Broad UI snapshots repeatedly full-scan, double-clone and stringify the complete console, while the bus event has no production subscriber. | Editor02/13/14/17 + EditorUI08 + Runtime11 create `LogIngressRange -> LogStoreGeneration -> diagnostics writer batch -> persistence receipt -> FilteredLogWindowGeneration -> RetainedConsoleRowDelta`. Keep the active segment open, batch and explicitly flush on byte/age/fatal/shutdown rules, expose cursor/range/O(1) sequence lookup, batch producer diagnostics, materialize visible rows only and hard-cut the unused duplicate bus projection. The writer lane must be recursion-safe and cannot depend on logging through itself. | producers `1/16/64`, rows `1/1K/1M`, payload `64B/8KiB`, stall `0/10ms/1s`, retained `2,048/100K`, visible `20/100`: producer disk/flush/sink work 0; queue bytes/age/RSS bounded; active open <=1; metadata/open near segment count; normal flush not per record; stable UI scan/clone/format 0; deterministic discontinuity/fatal/shutdown; managed tests and F0/F4 WPR CPU/waits/file-I/O/RSS/power pass |

## Requested owner-plan updates

### Editor17

Make the in-memory log generation the immediate authority. Specify admitted/dropped range receipts,
logical/RSS budgets, batch APIs and persistence/fatal/shutdown durability. Replace per-record
rolling-file operations with one open segment and explicit batch/flush lifecycle.

### Editor14 and Runtime11

Own one recursion-safe diagnostics blocking-I/O lane with count/byte/age/deadline bounds and phase
counters. It must not submit through a path whose failure logs into the same queue. Shutdown waits
only at the explicit bounded flush phase.

### Editor02 and EditorUI08

Publish one typed log generation/range invalidation to the retained console. Broad editor/chrome
snapshots reference the cached console generation. Filter and visible-window owners materialize only
visible plus overscan rows. Delete the unused per-record JSON bus path after the typed consumer is
live; do not retain two projection authorities.

### Editor13

Project compiler diagnostics through batch ingress. Generation/request/step dedup remains, but one
diagnostic batch cannot cause N producer locks, file opens or flush requests.

### Render17

Record ingress wait, store lock hold, admitted/dropped rows/bytes, writer queue age/RSS, escape/
encode rows, segment open/metadata/write/flush and UI scan/clone/format/visible-row counts. RenderDoc
is not an acceptance tool for this slice.

## Requested protected index state

- `pending.md`: add the frozen module counts/fingerprint, `static_complete / dynamic_blocked`, the
  canonical review and PERF-MVP-644.
- `review.md`: do not add the module until PERF-MVP-644, approved-root tests, scale/RSS gates and
  31-run F0/F4 WPR CPU/wait/file-I/O/power evidence are green.

## Milestone and notification state

This is static architecture evidence only. Product build and approved fixture placement are blocked,
so no performance milestone commit or WeCom notification is due. Both become mandatory after owner
plan acceptance and the dynamic matrix passes.
