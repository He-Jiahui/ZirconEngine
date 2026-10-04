---
related_code:
  - zircon_editor/src/ui/activity
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/workbench_notifications.rs
  - zircon_editor/src/ui/workbench/reflection/activity_descriptors.rs
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
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Notifications/NotificationManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Notifications/SNotificationList.h
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Public/IMessageLogListing.h
---

# Protected plan routing: editor Activity generation and projection

## Reason for routing

The main performance plan, `review.md`, `pending.md`, Optimize10 and numbered owner plans are protected
or foreign dirty. This record routes the current 4/4-file evidence without overwriting their owners.
Evidence source:
`2026-08-19-editor-ui-activity-generation-projection-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-017

Extend the stable progress gate from the status bar to Activity notification projection. A stable job
generation currently still participates in complete progress snapshot, localization, view allocation
and bridge comparison. Require a shared primary/full progress generation, row revision and one
frame-coalesced Activity apply. Stable generation must perform zero label/detail clone, localization,
formatting or presentation invalidation.

### PERF-MVP-076 and PERF-MVP-099

Add Activity descriptor registry work to the full-reflection rebuild evidence:

- every reflection recreates descriptor vectors even when the view registry is unchanged;
- each descriptor's reflection path is formatted by its constructor and immediately formatted again;
- duplicate-ID registration discards new metadata, so current work is both redundant and stale;
- control-service catalog getters deep-clone complete descriptor vectors.

Required target: one `ViewRegistryGeneration -> ActivityRegistryGeneration` build, atomic
add/update/remove publication, shared/indexed reads and zero descriptor/path/catalog work for stable
reflection. Do not add a second Workbench-local cache outside the common generation invalidation.

### PERF-MVP-105

Keep the committed-identity target and add Activity synchronization: retained tick and dispatch-side
effect paths must read the active Workbench template identity in O(1), compare source generations,
and coalesce to at most one apply per frame. A notification generation miss must not build complete
chrome merely to determine whether the Workbench window is active.

### PERF-MVP-269

Promote the notification model from a bounded full snapshot to typed row generations/deltas. The
current 64-entry cap is a useful pressure boundary, but all retained rows are still localized and
allocated before semantic equality is known. Require stable ID/revision, locale generation, deadline
delta, aggregate metadata, visible-window projection and explicit entries/bytes/age/drop/coalesce
counters. Preserve Optimize10's durable record and action semantics; do not treat the transient Slate
API as proof of a complete journal design.

### New P0 child item: paged Activity Log projection

Add one Performance01 child jointly owned by Editor17 and Editor06:

- the canonical filtered snapshot is cloned into `ActivityLogView` wrappers;
- the host then traverses the complete history for joined text, severity and jump arrays;
- live presentation cost follows retained history rather than visible rows.

Required target: logging-owned immutable generation and cursor/page query over shared records;
visible-plus-overscan typed projection; one formatting pass per visible changed row; zero stable work;
and explicit full-text materialization only for copy/export. Filter, eviction, sequence and `LogJump`
remain canonical logging semantics.

## Requested owner-plan updates

### Editor17

Own `NotificationAuthority` and `EditorLogService` generations. Publish stable record identity,
revision, locale/deadline inputs, bounded deltas, journal/query cursors and retention diagnostics.
Progress must terminalize into durable records rather than disappear. The service must not own
Workbench strings, nodes or paint state.

### Editor06

Consume logging pages as typed shared rows and route typed jumps. Replace the current full joined-text
live DTO with a visible-window model; keep full copy/export as an explicit command. Do not add a
console-local history, filter or unbounded cache.

Before terminal acceptance, update the current Editor17 static contract fixtures for the folder-backed
`play_pending_decision/tests/**` topology and the moved
`system/progress_observer::ProgressObserverDispatch` owner. The present broad baseline is 8/10; do not
move production types back into facade files merely to satisfy source-shape assertions.

### EditorLayout09

Retain bounded notification retention, unread and overflow ownership. Extend it with entry/byte/age
budgets, row revision/delta, cursor expiry and observable coalesce/drop reasons. The existing 64-row
limit is not permission to rebuild all 64 rows on every stable tick.

### EditorUI08

Own the frame-coalesced consumer and immutable Activity registry projection. Compare cheap source,
locale, template and registry generations before materialization; patch changed visible rows and
descriptor IDs only; publish at most once per frame. No plugin/localization callback, full chrome
build, log join or descriptor catalog clone may run under stable generation.

### Optimize zircon_editor/10

Retain product-correctness ownership for notification journal, decisions, actions, accessibility and
diagnostic integration. Performance01 supplies the quantitative generation, allocation, scale,
latency and power gates. The plans share one typed record authority and must not create parallel
notification histories.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/activity/**` with 4/4 files,
  472 lines, 3 tests, fingerprint `9f580df0...`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require the notification, descriptor and logging generation
  cutovers plus managed Cargo, scale counters, F0/F4 WPR/ETW, allocator/RSS/package-power evidence
  and rendering parity where submitted UI changes.
- Preserve these as module-level rows; do not expand the protected indexes into four per-file entries.

## Validation and milestone rule

The static review is complete, not the performance milestone. Run the matrix from the architecture
review with artifacts under D/E/F only. RenderDoc is conditional on submitted UI/render-resource
changes and cannot replace CPU/lock/allocation evidence. Commit and send quantified WeCom results only
after the owner cutovers, tests and dynamic product gates pass; do not send this routing record as a
completed milestone.
