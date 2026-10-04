---
related_code:
  - zircon_editor/src/core/extension
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Toolkits/AssetEditorToolkit.h
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/editor_node.cpp
---

# Protected plan routing: editor extension lifecycle and mounted generation

## Reason for routing

The main performance plan, `review.md`, `pending.md`, optimize report and numbered owner plans are
protected or foreign dirty. This record routes the current 30/30-file evidence without overwriting
their owners. Evidence source:
`2026-08-19-editor-core-extension-lifecycle-generation-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-079

Promote the registration optimization from a batch-clone issue to a single mounted-generation
transaction:

- one shell mutex currently spans validation, prior asset replay, plugin overlay preparation,
  complete command-registry clone, family installation and ContributionStore publication;
- the extension batch is cloned before family extraction and again for the Store ticket record;
- views, modes and providers become visible before the contribution ticket and command registry;
- sequential owners copy touched COW family maps and can approach quadratic publication work.

Required target: one lock-free prepare plan, one generation-checked atomic commit, one owner receipt,
zero foreign callback under shell/registry locks, zero prior full-registry replay, and work near
changed owner rows plus affected dependency edges.

### PERF-MVP-538

Link the new evidence and make the sole desired-to-mounted reconciler the shared owner of plugin,
extension and Workbench state. Stable `Arc<ContributionSnapshot>` is a useful read model, but it must
not retain executable callback roots as a second unload authority. Compile capability membership,
Inspector dispatch, field-editor lookup and direct family indexes once per mounted generation.

Acceptance must include desired manager generation versus mounted generation, owner receipt counts,
retained callback roots, stale snapshot readers, disable/reload quiescence, family build counts,
clone bytes and shell/command lock wait/hold.

### PERF-MVP-595

Keep its targeted pane-source and visibility scope. Add the common callback supervisor and owner
retirement fence so pane-data `snapshot()` shares affinity, deadline, bytes, generation cancellation,
fault and last-good policy with Inspector and overlay callbacks. This task does not own the global
extension reconciler.

### New P0 child item: retirement and document toolkit registry

Add one Performance01 child owned jointly by Editor06/12 and Plugins01:

- production has no `ContributionStore::revoke()` caller;
- disable, project close and reload do not retire all family callbacks before native unload;
- `DocumentToolkitRegistry` calls `descriptor()` and can drop trait objects under its mutex;
- every register/close fully rebuilds the descriptor array; clear scans repeatedly and publishes
  after dropping maps under lock.

Required target: owner-scoped reverse receipts, admission close then drain/revoke/unload ordering,
owned descriptors in registry state, no virtual call/drop under lock, one batch-clear publication and
a leak/root census before unload. Do not copy Unreal ModularFeatures' lock-held broadcast or
ToolMenus' global unregister scan.

## Requested owner-plan updates

### Editor12 and Plugins01

Own `EditorPluginManagerSnapshot -> ExtensionReconciler -> ExtensionRuntimeSnapshot`. Prepare all
family candidates and supervised callbacks outside locks, atomically publish one mounted generation,
and retire owner leases in reverse dependency order before binary unload. ContributionStore becomes
manifest/read projection, not callback lifetime owner.

Before implementation acceptance, reconcile the current Editor12 integration contract baseline:
manager/admission/catalog-projection tests currently pass 24/32 and fail 8/32 across capability
transaction typing, lifecycle fixture placement, project/native-loader publication and projection
identity. Do not weaken those assertions to make the suite green; align contracts with the accepted
single-generation architecture and preserve recoverable admission and rollback behavior.

### Editor02, Editor05 and EditorUI08

Editor02 supplies immutable reflection/authoring generations for Inspector work. Editor05 moves
scene/overlay callbacks to measured extraction over sealed inputs. EditorUI08 consumes compiled
Inspector/field/pane handles without rebuilding catalogs on stable chrome and without invoking plugin
code under shell/world locks.

### Editor06

Hard-cut document toolkits to entries containing owned descriptors plus host-neutral lifecycle state.
Prepare external work before commit, publish immutable registry state without trait calls, defer
trait-object drops until after guard release, and preserve dirty/save/close token semantics.

### Editor14 and Runtime11

Provide shared callback/job affinity, deadline, cancellation, output and queue budgets plus trace
counters. They do not own plugin-private pools, extension desired state or Workbench registries.

### Optimize zircon_editor/50

Retain this report as the product-correctness owner. Performance01 supplies the quantitative gates:
owners/rows/callbacks/documents scale, build/clone/lock/root counters, 31-run F0/F4 WPR distributions,
allocator/RSS/package power and viewport RenderDoc parity where rendering behavior changes.

## Requested protected index state

- `pending.md`: add or retain one concise module row for `zircon_editor/src/core/extension/**` with
  30/30 files, 5,267 lines, 38 tests, fingerprint `2425d286...`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require one desired-to-mounted reconciler, product revoke,
  callback quiescence, lock-free foreign execution, non-quadratic generation publication, toolkit
  lock hard cut and the complete dynamic acceptance matrix.

## Milestone and notification state

This is static architecture evidence and cross-plan routing, not an accepted milestone. No git
commit or WeCom notification is due. Both become mandatory after implementation, managed dynamic
acceptance and protected-index reconciliation are complete.
