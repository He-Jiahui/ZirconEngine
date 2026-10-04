---
related_code:
  - zircon_editor/src/ui/workbench/window_registry
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/layout_hosts/collect_instance_hosts.rs
tests:
  - zircon_editor/src/tests/workbench/registry/window_registry.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
doc_type: current-architecture-performance-review
status: static_complete_dead_mirror_hard_cut_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor Workbench window registry dead mirror and layout generation review

## Status

- Result: `static_complete / dead_mirror_hard_cut_required / dynamic_pending`.
- MVP priority: P0 for removing the unconsumed production rebuild; P1 for typed layout generation
  and future window/plugin-page ownership.
- Accounting: retain `zircon_editor/src/ui/workbench/window_registry/**` in `pending.md`. Do not add
  it to `review.md` until the owner conflict and product gates below close.
- Code disposition: no Rust source changed. The current deletion is small, but EditorLayout07 still
  plans to expand this registry as the unique window authority. That protected plan must be corrected
  before a hard cut so later tab/plugin work cannot recreate the same parallel authority.

## Exact scope

| scope | files | physical lines | tests | raw bytes | sorted path-LF-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/window_registry/**` | 9/9 | 481 | 1 in-module | 15,393 | `f5b043fdeceb87579eca6312a4f572987c496ef8a9a2de4529038a6f7b1297e5` |
| focused registry and preset caller | 2/2 | 507 | 8 | 18,395 | `ac788e1fe2978555fec8076720e94991fadaed94d95c3665264372131b172e45` |

The fingerprint is SHA256 over each sorted normalized path, LF, then raw file bytes. All nine
production files, the focused test file and the preset caller were read in full. Product use was
traced through `EditorUiHost`, every session metadata recompute site, preset construction and the
native window host boundary.

## Current acceptance record

| area | current-source verdict |
|---|---|
| production ownership | `EditorUiHost.window_registry` is initialized and replaced, but never read by production code. It is an unconsumed mutable mirror. |
| rebuild path | Every changed layout metadata transaction collects all placements, deep-clones all live instances, rebuilds four ordered maps and copies drawer/window strings and IDs. |
| registry builder | Instance rows are borrowed only after the caller has already deep-cloned them. Repeated register/retain during a clean rebuild can make one drawer bucket quadratic. |
| preset caller | `default_window_registry()` creates an ephemeral registry used by preset/tests. That cold verifier does not justify a permanently rebuilt host mirror. |
| tests | Six external behavior tests plus one source guard use tiny fixtures. The source guard checks only that `instances_by_id()` does not clone rows and misses its upstream full clone. |

## Structural bottlenecks

### P0: the live host rebuilds a registry with zero production readers

`EditorUiHost` owns `Mutex<EditorWindowRegistry>`, exposes a private lock helper, initializes the
field, and replaces it in `recompute_session_metadata()`. Repository-wide current-source search
finds no production read of that field or lock. Product window creation/sync instead consumes
`WorkbenchLayout` through `WindowHostManager` and retained presenters.

This is not a speculative cache miss. It is provably dead work on every changed focus, attach,
close, restore, reset and persistence recompute. The immediate hard-cut target is to delete the host
field, lock helper, initialization and rebuild call. Keep an ephemeral preset validation builder
only if its tests still express useful schema invariants. Do not replace the dead mirror with another
cache until a real consumer and generation contract exist.

### P0: the dead rebuild pays for arbitrary document payload bytes

Before `sync_from_layout()`, `recompute_session_metadata()` clones every session `ViewInstance` into
a vector. That clone includes arbitrary `serde_json::Value`, title, descriptor/instance IDs and host
path. The registry only reads ID, descriptor and title, yet unrelated document payload bytes are
copied for a result that is never consumed.

Inside `sync_from_layout()`, `instances_by_id()` builds a `BTreeMap` by cloning every instance ID.
It then copies IDs/titles/descriptors into window/drawer records and ordered maps. The existing
`window_registry_indexes_instances_without_cloning_rows` source test starts after the damaging
upstream clone and therefore cannot prove the product path allocation-free.

### P1: full rebuild complexity compounds layout metadata work

The containing transaction already traverses the full layout to collect placements and updates all
live hosts. The registry then traverses activity windows, drawer tab stacks and floating windows
again. During clean construction, `register_drawer_view()` performs `retain()` on the destination
bucket before each push even though no duplicate can exist; D tabs in one bucket produce O(D^2)
comparisons. Legacy/noncanonical `activity_windows()` may also allocate a canonical map.

After the registry replacement, the same transaction clones active drawer layouts, retains editor
session maps and syncs native windows. Deleting the dead mirror removes one complete pass. The
remaining owner must publish typed layout deltas so focus is near O(1) and dock/close work is bounded
by the affected subtree rather than another full-layout cache.

### P0 plan conflict: EditorLayout07 would promote dead derived state into a third authority

EditorLayout07 currently says the window registry will become the unique source of truth and should
not be rewritten. Current product evidence contradicts that premise: `WorkbenchLayout` is the
persisted topology/mutation authority, session state owns live instances, and `WindowHostManager`
owns OS window state; the registry is not read.

Before implementing PurposeView, Chrome tabs or drawer detach/reattach, EditorLayout07 must hard-cut
to the shared ownership model: layout generation owns persisted tab/drawer/window topology; session
generation owns live instance metadata; native host generation owns OS handles. Consumers receive
immutable indexes/deltas from those generations. A builder named `EditorWindowRegistry` may remain
for cold preset validation, but it must not become a separately mutable product authority.

## Reference-engine evidence

- Unreal `TabManager.cpp:1220-1288` performs the full `RestoreFrom()` layout walk at the explicit
  restore boundary, finishes restore, then returns the live docking area. It does not rebuild a
  second unused registry after every focus or tab operation.
- Unreal `TabManager.cpp:3171-3215` handles foreground, relocate, open and close as explicit events.
  Relocation updates the affected docking areas, stats and menus, then requests persistence; open
  and close request persistence without reconstructing every tab/window descriptor table.
- Unreal `TabManager.cpp:1164-1185` coalesces persistent-layout requests, while
  `1197-1218` maintains the separate tab-spawner registry through explicit register/unregister.
  This supports distinct descriptor lifecycle and live layout ownership, not an unconsumed mirror.
- Unreal `TabManager.h:485-514` persists compact tab identity/state. Live tab/widget state remains
  with docking owners, which supports the layout/session/native-host split above.

These references establish lifecycle and ownership boundaries, not Zircon performance parity.
Current-product counters, WPR/ETW and fault tests remain required.

## Required hard cut

1. Delete the unconsumed `EditorUiHost.window_registry` field, lock helper, initialization and full
   rebuild. Prove repository-wide that no production reader remains and dead mirror resident bytes
   and rebuild count are zero.
2. Remove the aggregate `Vec<ViewInstance>` clone from metadata recompute. Layout/session indexing
   uses borrowed or immutable generation data and never copies arbitrary payloads.
3. Make layout apply return typed affected instance/window/drawer/page/focus deltas. Maintain only
   indexes with real consumers; focus does not traverse all placements or drawers.
4. Keep preset registry construction cold and explicit, or replace it with direct layout invariant
   validation. Its types cannot leak into a second live authority.
5. Rewrite EditorLayout07/08 milestones so PurposeView, plugin pages, tabs and drawer movement commit
   through the canonical layout/session/extension generations and reconcile native hosts once.
6. Coalesce grouped resize and multi-view close into one transaction, one metadata delta, one native
   host sync and one persistence request.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Counters for metadata calls, placement/window/drawer/instance visits, payload clone bytes, registry builds/resident bytes, bucket retain comparisons and native sync. | current source re-read |
| M1 | Hard-delete the dead host registry mirror and upstream full-instance clone; preserve preset validation separately. | EditorUI08 + EditorLayout07 correction |
| M2 | Typed layout delta and generation-owned indexes for focus/dock/close/resize. | EditorUI08 + Optimize13 |
| M3 | PurposeView/plugin page/tab/drawer lifecycle on canonical layout/session/extension generations. | EditorLayout07/08 + Optimize50 |
| M4 | Current-source managed Cargo/F4 and WPR/ETW CPU/allocation/lock/power matrix. | M0-M3 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| dead mirror | focus/attach/close/restore/reset with views/windows/drawers `1/100/1k/100k` | host registry builds, reads, resident bytes and payload clone bytes `=0`; layout/window behavior unchanged |
| layout delta | same focus, changed focus, dock, detach, drawer selection/resize, page switch | stable/no-op work `=0`; focus visits near O(1); dock/close visits bounded by affected subtree; one native sync/publish |
| grouped actions | left/right resize and close tabs `1/8/128/1k` | one transaction, metadata delta, host sync, persistence request and invalidation per group; failure atomic |
| preset/plugin | builtin presets, plugin view enable/disable/reload, unknown view restore | cold validation only; no second live authority; deterministic topology and bounded placeholder/reconcile parity |
| product | F4 idle and focus/dock/resize/close/restore storms, 31 runs | WPR/ETW CPU, allocations, lock hold/wait, input-to-pixel p50/p95/p99, RSS and package power on identical hardware/config; artifacts only on D/E/F |

RenderDoc is required only if the hard cut changes rendering-visible resources, draw order or pixels.
It cannot establish removal of dead CPU work, JSON clone bytes, lock cost or package power.

## Static gates executed

- Read 9/9 production files, the full focused registry test and preset caller; reproduced the line,
  byte, test counts and both current-source fingerprints above.
- Proved by repository-wide call search that the live host field has one initializer and one writer,
  with zero production readers; traced all `sync_from_layout()` call sites.
- Traced all ten session metadata recompute sites and the surrounding placement, active drawer,
  editor session and native window synchronization work.
- Read the cited Unreal restore boundary, exact tab lifecycle, coalesced persistence and compact tab
  state ranges. Exact Zircon paths currently report zero coordinator leases and no local dirty files.
- No Cargo lane, F4 launch, WPR/ETW, package-power or RenderDoc capture was run. Dynamic acceptance
  remains pending; RenderDoc is not applicable because no rendering-visible source changed.

## Completion rule

This module remains pending until M0-M4 pass against a current source fingerprint and EditorLayout07
no longer declares the dead mirror as product authority. A source-text no-clone guard, tiny fixture
or unused cache rebuild is not acceptance. No milestone commit or WeCom completion message is
permitted before quantified product evidence.
