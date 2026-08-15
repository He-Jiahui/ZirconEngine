---
related_code:
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/v2_design_tokens.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/ConsoleManager.cpp
  - dev/godot/editor/settings/editor_settings_dialog.cpp
---

# Protected plan routing: settings file generation and hot projection

## Reason for routing

Performance01, `review.md`, `pending.md` and the owner plans are protected/foreign dirty in this
session. This record requests current-source corrections without overwriting their owners. Evidence
is `2026-08-15-editor-core-settings-file-generation-current-architecture-review.md`.

## Requested Performance01 corrections

Replace the stale settings accounting with 16/16 current Rust files, 3,937 physical lines, 34 tests,
zero ignored and ordinal fingerprint
`a8a4570756f9a33280550c6d14df9ef50780d38ab6576fbc0682c70987a8c71e`. The old 9-file/1,625-line/12-
test row predates authority/snapshot/startup owners, the bounded persistence lane and split tests.

Correct `PERF-MVP-590` rather than retaining the old synchronous-UI diagnosis:

- UI/frame callers now perform zero filesystem work; Runtime11 owns a bounded, globally serial
  settings lane with tickets, retry, cancellation, fence and shutdown;
- the remaining P0 is that lane identity includes setting key while output rewrites the complete
  physical file, so distinct-key bursts do not coalesce and can repeat complete encode/write/flush;
- full encoding executes under the settings authority mutex, so worker serialization can still
  delay UI mutations even though disk I/O is off-thread;
- request generation is a trigger, not sealed file bytes, so per-key tickets cannot precisely state
  the durable file generation. The fix must hard-cut receipts and viewport tracking to file
  generation; omitting the key from `lane_key` alone is not accepted.

Correct `PERF-MVP-591` because one production authority, bounded change history, registration-time
built-in slots and shared unchanged snapshot payloads now exist. Replace the old three-authority/
unbounded-journal diagnosis with:

- every stable retained tick still loads the settings snapshot and takes the global V2 token
  projection write lock; the sole subscriber is already consumed by locale hot apply;
- keymap lookup retains a projection mutex and the legacy snapshot accessor deep-clones the complete
  keymap;
- persistent bulk replacement clones the old map/changed-key set and builds one transient snapshot
  per changed key;
- changed large built-in payloads are deep-cloned between registry and snapshot instead of sharing
  one immutable owner.

Keep the settings current-text double parse linked to `PERF-MVP-570`/Editor11. Do not add a
settings-private envelope parser.

## Required target architecture

1. Editor17 owns one monotonic `SettingsFileGeneration` per `(scope, canonical physical path)`, with
   latest dirty/durable/failed generation, changed-key mask and immutable/shared values.
2. Runtime11 permits one running plus at most one latest pending settings generation per file.
   Interactive mutation uses measured debounce; explicit Apply/Close/project switch/shutdown fences
   bypass it. Encode occurs outside authority/project locks.
3. File-generation receipts replace per-key durability semantics. A newer successful generation
   satisfies all older included key changes; retry/cancel/failure names the exact file target and
   generation.
4. Preserve the atomic writer and failure semantics, add unchanged durable-byte suppression, and
   retain dirty state until the latest generation succeeds.
5. Authority publishes one affected-slot mask through a fanout dispatcher after unlocking. Stable
   retained frames do no settings load/lock/Arc work; design tokens and keymap publish shared
   immutable handles only when changed.
6. Bulk layer replacement takes/moves old state, performs sorted linear diff and publishes one final
   snapshot/mask. Large typed values share immutable bodies across registry, snapshot and persistence.

## Requested owner-plan updates

### Editor17

Revise the open settings persistence/hot-projection failure record. Preserve the completed sole
authority, bounded journal, typed slots and off-UI I/O work, then own the file-generation projection,
dirty/durable state, dispatcher/mask, bulk replace and shared-value hard cut. No compatibility
per-key durability path remains after migration.

### Runtime11

Reuse `BoundedKeyedIoLane`, but expose the file-generation/latest-pending semantics Editor17 needs
without weakening global entry/byte/fence/shutdown guarantees. Do not add an editor-private thread
pool. Diagnostics must distinguish admitted triggers, coalesced file generations and executed writes.

### Editor05

Replace viewport's per-scope/key ticket tracking with file-generation receipts. Rapid snap-step
updates must converge into the newest Project file generation; project close waits on the relevant
file fence and reports exact terminal failure.

### EditorUI08

Make settings projection change-driven. The retained tick must not acquire the global design-token
write lock on stable frames. Publish `Arc<EditorKeymap>` and token handles once per affected
generation, and keep main-thread application to the small changed-slot commit.

### Editor11 and Editor12

Editor11 removes the current-text double parse through the shared bounded versioned reader.
Editor12 ensures plugin definitions and large values enter the same compiled settings generation and
file-value budgets; it must not create a plugin-private settings store or serializer.

### Render17

Own file generation, dirty age, admitted/coalesced/executed work, encode/write/flush counts and
bytes, authority/project lock wait/hold, durable generation, projection dispatch/apply, clone bytes,
RSS and power counters. WPR product traces compare Zircon and the local Unreal editor on the same
machine, scene/project, frame cap and power plan.

## Acceptance additions

- Distinct keys 1/10/1K at 1/60/1K updates/s: encode/write/flush scale with committed file
  generations, not keys; one running plus at most one latest pending generation/file.
- Keys 7/1K/100K and values 0/1KiB/1MiB: full encode under authority/project locks zero; UI
  filesystem wall zero; final durable bytes include every accepted change.
- Failure before write/flush/rename, retry/cancel/shutdown/project switch: receipt identities exact,
  newest accepted generation succeeds exactly once or returns explicit failure, stale project apply
  zero.
- Stable 60/120/240Hz for 10/300s: settings snapshot loads, projection locks, Arc clones and theme/
  keymap rebuilds zero until an affected generation changes.
- Bulk 1/1K/100K values: old-map full clone zero, sorted diff near linear, final snapshot
  publications one, each built-in recalculated at most once.
- Current managed Cargo plus F0/F4 WPR CPU/thread/wake/lock/file-I/O p50/p95, allocation/RSS/package
  power and same-machine Unreal comparison are mandatory before acceptance.

## Requested protected index state

- `pending.md`: replace the stale module row with one concise
  `zircon_editor/src/core/settings/**` row, `static_complete / dynamic_pending`, current counts and
  review link.
- `review.md`: do not add the module until current Cargo, file-generation scale gates, F0/F4 WPR and
  quantified CPU/RSS/power close the matrix.

## Milestone and notification state

This is a static architecture review, not an accepted performance milestone. No commit or WeCom
notification is due. Both become required after the dynamic matrix and protected indexes are
accepted by their owners.
