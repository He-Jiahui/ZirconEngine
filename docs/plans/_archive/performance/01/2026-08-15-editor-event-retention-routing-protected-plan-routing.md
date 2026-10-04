---
related_code:
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/godot/core/object/message_queue.cpp
---

# Protected plan routing: editor event retention and routing

## Reason for routing

`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`, `review.md`, `pending.md` and
the numbered owner plans are protected/foreign dirty in this session. This record preserves exact
current-source corrections without overwriting another owner's work. The evidence source is
`2026-08-15-editor-event-retention-routing-current-architecture-review.md`.

## Requested Performance01 correction

Replace the stale current-state half of `PERF-MVP-067` and its July summary. Current facts are:

- retention is bounded by three independent count/encoded-byte/age budgets;
- latest replacement uses a key index and ordered maps/sets, not a linear `VecDeque` scan/remove;
- immutable route generations move filter/enqueue outside the listener-registry mutex;
- full enumeration is a three-way ordered merge, not a full sort;
- listener delivery is cursor-first and count-bounded to 256 rows;
- current production/test inventory is 36/36 files, 2,667 lines, 8 inline tests and 30/30 external
  files, 8,128 lines, 138 tests.

Keep `PERF-MVP-067` P0, but rewrite the root cause: command completion, audit retention and observer
fanout still share one synchronous path. Successful host dispatch deep-clones the record; every event
allocates and fills a discarded JSON buffer for size accounting; journal indexing and matching
per-inbox locks run inline; pages are count-only and journal snapshots still deep-materialize all
records. Existing storm tests prove bounded retained state, not bounded main-thread arrival cost.

Replace the target with four dependency-ordered milestones from the evidence record: stage counters
and F4 WPR baseline; one shared command-result/effect owner plus bounded audit/observer handoff;
entry+byte+deadline fanout/pages and cursor journal export; then same-machine latency/RSS/power
comparison. Preserve authoring mutation on the editor owner and explicit main-affinity callbacks.

## Requested owner-plan updates

### Editor02

Replace the stale bullets in `docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md:294-300`.
Retain Editor02 ownership of the route generation, journal, listener inbox and cursor/page contract.
Add counters for record clone bytes, serialized-size traversal/allocation, journal/index stages,
route visits, per-inbox wait/hold and owned/JSON projection bytes. Do not mark the existing 1k/10k
storm tests as performance acceptance without timing/allocation/lock evidence.

### EditorUI08

Consume one shared immutable event result/effect owner. The retained host must not force a deep copy
solely because audit retention also needs the record. Keep input mutation/effect application on the
editor owner and measure input-to-present separately from audit and observer stages.

### Editor12 and Plugins01

Consume count+byte+deadline pages or an equivalent bounded cursor directly. Declare callback affinity;
invoke non-main callbacks through one bounded Runtime11 ticket, and keep main callbacks on the editor
owner under the same budget. Preserve plugin generation, reload/unload cancellation, order, ack and
typed failure/quarantine diagnostics.

### Runtime11

Provide scheduling only after Editor02 has explicit admission, affinity and cancellation contracts.
Do not create a private editor-event pool or move authoring mutation off its authority thread.

## Requested protected index state

- `pending.md`: add or retain one concise module row for `zircon_editor/src/core/editor_event/**` with
  `static_complete / dynamic_pending`, 36/36 production files, 2,667 lines, 8 inline tests and the
  current review link. Note the related 30/30 external files and 138 tests in evidence.
- `review.md`: do not add the module. Managed Cargo, stage counters, listener contention, F4 WPR and
  same-machine latency/RSS/power evidence are absent.

## Milestone and notification state

This is a static review/routing record, not an accepted performance milestone. No git commit or WeCom
notification is due. Commit and quantified WeCom notification occur only after current-source dynamic
evidence closes the acceptance table and the protected indexes are reconciled by their owner.
