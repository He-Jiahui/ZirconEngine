---
related_code:
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/UnrealEdEngine.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
---

# Protected plan routing: editor extension contribution and overlay

## Reason for routing

`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`, `review.md`, `pending.md` and
the numbered owner plans are protected/foreign dirty in this session. This record preserves exact
current-source corrections without overwriting another owner's work. The evidence source is
`2026-08-15-editor-extension-contribution-overlay-current-architecture-review.md`.

## Requested Performance01 correction

Correct only the editor-extension portion of `PERF-MVP-538`; retain its unreviewed runtime/native
catalog findings. Current stable editor readers use `ContributionSnapshot` backed by shared typed
`Arc<BTreeMap<...>>` generations, so remove the claim that stable chrome/query consumers clone every
active registry or rebuild contribution vectors. Compatibility getters still allocate, but they are
not the established steady-state authority.

Replace that stale subsection with the current P0 registration root cause:

- one shell mutex spans validation, asset replay, candidate preparation, install and publication;
- the complete `ContributionBatch` and prior command registry are cloned;
- asset-type validation rebuilds builtins and clones/reapplies every existing contribution for each
  registration, approaching quadratic replay across sequential plugins;
- viewport provider preparation invokes the foreign `registration.create()` factory while the shell
  mutex is held;
- manager views/modes/providers are installed before contribution-store commit, so optimization must
  share the existing atomicity correction rather than add a parallel cache.

Update `PERF-MVP-538` acceptance with shell/command wait+hold, callback-under-lock count, candidate
rows, clone bytes, replayed asset rows and publish count. The target is one prepared generation:
foreign work outside locks, one generation-checked atomic commit, zero full prior-registry clone and
work near changed rows. Preserve rollback, reload/unload quiescence and plugin failure diagnostics.

`PERF-MVP-595` is already current: targeted shell content queries only one source, while explicit full
host recompute can still call every enabled source. Link the new evidence; do not rewrite it as
unconditional per-frame work. Add source generation/dirty/`NotModified`, visible-demand querying and
entry+byte+deadline bounds to its acceptance.

Add or assign one Performance01 child item for viewport overlay cache-miss amplification. Existing
interaction caching is valid, but on a miss every enabled provider runs synchronously with live
`&Scene` input and has no per-provider wall/output budget, demand generation or last-good result.
Route it to Editor05/Editor12/Plugins01; use Runtime11 only after a provider declares non-main affinity
and consumes sealed immutable input.

## Requested owner-plan updates

### Editor12 and Plugins01

Make registration a prepare/commit transaction over one contribution generation. Capture immutable
input generations under short locks, run pure validation and all foreign factories outside locks,
then verify generations and atomically publish commands, views, modes, providers and contributions.
Provide owner/family/id indexes in this generation and eliminate per-plugin replay of all asset rows.
Reuse the existing plugin-registration atomicity failure record as the correctness gate.

### Editor05

Treat overlay providers as demand- and generation-scoped extract contributors. Measure each provider,
declare selection applicability, bound output entries/bytes and retain last-good immutable output.
Keep live-scene/main-affinity work on the editor owner; do not make a borrowed `&Scene` callback async.

### EditorUI08

Keep the targeted one-source pane path. Extend source snapshots with generation/dirty/`NotModified`
and reserve all-source collection for explicit full recompute/diagnostic export under a count, byte
and deadline slice. Publish one retained-host generation and reject stale results.

### Runtime11

Provide bounded scheduling only for provider/pane callbacks that explicitly declare non-main affinity,
sealed immutable input, output budgets and generation cancellation. Do not introduce a plugin-private
pool or move shell/World authority into worker callbacks.

## Requested protected index state

- `pending.md`: add or retain one concise module row for `zircon_editor/src/core/editor_extension.rs`
  plus `core/editor_extension/**`, with `static_complete / dynamic_pending`, 5/5 files, 1,540 lines,
  5 inline tests and the current review link.
- `review.md`: do not add the module. Managed Cargo, registration scale/lock/clone counters, F0/F4 WPR,
  pane/provider budgets, same-machine latency/RSS/power and real-scene RenderDoc parity are absent.

## Milestone and notification state

This is a static review/routing record, not an accepted performance milestone. No git commit or WeCom
notification is due. Commit and quantified WeCom notification occur only after current-source dynamic
evidence closes the acceptance matrix and the protected indexes are reconciled by their owner.
