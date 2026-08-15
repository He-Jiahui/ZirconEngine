---
related_code:
  - zircon_editor/src/core/notifications
  - zircon_editor/src/ui/host/play_pending_decision
  - zircon_editor/src/ui/retained_host/app/workbench_notifications.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/NotificationManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/SlateAsyncTaskNotificationImpl.cpp
---

# Protected plan routing: editor notification generation and projection

## Reason for routing

`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`, `review.md`, `pending.md` and
the owner plans are protected/foreign dirty in this session. This record preserves the current-source
corrections without overwriting their owners. The evidence source is
`2026-08-15-editor-notification-generation-projection-current-architecture-review.md`.

## Requested Performance01 correction

Update `PERF-MVP-596` from a decision-only projection issue to the current unified activity
notification path. Preserve its P0 priority and existing Decision diagnosis, then add:

- current scope is 25/25 files, 3,469 lines and 38 tests, not 9/9, 1,562 and 18;
- the old empty-option early-return bug is gone; empty input reaches the bridge and can clear rows;
- every active main-Workbench tick also snapshots/localizes all live Toast and bound Progress rows;
- Progress snapshot builds a captured map/id vector, queries job snapshots, relocks and builds another
  map before cloning output rows;
- Toast scans/prunes/clones its bounded live map on every tick even before the next expiry;
- the bridge encodes up to 64 typed rows into strings, reparses unread/id/kind, clones old arrays and
  performs semantic string parsing before discovering no change;
- toast publish/dispatch can invoke the same complete synchronization more than once in one frame.

Replace the target with one Editor17-owned immutable `ActivityNotificationProjection` keyed by
Decision, Toast, Progress and locale generations and carrying `next_toast_expiry`. Editor04 and
Editor14 publish indexed/shared source generations; EditorUI08 applies a visible changed generation
at most once and treats empty as a clear token. Dispatch marks dirty instead of synchronously
rebuilding. Encode runtime-UI compatibility rows once per changed generation only.

Keep related ownership distinct:

- `PERF-MVP-017` continues to own the separate one-row status-bar primary-job projection;
- `PERF-MVP-105` continues to own active-template/full-chrome lookup;
- `PERF-MVP-269` continues to own notification component string-array parsing and row mutation after
  the generation boundary.

Update acceptance with source/unified generations, next-expiry wakeups, sync calls/frame, lock
wait/hold, rows cloned/localized, translated/encoded bytes, pipe parsing, final invalidations and
same-machine F4 CPU/RSS/package-power. Stable work after initial apply and before the next expiry must
be zero; each accepted generation must build/apply at most once.

## Requested owner-plan updates

### Editor17

Own the single unified activity notification projection, source revision tuple, locale revision,
unread/overflow aggregates and `next_toast_expiry`. Return shared typed rows or `NotModified`; do not
create a second authority. Publish and expiry transitions increment exactly once, and an empty
generation remains observable.

### Editor04

Publish Decision pending generation and direct ticket/notification/selection indexes. Build localized
Play choices once per Decision or locale generation; preserve ticket incarnation, cursor-gap,
idempotent resolution and explicit recovery semantics.

### Editor14

Expose the authoritative Progress generation/shared rows to Editor17. Remove per-tick duplicate map
construction for the notification surface. Keep `PERF-MVP-017`'s status-bar primary snapshot as a
separate consumer of the same generation, not a second cache.

### EditorUI08

Store the last applied unified generation and apply only when active and changed. Coalesce tick and
dispatch dirtiness so one frame applies at most once. Use typed metadata for unread/id/kind; if the
string-array bridge remains during migration, encode it once per changed generation and never parse
it as authority.

## Requested protected index state

- `pending.md`: replace the stale row with one concise module row for
  `zircon_editor/src/core/notifications/**`, `static_complete / dynamic_pending`, 25/25 files, 3,469
  lines, 38 tests and the current review link.
- `review.md`: do not add the module. Managed Cargo, stable/change scale counters, F4 WPR,
  same-machine latency/RSS/power and downstream RenderDoc paint parity are absent.

## Milestone and notification state

This is a static review/routing record, not an accepted performance milestone. No git commit or WeCom
notification is due. Commit and quantified WeCom notification occur only after current-source dynamic
evidence closes the acceptance matrix and the protected indexes are reconciled by their owner.
