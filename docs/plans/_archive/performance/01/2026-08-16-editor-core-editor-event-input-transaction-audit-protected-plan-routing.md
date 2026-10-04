---
related_code:
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/ScopedTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
---

# Protected plan routing: editor input, transaction, replay, and audit separation

## Reason for routing

The main performance plan, `review.md`, `pending.md`, optimize record and numbered owner plans are
protected/foreign dirty in this session. This record routes the current-source structural findings
without overwriting their owners. Evidence source:
`2026-08-16-editor-core-editor-event-input-transaction-audit-current-architecture-review.md`.

## New Performance01 task

Add `PERF-MVP-646` P0: hard-cut the universal `EditorEventRecord` completion/replay pipeline into
realtime input, committed semantic transaction, typed invalidation receipt and bounded audit delivery.

Current evidence:

- every handled viewport move enters command reverse discovery, the global shell, record/effect/result
  allocation, full JSON size encoding, journal indexes and listener fanout;
- event order and authoring revision share one pre-execution stamp, so failed/no-op/high-frequency input
  advances document revision;
- `EditorEventReplay` re-dispatches every supplied journal record and ignores undo/replay policy;
- the journal includes raw input, transient UI, failures and external side-effect requests;
- `HoverNode` and `PressNode` latest keys omit `node_path` and can coalesce away required clears.

Required target:

- direct owner-thread `RealtimeInput` with latest-value coalescing only where edge ordering is not lost;
- typed `CommandRoute` and explicit successful transaction commit as the only authoring revision/replay
  authority;
- one shared typed execution receipt with fixed invalidation mask, no per-input JSON/result/effect Vec;
- versioned replay disposition that executes only committed semantic operations and rejects ambiguous
  legacy rows;
- optional audit envelopes encoded only for actual ABI/persistence consumers, with count+byte+deadline
  admission and declared listener affinity;
- no private event scheduler and no unbounded handoff.

Acceptance is the full matrix in the evidence record: pointer 125/500/1,000 Hz and 1M storms;
changed/no-op/failure commands; replay-class matrix including save/import/close; listeners
0/1/1k/10k; payload 64 B/2 MiB/64 MiB; and same-machine F4 WPR CPU/lock/allocation/RSS/power and
input-to-present results. Raw input and external side-effect replay count must be zero. Authoring
revision advances exactly once per successful changed commit.

## Existing task correction

### PERF-MVP-067

Retain the 2026-08-15 corrections: current source already has three hard retention classes, indexed
latest replacement, immutable route snapshots, cursor-first three-way pages and no full-set sort.

Keep `PERF-MVP-067` for bounded audit/listener storage, shared-log versus per-inbox measurement,
count+byte+deadline pages, one final ABI materialization and listener lag/ack/admission. Move raw input,
revision and executable replay ownership to `PERF-MVP-646`; otherwise a queue optimization would keep
the invalid universal-event architecture.

### PERF-MVP-645

Editor08's typed `CommandRoute` is a dependency of `PERF-MVP-646`. Direct realtime input must bypass
the command registry. Committed commands carry identity into transaction and audit receipts; replay
never reverse-discovers identity by scanning event payloads.

## Requested owner-plan updates

### EditorUI01

Own direct realtime input routing, pointer edge ordering and frame-boundary latest-value coalescing.
Stable pointer movement must produce typed no-damage without journal/command/replay work.

### Editor03

Own the sole versioned committed-operation/transaction replay schema. Separate event order from
successful authoring revision. Remove executable replay of raw `EditorEventRecord` arrays after a
fail-closed compatibility migration.

### Editor02

Own optional audit derivation, retention, listener topics/cursors and bounded page/ABI materialization.
Fix node-qualified hover/press state before retaining latest coalescing. Measure central shared-log
subscriber cursors against per-inbox indexes before selecting a data structure.

### Editor08

Publish typed command/operation/direct-input identity before execution under `PERF-MVP-645`; never
recover command identity from arbitrary completed events.

### EditorUI08

Consume the shared typed execution receipt and fixed invalidation mask. No-op input advances neither
authoring nor presentation generation. Apply exact hover/press patches without forcing a broad audit
record or shell rebuild.

### Editor12 and Runtime11

Plugins declare audit topic, delivery class and affinity. Main-affinity callbacks remain budgeted on
the editor owner; only declared non-main work uses existing Runtime11 bounded scheduling with
cancellation and generation checks.

### Optimize zircon_editor/01

Add the event-pipeline dependency to the retained-UI architecture milestones: the existing warmup,
measured, quiescence and typed no-damage evidence cannot pass while raw pointer movement performs
command reverse lookup and synchronous audit JSON/index work.

## Requested protected index state

- `pending.md`: retain one concise `zircon_editor/src/core/editor_event/**` module row with
  `static_complete / dynamic_pending`, 36/36 production files, 2,667 lines, 8 tests, related 30/30
  external files and the new structural-review link.
- `review.md`: do not add the module. Managed current-source tests, semantic replay migration,
  pointer/command/listener counters and F4 WPR same-machine evidence are absent.

## Milestone and notification state

This is a static structural review and protected-plan routing record, not an accepted milestone. No
git commit or WeCom notification is due. Commit and quantified WeCom notification occur only after
the dynamic matrix passes and the protected indexes are reconciled by their owners.
