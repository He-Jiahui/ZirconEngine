---
related_code:
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/core/plugin/lifecycle_message_bridge.rs
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
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
---

# Protected plan routing: editor message bus and UI delta

## Reason for routing

`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`, `review.md`, `pending.md` and
the numbered owner plans are protected/foreign dirty in this session. This record preserves the
current-source correction and exact requested plan changes without overwriting another owner's work.

The evidence source is
`2026-08-15-editor-message-bus-ui-delta-current-architecture-review.md`.

## Requested Performance01 correction

Replace the stale current-state half of `PERF-MVP-019`. The bus no longer holds the global mutex
through per-inbox fanout, no longer prepares/sizes two lossless deliveries, and request does not
deep-clone Custom JSON before creating the shared delivery. Current facts are:

- route target resolution, sequence allocation, one delivery construction and retained-byte sizing
  occur under the global bus lock;
- dispatch uses sorted per-subscriber inbox locks outside that lock and shares one cached `Arc`
  delivery; lossless fanout is atomic;
- zero-target publication still builds/sizes a delivery and detailed report;
- full-inbox drain has no entry/byte/deadline page or wall age;
- best-effort fanout can head-of-line block on a contended earlier inbox;
- `EditorUiDeltaQueue` is a new unbounded patch/barrier journal owned and materialized under the
  global bus mutex;
- retained host clones every drained patch and can run full reflection plus retry;
- scene-inspection generation gaps recover through reflow but can repeatedly destroy sparse scaling.

Keep `PERF-MVP-019` P0 because the bus owns retained-host invalidation state. Its new target should be
a bounded retained-owner UI delta cursor plus bounded inbox pages, zero-target early return, wall-age
and lock telemetry, and preserved atomic lossless/request semantics. Do not require a dedicated bus
thread; Unreal is evidence for affinity separation, not a mandate for private scheduling.

Link UI materialization work to existing `PERF-MVP-113`: stable frames must materialize/clone/apply
zero patches, an accepted generation must apply once, and stale/error fallback must rebuild at most
once without replaying the stale page.

Keep `PERF-MVP-594` for the lifecycle bridge. Add the current proof that it still drains the entire
bus inbox into a second unbounded `VecDeque` and invokes all callbacks while holding the bridge
pending mutex. Acceptance remains callback-in-lock=0, bounded per-tick entries/bytes/wall, one owner,
generation-safe error/reload/unload and no accepted edge loss/duplication/reordering.

## Requested owner-plan updates

### Editor02

Update the 2026-07-30 supplement in
`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md:286-292`; lines 288-290 repeat the stale
global-lock and duplicate-preparation diagnosis. Preserve the still-correct bounded lanes, inbox page,
wall-age, scene gap and plugin bridge requirements. Add the UI delta retention owner split and the
0/1/100/10k subscriber, 1/16 publisher contention matrix from the current report.

### EditorUI08

Add a retained UI delta milestone to
`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md` and its performance handoff:
patches are latest-by-node within a generation, discrete edges are ordered and admission-bounded,
pages expose entries/bytes/remaining/age, and fallback never reapplies a stale page after a full
reflection rebuild. Validate stable, 1k-barrier and 10k/100k-node cases with stage wall/counters.

### Editor12 and Plugins01

Update `docs/plans/zircon_editor/editor/12-plugin-management.md` and
`docs/plans/zircon_plugins/01-plugin-architecture-core.md` to consume Editor02 bounded pages directly,
snapshot ordered active handles/generation under short locks, invoke callbacks outside all bus,
bridge and manager locks, then perform one generation-checked commit. Slow/faulted plugins require
typed quarantine diagnostics and unload/reload-safe cancellation.

### Runtime11

`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md` may supply only a bounded single-flight
ticket for plugins that explicitly declare non-main callback affinity. It must not become a second
editor scheduler or a private catch-all callback pool.

## Requested protected index state

- `pending.md`: retain one concise module row for `zircon_editor/src/core/editor_message/**` with
  `static_complete / dynamic_pending`, 35/35 production files, 2,935 lines, 10 inline tests and the
  current review link. Note the related 13/13 external files and 32 tests in the evidence column.
- `review.md`: do not add the module. Managed Cargo, ignored fanout benchmark, multi-producer
  contention/backpressure counters, UI delta scale gates and F4 WPR evidence are still absent.

## Milestone and notification state

This is a static review/routing record, not an accepted performance milestone. No git commit or WeCom
notification is due. Commit and quantified WeCom notification occur after current-source dynamic
evidence passes the acceptance matrices and the protected indexes are reconciled by their owner.
