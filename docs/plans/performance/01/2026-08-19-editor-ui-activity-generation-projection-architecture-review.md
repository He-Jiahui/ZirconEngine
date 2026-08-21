---
related_code:
  - zircon_editor/src/ui/activity
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/workbench_notifications.rs
  - zircon_editor/src/ui/workbench/reflection/activity_descriptors.rs
tests:
  - zircon_editor/src/ui/activity/view.rs
  - tools/tests/test_editor17_activity_log_projection_contract.py
  - tools/tests/test_editor17_decision_notification_center_contract.py
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Notifications/NotificationManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Notifications/SNotificationList.h
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Public/IMessageLogListing.h
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
created_at: 2026-08-19
---

# Editor UI activity generation and projection architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for stable-tick notification work, reflection registry churn and full Activity Log
  materialization; P1 for descriptor allocation after the generation boundary exists.
- Accounting: retain `zircon_editor/src/ui/activity/**` in `pending.md`. Do not add it to
  `review.md` before the generation/delta cutover and the dynamic matrix pass.
- Code disposition: no Rust source changed. The complete `zircon_editor/src/ui/**` area is actively
  owned by another session, and the defects span notification, logging, Workbench and host owners.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/activity/**` | 4/4 | 472 | 3 | 14,213 | `9f580df0e6b89cce6a70f72158faedf862fa72b25b740a8a6a8271fbf5ea432f` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All four current Rust
files and their three tests were read in full. Production reachability was followed through retained
notification synchronization, Activity Log projection, Workbench reflection and the control service.

## File acceptance record

| file | current-source performance verdict |
|---|---|
| `mod.rs` | Re-export only. No independent runtime cost. |
| `slot.rs` | Finite enum plus preference value. No independent hotspot. |
| `view.rs` | Toast and progress helpers localize and allocate every row for every call. Log projection clones every `LogRecord` into a wrapper before the host performs three more full passes. Typed fields and immutable core inputs are positive, but this is not a generation-owned read model. |
| `window.rs` | Descriptor construction formats and owns a reflection path. The Workbench adapter immediately replaces that path with an identical newly formatted value, so current construction pays two path/string allocations per window. This is secondary to full registry reconstruction. |

## Structural bottlenecks

### P0: stable notification synchronization performs full work before equality

`sync_activity_notifications()` first calls
`active_activity_window_template_document_is()`, a known complete-chrome query. It then requests all
pending decisions, clones the live toast and job-progress snapshots, localizes every row and allocates
complete Activity view vectors. Only after this work does the bridge compare the prepared snapshot and
return unchanged. The method is reached from the retained tick and dispatch-side-effect paths, so a
stable editor can pay the same gate and projection more than once per interaction frame.

The local 64-entry retained limit prevents unbounded output but does not make unchanged work free.
`remaining_lifetime` also changes with wall time, so coupling expiry display to semantic row equality
can invalidate the whole center even when only a timer scalar changed.

### P0: Activity descriptors are rebuilt and discarded on every full reflection

`refresh_reflection_for_shell()` obtains every Workbench view descriptor, rebuilds Activity view and
window vectors, then calls `register_activity_descriptors()`. Registration checks existing IDs and
discards duplicate freshly allocated descriptors; it does not update an existing descriptor if its
source title, icon, placement or capabilities changed. Constructors format a default reflection path,
and the adapter immediately formats and assigns the same path again.

This combines wasted stable-reflection work with a stale-state defect. A local `format!` removal would
save one allocation but preserve the wrong ownership and refresh algorithm.

### P0: Activity Log projection clones and flattens the complete filtered history

`EditorLogService::snapshot()` returns the current filtered records. `activity_log_views()` clones
every record into another vector. `editor_activity_log.rs` then separately maps all views to formatted
row strings and joins them, maps all rows to severity levels, and maps all rows again to optional jump
sequences. The resulting parallel arrays duplicate identity and make a small visible console window
pay for the complete filtered history.

The canonical filter and typed `LogJump` are correct boundaries and must remain. The target is not a
second console cache: logging owns one paged generation and the UI materializes only the requested
visible window.

### P2: unused control-service snapshots deep-clone descriptor catalogs

`EditorUiControlService` indexes descriptors by ID, which is useful. Its `activity_views()` and
`activity_windows()` APIs nevertheless collect cloned descriptor vectors. No current UI production
caller was found for either control-service getter; similarly named layout getters are separate APIs.
These methods are therefore dead cost surfaces rather than current hotspots. Remove them if the
generation cutover leaves them unused; otherwise return an immutable registry-generation handle or
borrowed/indexed query instead of restoring caller-local catalog clones.

## Reference-engine evidence

- Unreal `NotificationManager.h:75-115` separates manager tick/add/queue operations and exposes a
  progress handle/update path. `SNotificationList.h:35-75` exposes typed item updates for text,
  subtext, completion and expiry instead of requiring callers to rebuild the whole list.
- Unreal `SNotificationList.h:130-235` defines notification identity, text, buttons, hyperlink,
  completion and lifetime as one typed lifecycle payload. Zircon should retain typed row identity and
  update only changed presentation fields.
- Unreal `IMessageLogListing.h:18-135` exposes filtered/shared messages, page and selection state,
  token execution, and data/page/selection change events. This supports a durable log authority with
  paged, event-driven UI projection instead of cloning and joining all records per refresh.

These sources establish lifecycle and projection shape, not comparable performance numbers. Unreal's
implementation is not evidence that Zircon already meets a CPU, latency or power target. Same-hardware
WPR/ETW, allocator and package-power measurements remain mandatory. A complete durable notification
journal is also not inferred from Slate's transient notification API.

## Required architecture cutover

1. `NotificationAuthority` publishes one immutable
   `ActivityNotificationGeneration(sequence, locale_generation, rows, aggregates)` plus bounded deltas
   keyed by stable record ID/revision. Toast expiry is a scheduler-owned deadline delta, not a reason
   to relocalize and rebuild all rows.
2. Retained tick and dispatch paths first compare cheap source/template/locale generations. They
   coalesce to at most one Activity apply per frame and materialize only changed rows. The bridge
   consumes typed rows; it must not decide equality after string projection.
3. `ViewRegistryGeneration` produces one immutable `ActivityRegistryGeneration` when structural view
   metadata changes. The control service atomically replaces/diffs that generation and publishes ID
   indexes. Stable reflection performs zero descriptor construction; changed descriptors replace stale
   values rather than being discarded by duplicate-ID checks.
4. `EditorLogService` publishes a generation plus cursor/page query over shared records. The console
   requests visible plus overscan rows, preserves typed severity/source/jump fields and formats each
   visible row once. Full joined text is an explicit export/copy operation, not the live presentation
   authority.
5. Keep one owner per fact: notification authority owns live/terminal records, logging owns retained
   diagnostics, the view registry owns descriptor metadata, and EditorUI08 owns only generation-bound
   retained presentation. No host-local unbounded cache or parallel history is allowed.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| stable notifications | rows `0/1/64`, stable ticks `1/100k`, tick plus dispatch in one frame | full snapshot/localization/row allocation/bridge serialization/chrome build `=0`; Activity apply `<=1/frame`; timer update touches one row/deadline field |
| notification delta | toast/progress/decision rows `1/64/1k`, change rate `0/1/100%` | work near changed rows plus visible window; no full semantic equality after formatting; queue entries/bytes/age/drop/coalesce remain bounded and observable |
| descriptor registry | descriptors `1/100/10k`, stable reflections `1/100k`, add/update/remove | stable descriptor/path build and catalog clone `=0`; changed registry build/publish `=1/generation`; stale metadata `=0`; indexed lookup near O(1) |
| Activity Log | records `1/1k/100k`, visible rows `20/50/100`, filter/page/jump/eviction | cloned/formatted rows `<=visible+overscan`; stable generation work `=0`; live full-history join `=0`; typed filter, order, severity and jump parity |
| product | F0 startup and F4 editor, cold/warm/idle/storm, 31 runs | WPR/ETW CPU, waits, wakeups, lock hold/wait, allocations, RSS, UI p95 and package power with identical hardware/assets/settings; artifacts stay on D/E/F |

RenderDoc is not a control-plane profiler for these defects. It is required only if the eventual UI
cutover changes submitted notification/log geometry or rendering resources; then capture draw/event,
resource and output parity. WPR/ETW and allocation counters are the primary dynamic evidence here.

## Static gates executed

- Read 4/4 current Rust files and all three in-module tests; reproduced 14,213 bytes and fingerprint
  `9f580df0...` after production-call review.
- Confirmed full notification localization/materialization precedes the bridge unchanged check and
  the complete-chrome activity-window gate precedes source generation checks.
- Confirmed every full reflection rebuilds Activity descriptors, duplicates reflection-path
  formatting and discards already-registered IDs rather than updating their metadata.
- Confirmed Activity Log creates a cloned wrapper vector, then complete text/level/jump projections;
  the canonical logging store and typed jump remain the required authority.
- Activity Log projection contracts passed 2/2. The broader Decision/Notification contract suite
  passed 8/10: one error still reads the old single-file `play_pending_decision/tests.rs` after tests
  became folder-backed, and one failure searches only the job-system root for
  `ProgressObserverDispatch` after it moved to `system/progress_observer.rs`. Both related source
  scopes are foreign modified; this report records the owner-integration baseline and does not edit it.
- `rustfmt --edition 2021 --check` passed for all 4/4 Activity Rust files. Scoped
  `git diff --check`, 20/20 routed-path existence and
  `python -m tools.session_coordinator --repo-root . --json plan audit` passed.
- The documentation convention gate reports 0 violations owned by these two records. Its global
  baseline remains red at 692 violations across 242 of 2,707 documents; unrelated debt was not edited.
- The source fingerprint was recomputed after documentation edits and remains `9f580df0...`.
- Read the cited Unreal primary sources and the current Optimize10/11 reviews. Dynamic Cargo, scale
  counters, F0/F4 launch, WPR/ETW, allocator and package-power evidence remain pending. This is not
  an accepted milestone, so no commit or WeCom notification is due.
