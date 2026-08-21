---
related_code:
  - zircon_editor/src/ui/workbench/layout
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/workspace_state.rs
tests:
  - zircon_editor/src/tests/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/tests/workbench/layout/document_attachment.rs
  - zircon_editor/src/tests/workbench/layout/drawer_attachment.rs
  - zircon_editor/src/tests/workbench/layout/drawer_extent.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/layout_commands.rs
  - zircon_editor/src/tests/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
  - zircon_editor/src/tests/workbench/layout/page_layout_templates.rs
  - zircon_editor/src/tests/workbench/layout/roundtrip_and_restore.rs
  - zircon_editor/src/tests/workbench/layout/split_creation.rs
  - zircon_editor/src/tests/workbench/layout/window_drawer_ownership.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
doc_type: current-architecture-performance-review
status: static_complete_atomic_authority_hard_cut_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor Workbench layout atomic transaction and location-index review

## Status

- Result: `static_complete / atomic_authority_hard_cut_required / dynamic_pending`.
- MVP priority: P0. Invalid attach, move or split can mutate the live layout before returning an
  error; ordinary successful commands also amplify one local edit into repeated full-layout scans,
  drawer normalization, deep copies and host metadata rebuilds.
- Accounting: keep `zircon_editor/src/ui/workbench/layout/**` in `pending.md`. It cannot enter
  `review.md` before atomicity, locality, current-source Cargo and product profiling gates pass.
- Code disposition: no Rust source changed. One production file and five focused test files in this
  exact scope have foreign formatting changes. More importantly, replacing the split clone alone
  would leave the invalid transaction order, duplicate authority and global host recompute intact.

## Exact scope

| scope | files | physical lines | tests | raw bytes | sorted path-LF-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/layout/**` | 39/39 | 1,891 | 6 in-module | 64,032 | `620b5eec8c1a535aa42610c1cb1bfa1bc600c7b834b8d180438ca45f262a1813` |
| focused layout tests | 11/11 | 1,253 | 30 | 42,780 | `86207291ee70ab77018122cdeb7906d89691759914b9a576d0b805a0f855df54` |

The fingerprint is SHA256 over each sorted normalized path, LF, then raw file bytes. All 39
production files and all 11 focused test files were read in full. The product path was traced
through `EditorUiHost::apply_layout_command_inner`, workspace capture/restore and
`recompute_session_metadata`; broader persistence, dirty-document and native-window findings remain
owned by Optimize13.

## Current acceptance record

| area | current-source verdict |
|---|---|
| authority | `WorkbenchLayout` serializes both legacy `drawers` and canonical `activity_windows[*].activity_drawers`; default construction and every changed command maintain both copies. |
| command order | Open/Move/Attach and CreateSplit call global detach before target lookup and validation. An invalid target returns `Err` after the old placement has already disappeared. |
| local mutation | CreateSplit clones the complete target subtree before replacing it. Tab insertion retain-scans the target stack and may scan it again for the anchor. |
| global work | Detach scans all activity-window drawers, every main-page document tree, every floating tree and both page/window vectors. Focus repeats a global ordered search across the same domains. |
| post-processing | Every changed command scans every drawer to normalize active selection, including repeated linear membership checks, then deep-clones the active drawer map into the legacy mirror. |
| host amplification | The host performs another legacy drawer sync and unconditionally calls full `recompute_session_metadata`, even when the manager returns `changed=false`. That path recollects all placements, clones all open instances, rebuilds the window registry and resynchronizes native windows. |
| diff contract | `LayoutDiff` contains only one Boolean. It cannot identify changed stack, page, drawer, window, focus, placement or persistence domains, so downstream incremental work is impossible to prove. |
| restore | `LayoutManager::restore_workspace` only selects and returns one owned layout. Validation, migration, staging and rollback are absent; host restoration later performs destructive live-state replacement. |
| tests | 36 focused tests cover small successful layouts, typed error values, serde and preset behavior. No test asserts failure byte-for-byte no-op, exact touched IDs, clone/visit counts, scale, generations or host recompute suppression. |

## Structural bottlenecks

### P0: a failed command is not a transaction

`manager/apply.rs:20-34` and `79-86` detach the instance before `attach_instance()` or
`workspace_node_mut()` validates the target. The validation failures are explicit in
`manager/attach.rs:20-73`, but they arrive after live mutation. Because the final normalize/mirror
step only runs for `Ok(changed=true)`, an error can leave the canonical layout changed while the
legacy mirror remains stale. `OpenView` additionally has a session/registry orphan risk in its host
caller, as Optimize13 records.

The required algorithm is an explicit transaction, not clone-and-rollback:

1. resolve source placement and target against one immutable generation;
2. validate target shape, anchor, identity, owner and dock policy without mutation;
3. prepare a minimal edit script and acquire required session/registry/document leases;
4. apply the edit to affected stacks/subtrees and update indexes once;
5. validate invariants, then atomically publish one generation and typed delta;
6. abort with zero live-state, registry, focus, projection, native-target or persistence change.

### P0: one local command is implemented as whole-layout maintenance

Let A be activity windows, D their drawers, T drawer tabs, P main pages, H document-tree nodes, W
floating windows and V live instances. A move/attach currently pays approximately:

```text
detach          O(sum(A*D*T) + sum(P*H) + sum(W*H))
target attach   O(target lookup + target stack + anchor scan)
normalize       O(sum(A*D*T))
legacy mirror   O(active drawer/tab bytes)
host metadata   O(all placements + V + window-registry/native-window projection)
```

The host repeats the legacy mirror after the manager already did it. Open/Move/Attach always report
`changed=true`, including a same-placement operation, while unchanged focus/extent/mode still pay
the host mirror and metadata rebuild. Closing K floating tabs through the current caller therefore
compounds into at least K whole-layout/session passes, consistent with PERF-MVP-602.

The target is one canonical placement graph with stable node/stack IDs and an index from instance ID
to `{host, node, slot}`. Ordinary focus becomes an indexed lookup plus affected-row state change;
move/close touches source stack, target stack and their ancestors; a split moves the old node instead
of recursively cloning it. Index maintenance belongs to the same transaction and must diagnose
duplicate placements rather than silently accepting traversal order.

### P0: duplicate drawer authority turns compatibility into steady-state cost

`WorkbenchLayout` derives serde over both drawer representations. `activity_windows()` may allocate
and canonicalize an owned map for legacy/noncanonical input, `active_activity_window_drawers()`
always clones the active map, and `sync_legacy_drawers_from_active_activity_window()` assigns that
clone back to `drawers`. This is a permanent dual-write scheme, not a migration boundary. The host
then clones the layout again for project workspace capture.

Canonical schema v2 must write only `activity_windows[*].activity_drawers`. Legacy drawers are an
input migration type consumed before live publication and never regenerated by current writes.
Projection consumers receive immutable generation handles or typed deltas, not a second mutable
map. This aligns with Optimize13 M1/M2 and prevents a cache from becoming a third authority.

### P0: Boolean diff forces broad downstream recompute

The manager knows which command, source and target changed, but reduces the result to
`LayoutDiff { changed }`. The host therefore reconstructs placement maps, retains instance tables,
clones all remaining instances, rebuilds the window registry and resynchronizes native hosts.
Introduce a commit result containing old/new generation plus bounded affected IDs and domain bits,
for example placement, focus, active page, drawer state, floating topology and persistence dirty.
Unchanged transactions publish nothing. Consumers update from the same commit result; they must not
rescan the layout to rediscover what the transaction already knew.

### P1: normalization is incomplete yet runs too broadly

Current normalization scans all drawers but ignores its `ViewRegistry`, emits no placeholders and
does not validate duplicate IDs, duplicate placements, empty split repair, path ambiguity, numeric
finiteness or dock policy. It is simultaneously expensive for ordinary commands and insufficient at
untrusted restore boundaries. Split the responsibilities:

- command prepare enforces local preconditions and preserves invariants incrementally;
- bounded schema migration/validation runs at deserialization/import boundaries;
- explicit repair produces typed diagnostics and is idempotent;
- debug verification may scan the whole graph, but is not a mandatory release hot path.

## Reference-engine evidence

- Unreal `SDockingTabStack.cpp:252-315` handles close/open against the owning persistent stack and
  live tab well. `1410-1527` updates the local stack's persistent tab array for open, move, close and
  remove rather than rebuilding a global layout DTO.
- Unreal `TabManager.cpp:3171-3208` has explicit foreground, relocate, opening and closing lifecycle
  callbacks. Relocation updates affected dock areas/menu/stats and requests persistence; it does not
  serialize, clone and restore the whole editor layout for each action.
- Unreal `TabManager.cpp:1164-1185` coalesces layout persistence with a deferred ticker specifically
  to avoid resize hitches. `1220-1288` treats restore as an explicit traced boundary and records
  invalid/collapsed areas instead of silently erasing all unknown state.

This evidence supports local stack ownership, explicit lifecycle and coalesced persistence. Unreal
still uses local linear array scans and relocation visits registered dock areas, so it is not proof
of global O(1) behavior. Zircon's acceptance target is affected-stack/subtree work with indexed
placement lookup and bounded ancestor updates, not an unsupported asymptotic claim.

## Required hard cut

1. Define `LayoutAuthority`, stable node/stack IDs, one placement index and one monotonic generation.
2. Define `LayoutTransaction` with immutable preflight, generation precondition, prepare/commit/abort,
   leases and typed diagnostics. All failure paths are byte-for-byte live-state no-ops.
3. Replace Boolean-only `LayoutDiff` with a bounded typed commit delta carrying affected identities
   and domain bits; host metadata, window registry and native projection consume that delta.
4. Remove the current serialized legacy drawer field after a one-way bounded migration. Never
   maintain compatibility mirrors in steady state.
5. Move existing subtrees on split; mutate only source/target stacks and ancestors. Keep local Vec
   ordering where appropriate, but do not global-scan every placement to find one known instance.
6. Coalesce layout persistence from committed generations. Same placement/focus/extent/page input
   returns unchanged before normalization, clone, metadata, event and persistence work.
7. Stage workspace/layout restore and atomically swap only after schema, registry, plugin placeholder,
   dirty-document and native-target preparation passes, as owned by Optimize13.
8. Add baseline counters and failure invariants before implementation, then preserve all current
   successful behavior tests.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Counters and failure fixtures for visited windows/drawers/nodes/tabs, clone bytes, metadata/registry/native sync, generations and touched IDs. | current source recheck |
| M1 | Stable layout IDs, placement index and typed commit delta with no host rediscovery scan. | EditorLayout07 + EditorUI08 |
| M2 | Atomic command prepare/commit/abort across layout, session and registry; invalid targets and stale generations are no-ops. | Optimize13 M1 |
| M3 | Canonical schema v2 and one-way legacy drawer migration; staged validated restore. | Optimize13 M2/M4 |
| M4 | Generation-coalesced persistence and delta-driven window/presentation consumers. | Optimize13 M3/M6 + EditorUI08 |
| M5 | Current-source Cargo/F4 plus WPR/ETW allocation, lock, latency, RSS and package-power matrix. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| atomic errors | every typed Attach/Move/Open/CreateSplit/Detach error; stale generation; injected registry/host failure | layout, registry, session, focus, generation, events, native targets and durable state byte-for-byte unchanged |
| locality | tabs/nodes/windows/views `1/100/1k/10k/100k`; depth `1/8/64`; source/target same/different | lookup near O(1); visits and clone bytes bounded by source/target stacks plus ancestors; whole-layout scans and subtree clone bytes `=0` |
| no-op | repeated focus/move/attach/page/mode/extent `1/1k/1M` | transaction, normalization, mirror, metadata, registry, native sync, event and persistence work `=0` |
| schema | legacy/current, duplicate IDs/placements, unknown plugin, NaN/Infinity, over-budget depth/count/bytes | one canonical drawer authority; deterministic bounded migration/validation; placeholder or typed rejection; second normalize has zero diff |
| batching | close/drop/resize groups `1/8/128/1k` | one prepare, commit, generation, metadata delta, event and persistence request per logical action; failure is all-or-nothing |
| product | F4 idle, focus, dock, split, float, close, page switch, restore and plugin missing/reload; 31 runs | WPR/ETW CPU, allocation, lock wait/hold, input-to-pixel p50/p95/p99, RSS and package power reported on identical hardware/config; artifacts only on D/E/F |

RenderDoc is only required after a rendering-visible layout cut to verify pixel/draw parity. It
cannot prove transaction atomicity, CPU traversal complexity, clone bytes, lock contention or power
by itself.

## Static gates executed

- Read 39/39 production files and 11/11 focused test files; reproduced the counts and both
  current-worktree fingerprints above.
- Traced manager apply/attach/detach/focus/normalize/restore plus host command, workspace projection,
  metadata and native-window synchronization paths.
- Reproduced detach-before-validation, target-subtree clone, global detach/focus traversal, global
  drawer normalization, duplicate legacy mirror, Boolean diff and unconditional host recompute.
- Confirmed no focused test asserts failure-state immutability, affected identities, visits, clone
  bytes, generations or scale; current error tests only compare the returned typed error.
- Read the cited Unreal local stack, relocate/open/close, deferred save and restore implementations.
- One production file and five focused tests are foreign dirty formatting changes and were not
  edited, reverted, formatted, staged or committed.
- No managed Cargo, F4, WPR/ETW, package-power or RenderDoc capture was run. Shared validation lanes
  remain active, and no source optimization is valid before M0/M1 establish the transaction and
  measurement contracts.

## Completion rule

This module remains pending until M0-M5 pass on one current-source fingerprint. A faster full clone,
a global `Arc<WorkbenchLayout>`, a HashMap added beside duplicate authorities, or successful
small-fixture tests are not acceptance. No milestone commit or WeCom completion message is permitted
before quantified current-product evidence exists.
