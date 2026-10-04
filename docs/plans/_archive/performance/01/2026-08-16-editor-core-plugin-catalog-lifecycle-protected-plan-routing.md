---
related_code:
  - zircon_editor/src/core/plugin
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
---

# Protected plan routing: editor plugin definition, lifecycle and extension publication

## Reason for routing

The main performance plan, `review.md`, `pending.md`, optimize plan and numbered owner plans are
protected/foreign dirty in this session. This record routes the exact current-source findings without
overwriting their owners. Evidence source:
`2026-08-16-editor-core-plugin-catalog-lifecycle-current-architecture-review.md`.

## Existing Performance01 task correction

### PERF-MVP-538

Retain P0 and replace any interpretation of this task as a collection-level clone cleanup with the
current publication-unit finding:

- immutable plugin definitions, runtime active/fault state, transient editor facts and lifetime
  callback history currently share one deep-cloned catalog generation;
- every manager snapshot rebuilds all active extension and asset registries even when structure and
  active set are unchanged;
- project native materialization clones each package registry twice per batch and project open uses
  separate registration and manifest publications;
- admission owns/clones manifests and uses recursive string-keyed DFS.

Required target: `EditorPluginDefinitionGeneration` + compact `EditorPluginRuntimeGeneration` +
`CompiledEditorExtensionGeneration {definition, active_set}` + one `CompiledProjectPluginPlan`.
Only discovery/project/reload changes definitions; only active-set changes rebuild compiled
extensions; transient callback success preserves both identities. Native batches use one candidate
owner, admission uses an iterative indexed graph, and one accepted project transaction builds and
publishes once.

### PERF-MVP-594

Retain P0 and add the exact current lock/queue proof:

- retained tick drains the entire lifecycle subscriber into a second unbounded queue;
- the bridge holds the pending mutex while callbacks run;
- manager holds `lifecycle_mutation` across every foreign callback;
- every callback clones entries/catalog, linearly finds registrations, appends unbounded history and
  publishes a new catalog/manager generation;
- current tests intentionally require both generations to increment for a routine external event.

Required target: one count+byte+deadline-bounded delivery page; short-lock active handle/generation
snapshot; callbacks outside bridge and manager locks; generation-checked fault/cancel commit; compact
latest lifecycle state and bounded count+byte+age diagnostics. Successful transient events perform
zero structural builds and retain no structural history.

Do not create `PERF-MVP-647`: the current evidence strengthens `538/594` rather than identifying an
independent root cause.

## Requested owner-plan updates

### Editor12

Own the hard cut between immutable plugin definitions and mutable runtime lifecycle state. Replace the
test contract that increments catalog generation for routine lifecycle broadcasts. Preserve phase,
failure/retry, hot-reload, rollback and old-reader semantics with explicit definition/runtime/active
set identities.

### Editor06

Own `CompiledEditorExtensionGeneration`. Build direct contribution/command/asset indexes once per
`{definition, active_set}` and share them across diagnostic and transient-event generations. Stable
reads and unrelated callbacks must show zero registry rebuilds and stable `Arc` identity.

### Editor02

Own a single bounded lifecycle subscription/cursor contract. The bridge must not create an unbounded
secondary queue or hold its mutex across callbacks. Delivery exposes count, owned bytes, deadline,
oldest age, lag and deferred reason.

### Plugins01 and PluginCallBridge11

Compile native discovery/load output and all accepted serialized batches into one project candidate.
Remove nested per-batch registry clones, preserve native callback lease/quiescence and keep the native
loader as one backend authority. Integrate with Plugins01's existing release/profiling ETW plan rather
than creating an editor-only benchmark or second discovery cache.

### Runtime11

Provide scheduling only for plugin callbacks that explicitly declare non-main affinity. Reuse the
bounded scheduler with cancellation, single-flight admission and generation checks. Main-affinity
callbacks remain on the editor owner under the same delivery deadline; no private plugin thread pool
is permitted.

### Optimize zircon_editor/01

Add a P0 plugin-control-plane dependency before retained-host F0/F4 acceptance: every retained tick
currently pumps an unbudgeted bridge, and any lifecycle fact can rebuild the full plugin/extension
product before UI recompute. The optimize plan's warmup/measured/quiescence evidence must include
plugin callbacks, queue bounds, definition/extension build counters and project-open publication
count; otherwise UI CPU/RSS/power attribution is incomplete.

## Requested protected index state

- `pending.md`: retain one concise `zircon_editor/src/core/plugin/**` row with
  `static_complete / dynamic_pending`, 35/35 files, 6,318 lines, 51 tests, current fingerprint and the
  2026-08-16 structural-review link.
- `review.md`: do not add the module. Current-source managed tests, release scale counters, project
  open/tick WPR CPU+lock+allocation/RSS/power evidence and pre/post results are absent.

## Milestone and notification state

This is a static structural review and protected-plan routing record, not an accepted milestone. No
git commit or WeCom notification is due. Commit and quantified WeCom notification occur only after
the dynamic matrix passes and protected indexes are reconciled by their owners.
