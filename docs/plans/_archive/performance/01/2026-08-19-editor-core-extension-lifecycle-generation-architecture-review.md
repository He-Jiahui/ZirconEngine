---
related_code:
  - zircon_editor/src/core/extension
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
tests:
  - zircon_editor/src/core/extension/store/tests.rs
  - zircon_editor/src/core/extension/toolkit/tests
  - tools/tests/test_editor02_plugin_registration_atomicity_contract.py
  - tools/tests/test_editor06_document_toolkit_contract.py
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
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Features/ModularFeatures.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenus.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Toolkits/AssetEditorToolkit.h
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/editor_node.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
created_at: 2026-08-19
---

# Editor core extension lifecycle and generation architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for extension admission, disable/reload, Inspector and document lifecycle; P1 for
  large-catalog lookup and snapshot scale after the ownership cutover.
- Accounting: retain `zircon_editor/src/core/extension/**` in `pending.md`. Do not add it to
  `review.md` before the single-generation reconciler, callback retirement and dynamic matrix pass.
- Code disposition: no Rust source changed. `store/tests.rs` contains a foreign owned-string test
  adjustment, and an active session owns overlapping editor source. The primary defects cross
  manager, shell, command, view, scene, toolkit and plugin ownership boundaries.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/extension/**` | 30/30 | 5,267 | 38 | 170,756 | `2425d286df8435193c0bf5aaebcb918a3359c08f7f0937b0dd2ec1515e2fda34` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 30 current Rust
files and all 38 tests were read in full. Production reachability was followed through registration,
Workbench snapshot/reflection, Inspector matching, document save/autosave/close and workspace clear.

## Module acceptance record

| module | scope | current-source performance verdict |
|---|---:|---|
| root and slots | 2 files / 127 lines / 3 tests | Typed finite placement and normalized presets are bounded construction work. No independent hotspot. |
| `inspector/**` | 2 files / 1,198 lines / 11 tests | The chain is deterministic and field definitions are reusable, but matching is linear and executes plugin `can_handle/build` directly. Unqualified misses allocate a lowercase type string; an asset-reference resolve constructs a new `Arc` slice. These are secondary to missing callback isolation and generation compilation. |
| `store/**` | 6 files / 2,590 lines / 12 tests | Immutable `Arc<ContributionSnapshot>`, ticket identity, atomic candidate publication and a bounded change count are worth preserving. Each contribution still deep-clones the full batch for retention, clones every touched existing family map through COW and publishes executable callback roots into both retained batch and snapshot. Product code never calls `revoke()`. |
| `toolkit/**` | 20 files / 1,352 lines / 12 tests | Save I/O correctly runs outside the registry mutex and save/close leases prevent overlap. Register/close rebuild the complete descriptor array under the mutex by calling external `descriptor()`; clear and close may drop callback-owned trait objects while the mutex remains held. The registry is therefore not a safe lifecycle owner. |

## Structural bottlenecks

### P0: desired plugin state and mounted extension state are separate authorities

The Plugin Manager publishes desired active extensions, while `register_editor_extension_owned()`
directly installs views, commands, scene modes, overlays and ContributionStore entries into the
Workbench. There is no production reconciler from desired manager generation to mounted generation.
`ContributionStore::revoke()` has no production caller; `contribution_owners` only appends tickets and
locates a ticket for template replacement.

Disable, project close and reload can consequently change manager state without retiring command,
view, Inspector, pane, scene, overlay or runtime-consumer callbacks. Old immutable snapshots are
correct as data snapshots but become unsafe code roots if a native module is unloaded first. This is
both a correctness defect and a performance defect: stale callbacks continue receiving work and no
owner-scoped resource census can explain CPU, wakeups or retained memory.

### P0: registration is a long shell-locked, multi-registry transaction without atomic commit

`register_editor_extension_owned()` holds the Workbench shell mutex across prior-registry validation,
asset registry replay, plugin overlay preparation, command registry cloning, view/mode/provider
installation and ContributionStore publication. The overlay factory is invoked inside that shell
critical section. Views, scene modes and providers are installed before the ticket is committed; the
complete command registry is replaced afterwards.

The extension batch is cloned before family extraction, then ContributionStore clones it again for
the ticket record while moving another copy into the published maps. Failure after an early install
has no common rollback receipt. Sequential plugin admission therefore combines long foreign work
under one global UI lock with repeated prior-state cloning/replay and split commit points.

### P0: Inspector callbacks run in shell/world critical paths and stable chrome rebuilds catalogs

Every `build_chrome_for_shell()` call reconstructs the capability `BTreeSet`, clones every enabled
Inspector callback into a new chain, revalidates it and rebuilds the ID set. It also reconstructs the
builtin field-editor map and clones all enabled field definitions. The Inspector snapshot then enters
the authoring-world access closure and invokes each customization's `can_handle`; field factories run
while reflected rows are materialized. The direct `inspector_customization()` query invokes
`can_handle` while the Workbench shell lock is held.

The target is not a faster linear scan inside these locks. Inspector dispatch and field-editor
resolution must be compiled once per mounted generation, plugin code must execute through a measured
callback supervisor outside registry/world locks, and presentation must consume sealed scene and
reflection inputs. Existing reflection O(F^2) field joining remains owned by PERF-MVP-567 and is not
duplicated here.

### P0: DocumentToolkitRegistry calls and drops foreign objects while holding its mutex

`publish_snapshot()` calls every toolkit's virtual `descriptor()` and clones every descriptor under
the registry mutex on each register and committed close. `clear()` first scans all entries for saves,
scans again for closes, clones every descriptor, clears both maps and republishes while still locked.
`commit_close()` removes an `Arc<dyn DocumentToolkit>` and returns before the guard is released, so
the final callback-owned object can be dropped under the registry mutex.

A panic or reentrant descriptor/destructor can poison, stall or deadlock the lifecycle authority.
The map may already be mutated before snapshot publication finishes. The positive save lease does
not repair this registration/retirement boundary.

### P1: per-plugin COW publication approaches quadratic work at catalog scale

`ContributionSnapshot` has 18 family maps. A contribution shallow-clones the snapshot, then
`Arc::make_mut` copies each touched existing map before inserting the new owner's rows. With N
sequential plugins contributing to the same family, copied map entries approach O(N^2). The retained
full `ContributionBatch` also duplicates descriptors and callback `Arc` roots already present in the
published snapshot. Template replacement repeats full retained-batch clone and touched-map COW.

The 4,096-entry journal is count-bounded, which is positive, but `changed_since()` clones every
retained change after the requested generation. It is not a substitute for an owner-indexed mounted
generation or a paged diagnostic stream.

## Reference-engine evidence

- Unreal `IModularFeatures.h:118-149` defines paired register/unregister and both lifecycle events;
  access is invalid after unregister. Zircon needs the paired lifetime rule. Unreal's
  `ModularFeatures.cpp:55-80` broadcasts while its critical section is held, so that lock behavior is
  explicitly not a model to copy.
- Unreal `ToolMenus.h:122-128` removes everything for an owner without forcing module load, and
  `527-541` scopes newly created menu entries to an owner. `ToolMenus.cpp:3391-3438` proves the
  owner-scoped removal behavior, but scans all menus; Zircon should use owner-to-family receipt
  indexes rather than copy that O(total menus) algorithm.
- Unreal `ModuleManager.h:83-92`, `295-325` and `522-534` expose loaded/unloaded generations,
  pre-unload/shutdown callbacks and a game-thread module-change event. Zircon should quiesce and
  revoke callback roots before binary unload.
- Unreal `AssetEditorToolkit.h:142-153`, `370-401` and `434-442` treats editor initialization,
  tab registration/unregistration, save/save-as and close reason as one toolkit lifecycle. Zircon
  should keep typed composition, but its toolkit owner must cover prepare, mount, save, close and
  retirement rather than only descriptor plus save hooks.
- Godot `editor_plugin.h:153-172`, `234-256` has paired add/remove APIs for docks, menus, import,
  export, gizmo and Inspector families. `editor_node.cpp:4417-4434` hides, clears, disables, removes
  forwarding registrations, detaches and erases active-plugin references before deletion.

These sources establish ownership and lifecycle shape, not a claim that Zircon matches their timing.
Same-hardware WPR/ETW, allocator and package-power measurements remain mandatory.

## Required architecture cutover

1. Make `EditorPluginManagerSnapshot(manager_generation, desired owners)` the sole desired-state
   input to an `ExtensionReconciler`. App startup, project activation, disable and reload all use it.
2. Prepare one immutable `ExtensionMountPlan` outside shell/registry locks. It validates schema,
   capabilities and dependencies, creates supervised callback handles, and records rollback actions.
3. Commit commands, views, menus, Inspector, templates, assets, scene, overlays and consumers under
   one mounted generation and one receipt. No family becomes visible before the commit fence.
4. On disable/close/reload: stop admission; cancel/drain jobs and callbacks; close active modes and
   toolkits; revoke family leases in reverse dependency order; publish the terminal generation;
   unload native code only after callback, snapshot and job root counts reach zero.
5. Turn ContributionStore into a manifest/read model. Build one frozen generation per reconcile
   batch, compile capabilities to stable IDs/bitsets, keep owner-to-family dense ranges, and retain
   removal receipts instead of a second full executable `ContributionBatch`.
6. Compile Inspector target dispatch and field-editor lookup once per mounted generation. Invoke
   plugin callbacks through affinity/deadline/output budgets over sealed inputs, outside shell/world
   locks, with fault and last-good policy.
7. Store an owned `DocumentToolkitDescriptor` in each registry entry. Prepare descriptor and close
   effects outside the mutex; mutate maps and publish immutable state without virtual calls; defer
   all trait-object drops until after guard release. Batch clear publishes once.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| admission | owners `0/1/100/1k`, families `1/18`, rows `1/100/10k`, success/failure | prepare outside shell; callback-in-lock=0; prior full registry clone/replay=0; commit/publish=1 per desired generation; failure visible rows=0 |
| publication | sequential/batch add, replace, disable, reload, close | snapshot/family builds=1 per mounted generation; stable build=0; owner revoke work near owned rows plus affected edges, not total catalog; retained executable batch copies=0 |
| Inspector | components `1/100/10k`, fields `1/100/1k`, customizations `0/1/100`, stable/1% changed | stable chain/container builds=0; shell/world callback wall=0; dispatch near exact target/priority candidates; normalized type allocation=0; stale generation apply=0 |
| toolkit | documents `0/1/100/10k`, concurrent save/close/clear, panic/reentry/drop stall | virtual-call/drop-under-registry-lock=0; unchanged snapshot build=0; batch clear publish=1; lock hold bounded to map commit; save/dirty/close semantics unchanged |
| retirement | disable/reload/unload during callback, job, save and old snapshot retention | admission closes first; callbacks/jobs drain or cancel within deadline; family receipt roots=0 before unload; no stale callback invocation; leak census explains remaining readers |
| product | F0 startup and F4 editor, cold/warm/reload, 31 runs | WPR/ETW CPU/waits/wakeups/lock/file/process/RSS and package power; allocator clone bytes; UI p95; same assets/settings/hardware; no C: artifacts |

RenderDoc is only required for scene/overlay/Inspector changes that can alter the rendered viewport.
It does not measure this control-plane registration path. Trace counters must first prove the CPU,
lock and callback ownership change; any viewport-affecting cutover then needs RenderDoc output parity.

## Static gates executed

- Read 30/30 current Rust files, all 38 tests and the named production callers.
- Reproduced 170,756 raw bytes and fingerprint `2425d286...` after caller/reference review.
- Confirmed `ContributionStore::revoke()` has no production caller and `contribution_owners` only
  appends/queries tickets.
- Confirmed extension registration invokes foreign preparation and clones/replays registries under
  the Workbench shell mutex; Inspector callbacks execute in shell/world access paths.
- Read the cited Unreal and Godot primary sources. Their lifecycle rules inform the target; their
  lock-adjacent broadcast and global menu scan are explicitly rejected as Zircon algorithms.
- Focused registration/toolkit static contracts passed 8/8. The broader current Editor12 manager,
  admission and catalog-projection contracts passed 24/32 and failed 8/32. Failures cover capability
  transaction type drift, lifecycle fixture relocation, native-loader/project publication contract
  drift and shared projection identity. Relevant plugin/host source is foreign modified; these are
  recorded as owner-integration preconditions rather than repaired in this documentation slice.
- Managed Cargo, F0/F4 product launch, scale counters, WPR/ETW, allocator and package-power evidence
  remain pending. This is not an accepted milestone, so no commit or WeCom notification is due.
