---
related_code:
  - zircon_editor/src/core/sync
  - zircon_editor/src/ui/host/editor_world_sync.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_editor/src/core/gateway/session/world_sync.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/ActorHierarchy.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerHierarchy.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/SceneOutlinerStandaloneTypes.h
---

# Protected plan routing: WorldSync routes and typed hierarchy operations

## Reason for routing

Performance01, `review.md`, `pending.md` and all owner plans are protected/foreign dirty in this
session. This record requests current-source corrections without overwriting their owners. Evidence
is `2026-08-15-editor-core-world-sync-routing-current-architecture-review.md`.

## Requested Performance01 corrections

Replace the stale `core/sync` accounting with 5/5 Rust files, 1,342 physical lines, 20 tests, zero
ignored and ordinal fingerprint
`da7589d8e6371d120447e460d377a5d3a719221d7ceba56c15af815b87afcea7`. The old 3-file/450-line/8-test
row predates the live pump and retained-host wiring.

Update `PERF-MVP-468` rather than retaining its old main diagnosis:

- current runtime typed indexes, reused ancestry scratch, bounded semantic fact coalescing and
  canonical dirty-token fast path are present;
- canonical projection no longer builds the three diagnostic trees on valid batches, and view dirty
  submission crosses one bus lock per batch;
- the current P0 is per-view remote subscriptions: equal WatchKeys create duplicate runtime tokens,
  mutation fanout and wire rows instead of one shared route with editor-local fanout;
- dirty token count/bytes remains unpaged, and malformed validation/diagnostics lacks a separate
  work budget.

Extend `PERF-MVP-597` with the current empty WorldSync chain: retained UI synchronously polls every
frame; dynamic empty drain locks the session/World/subscription table and serializes/decodes `[]`;
in-process still locks producer state. WorldSync must use the same Runtime11 per-session lane and
ready/wake generation as runtime-event work, not another executor.

Link `PERF-MVP-459/563` to the duplicate authority finding: runtime already creates typed mutation
facts and an inspection generation, but editor converts facts to unused JSON and then re-observes the
World to build another inspection message. The final change set/inspection artifact must be
producer-owned and generation-first.

Add the replacement correctness finding: generation is read separately from drain, so replacement
can make the pump destructively consume the first new-session batch using old watch/generation
state. Watch/unwatch avoids this only by holding the replacement mutex across foreign calls.

## Required target architecture

1. One immutable producer-owned `WorldChangeGeneration` carries typed changes, affected routes,
   canonical order, inspection generation and explicit overflow/resync.
2. One immutable editor `WorldSyncRouteGeneration` groups by distinct WatchKey. Runtime subscription
   count and wire identities scale with distinct affected routes; multiple views fan out locally.
3. Ready generation/wake suppresses stable polling. Dynamic pages have count/byte/age limits and a
   cursor; oversized or failed transport cannot destructively lose a batch.
4. Runtime11 seals under short session ownership, performs encode/decode outside the lock and returns
   immutable generation-tagged completions. No WorldSync-private pool.
5. Gateway work uses a generation lease and current-generation commit. Foreign work is outside the
   replacement mutex; stale completions never apply and never consume new-session data.
6. Editor05/Layout09 directly queues typed Added/Removed/Moved operations under a frame budget.
   Full reflow is reserved for broad invalidation, overflow or generation gap. Delete the unused
   `editor.world_fact` JSON path.
7. Render17 owns route/queue/lock/encode/decode/project/apply/cancel/overflow/reflow measurements.

## Requested owner-plan updates

### Editor02

Own distinct-key route compilation, local view fanout, ready-only consumption, generation leases and
bounded page/cursor behavior. Preserve deterministic cleanup and malformed diagnostics without
keeping per-view remote tokens as a compatibility path.

### Runtime07 and Runtime08

Publish the typed world change set and immutable inspection generation atomically with the mutation
commit. Runtime direct indexes remain support structures; facts and dirty tokens may not become
parallel replay authorities.

### Runtime10 and Runtime11

Hard-cut the ABI/session execution to ready generation plus bounded pages. Seal/move ownership under
a short lock, serialize outside it and execute on the existing per-session ordered lane. Replacement,
stop and unload cancel old work without waiting for provider wall time.

### Editor05 and EditorLayout09

Consume typed hierarchy operations with stable entity identity and a resumable count/time budget.
Preserve sparse patches and authoritative recovery, but do not reacquire/reobserve the full World for
every dirty route. Filtered views need a measured incremental policy or an explicit reflow reason.

### EditorUI08 and Render17

Retained tick polls immutable current-generation completions only when signaled. Central metrics must
cover idle calls/locks/bytes, route sharing, queue age/peak, operation apply, stale cancellation and
reflow causes; no per-module diagnostic sort on stable frames.

## Acceptance additions

- In-process/dynamic idle at 60/120/240Hz for 10/300s: WorldSync FFI, session/World/subscription/bus
  locks, JSON bytes, topic allocations and scans zero while ready generation is stable.
- Keys 1/100/10K and bindings/key 1/2/16: runtime subscriptions/wire identities equal distinct
  affected routes, not bindings; stable route generation work zero.
- Entities 1/1K/100K with add/remove/move and storms to 1M operations: accepted loss/dup/reorder zero,
  queue hard-bounded, UI apply within budget and full reflow only with typed reason.
- Replacement before/during/after 0/10ms/10s transport: first new-session change exactly once, stale
  apply zero, replacement wait independent of provider delay.
- Current managed Cargo plus F4 WPR CPU/thread/wake/lock p50/p95, allocation/RSS/package power and
  same-machine Unreal comparison required before acceptance.

## Requested protected index state

- `pending.md`: replace the stale module row with one concise
  `zircon_editor/src/core/sync/**` row, `static_complete / dynamic_pending`, current counts and review
  link.
- `review.md`: do not add the module until current Cargo, route-scale gates, F4 WPR and quantified
  CPU/RSS/power close the matrix.

## Milestone and notification state

This is a static architecture review, not an accepted performance milestone. No commit or WeCom
notification is due. Both become required after the dynamic matrix and protected indexes are
accepted by their owners.
