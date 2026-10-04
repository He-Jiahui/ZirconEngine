---
related_code:
  - zircon_editor/src/ui/workbench/model
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
tests:
  - zircon_editor/src/tests/workbench/view_model
  - zircon_editor/src/tests/workbench/reflection/model_projection.rs
  - zircon_editor/src/tests/workbench/fixture/view_model_projection.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenu.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
doc_type: current-architecture-performance-review
status: static_complete_domain_generation_hard_cut_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor Workbench model domain generation and menu compilation review

## Status

- Result: `static_complete / domain_generation_hard_cut_required / dynamic_pending`.
- MVP priority: P0 for the full shell model and command-lock path; P1 for cold empty-state row
  materialization after the P0 ownership cut.
- Accounting: retain `zircon_editor/src/ui/workbench/model/**` in `pending.md`. Do not add it to
  `review.md` until the owner and dynamic gates below close.
- Code disposition: no Rust source changed. The repeated work is produced by the current ownership
  graph, so replacing individual clones with `Arc` or caching the monolithic model would preserve
  stale authority and make plugin revoke, layout mutation and contextual menu state harder to prove.

## Exact scope

| scope | files | physical lines | tests | raw bytes | sorted path-LF-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/model/**` | 44/44 | 1,179 | 0 in-module | 40,084 | `005b3d2a9f8cf845b133b8843aee56c1dda09f94d26e17997b9ae4f27b6e2862` |
| focused model/reflection tests | 7/7 | 1,637 | 17 | 59,467 | `2ac2d68fd6c491e6cfc2db23f3d5dbb013b8eb701b28dccfacbd1422256ea3f3` |

The fingerprint is SHA256 over each sorted normalized path, LF, then raw file bytes. All 44
production files and all seven focused test files were read in full. The product path was traced
through both classic shell reflection and retained-host full recompute, including lock acquisition,
chrome construction, model consumers and committed shell retention.

## Current acceptance record

| area | current-source verdict |
|---|---|
| active page | `active_page_snapshot()` linearly finds and deep-clones the active recursive page, including all tab strings and arbitrary serializable view payloads. |
| page projection | The clone is scanned for host-strip dirty/breadcrumb state, flattened into owned document rows, and finally moved as another recursive document representation into the model. |
| drawers | The complete drawer snapshot is cloned into `drawer_ring`, while the same source is separately projected into owned `tool_windows` rows and empty-state actions. |
| floating windows | Every full model build clones window IDs, titles, focus IDs and flattened owned tab rows. |
| base menus | `menu_bar_model()` calls `menu_model()` for seven fixed labels; each call scans the whole command registry and materializes owned row strings. |
| extension menus | Every contributing build collects and sorts all menu descriptors, reparses path segments, linearly probes menu/branch vectors, recomputes branch enablement and recursively scans the whole menu tree for each contributed view. |
| locks | Classic reflection holds the shell and command mutexes across descriptor/chrome/model/reflection/snapshot publication. Retained full recompute holds the command mutex across the complete model build. |
| tests | The 17 focused tests prove small-fixture content, ordering and capability behavior. The largest extension-menu fixture has two items; no test asserts generation reuse, clone bytes, lock duration, invalidation scope or scale complexity. |

## Structural bottlenecks

### P0: one owned model contains repeated projections of the same domain

`WorkbenchViewModel::build_with_contributions_and_context()` first clones the full active
`MainPageSnapshot`. The host strip then creates owned page titles, IDs, close IDs and breadcrumbs;
document-tab collection recursively clones workspace targets, paths, IDs, titles, icons and empty
states; `build_document_workspace()` finally moves the original recursive page clone into the model.
The same logical document data therefore survives in both recursive workspace form and a flat owned
tab list, with additional active-page strings in the host strip.

Drawers have the same problem. `drawer_ring.drawers = chrome.workbench.drawers.clone()` copies the
entire snapshot. `build_tool_windows()` immediately walks the same drawers and constructs another
owned map of tab IDs, titles, icons and action-bearing empty states. Floating windows repeat the
flat tab projection for each window. These are not independent product authorities; they are
consumer views of layout/session state and should share one immutable generation.

The minimum structural cut is not `Arc<WorkbenchViewModel>`. Introduce compact, generation-owned
domain artifacts:

- layout generation: page/drawer/window topology and stable row IDs;
- session generation: live view title, dirty flag, content kind and payload handle;
- command/menu generation: immutable menu structure and operation-to-row indexes;
- contextual state generation: enablement/check/visibility bits only;
- status generation: status/task/viewport labels independent of layout/menu generations.

Consumers retain typed handles plus generation cursors. Serialized document payload is resolved by
the active content consumer and must not be copied into shell layout, menu, tab-strip or pointer
projections.

### P0: the base menu is rebuilt as seven full registry scans under a command mutex

`core/commands/menu.rs:12-26` enumerates seven top-level labels. For each label,
`menu_model()` at `30-43` iterates every command and `command_menu_item()` creates owned label,
operation and shortcut rows. One model build therefore performs about `7N` command visits before
extension work. Retained full recompute takes `runtime.commands().lock()` at
`recompute/shell/builder.rs:75` and keeps it through `WorkbenchViewModel::build_with_context()`.

Classic reflection is wider: `editor_event_runtime_reflection.rs:29-37` acquires both shell and
command locks, then `40-86` builds descriptors, chrome, model, reflection routes, the full control
snapshot and publishes it before returning. Menu cost therefore increases contention and user input
tail latency, not only allocation count.

The target is a command-registry structural generation containing stable top-level buckets,
immutable row metadata and operation-to-row indexes. A context generation evaluates only affected
enablement/check/visibility dependencies into compact row-state patches. The host snapshots the
generation handles while locked, releases all mutable locks, builds or patches outside them, then
commits only if the source generations still match.

### P0: extension menu assembly is repeated tree construction with quadratic worst cases

`extension_menu.rs:14-23` collects and sorts all visible descriptors on each contributing model
build, `67-72` allocates a path-segment vector for every item, and `92-106` linearly finds the
top-level menu. Recursive insertion at `109-125` linearly scans sibling vectors at every level and
rescans child enablement while unwinding. For M items of depth D concentrated in common branches,
the worst case grows toward `O(M^2 * D)` comparisons rather than a one-time indexed compilation.

The contributed-view fallback at `25-58` is separate repeated work. For every view it reparses an
open operation, recursively scans the complete current menu tree for a duplicate at `29-33` and
linearly finds the View menu. With V views and T menu nodes this is `O(V * T)` before row creation.

Compile extension structure once per mounted extension/capability generation into a stable menu
trie or indexed row table. Store owner/mount generation on every row, keep an operation-to-row map,
and retire owner rows atomically on disable/reload. Contextual command state remains a separate
patchable vector. Menu open may generate a contextual visible widget from that registered structure;
ordinary pointer, selection, status or viewport changes must not sort and rebuild the menu tree.

### P0: traversal and allocation are hidden by functional-only tests

The focused tests call `WorkbenchViewModel::build()` against small fixtures and assert labels,
ordering, bindings and capability filtering. No test contains timing, allocation, visit, generation,
lock or scale instrumentation. The two-item plugin test cannot distinguish indexed compilation from
the current quadratic insertion/dedup algorithm. Existing functional tests remain useful parity
tests, but they are not performance acceptance.

The existing `WorkbenchModelBuildCount` and `recompute_build_workbench_model` scope are useful
starting signals. They do not identify page/drawer/tab clone bytes, menu scans, path allocations,
branch probes, operation-dedup visits or command/shell lock hold and wait. Those counters must be
added before implementation so the baseline and post-cut evidence use the same definitions.

### P1: status and empty-state rows allocate cold metadata on unrelated rebuilds

Status construction formats viewport, selection, grid and fixed diagnostic strings on every full
build. Drawer/document projection also recreates static empty-state labels and bindings per tab.
These costs are smaller than the duplicated domain graph and menu scans. After the P0 cut, move
static row metadata into immutable definitions and update status values through its own generation;
do not build a second cache around the current monolithic model.

## Reference-engine evidence

- Unreal `SlateInvalidationRoot.cpp:299-330` records explicit invalidation reasons on one widget
  proxy and pushes the unique proxy into pre-update, prepass and post-update heaps. It does not use
  an arbitrary input event as a request to rebuild every UI domain.
- Unreal `SlateInvalidationRoot.cpp:1281-1379` processes child/visibility/attribute, attribute,
  prepass and post-update work as separate phases while the fast path remains valid. Only the
  root-level slow path at `1387-1394` resets all update lists and cached element data.
- Unreal `ToolMenus.cpp:2862-2885` generates a menu copy from a registered hierarchy, and
  `2888-2897` binds contextual generation to the explicit widget-generation boundary. This supports
  retained registered structure plus contextual generation, not full menu assembly on every shell
  event.
- Unreal `ToolMenu.cpp:206-210` registers dynamic sections as retained section constructs.
  `ToolMenus.cpp:3391-3438` removes rows/sections by owner and refreshes displayed widgets only when
  removal changed state. This is direct evidence for owner-scoped plugin menu retirement.

These references establish ownership, invalidation and lifecycle shape. They do not prove Zircon
timing or power parity. That requires current-product WPR/ETW and allocator evidence on identical
hardware and workloads.

## Required hard cut

1. Define typed layout, session, command/menu, context and status generations with explicit changed
   IDs. Remove the monolithic snapshot-to-owned-model contract from high-frequency input paths.
2. Replace active-page deep clone with a stable page handle and compact row tables. Shell consumers
   cannot retain arbitrary serialized payload bytes.
3. Publish one shared pane/tab artifact for drawer ring, tool-window, floating/native and reflection
   consumers. A changed row is materialized once per generation; unchanged consumers do zero work.
4. Compile base and extension menu structure once per command/contribution/capability generation.
   Use stable menu/row IDs, owner generation, hierarchy indexes and operation-to-row lookup.
5. Evaluate context-dependent enablement/check/visibility into affected row-state patches. Generate
   contextual visible menu widgets at menu-open or explicit refresh boundaries.
6. Snapshot immutable handles under shell/command locks, release locks, build outside, and validate
   generations on commit. Plugin/user callbacks never execute under those locks.
7. Coalesce dirty domains once per frame. Pointer, typing, selection, status and viewport changes
   invalidate only their declared domains; stable/no-op input produces no model build.
8. Keep all functional tests, then add scale/counter tests before optimizing implementation details.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Baseline counters for model builds, domain visits, owned clone bytes, menu scans/sorts/path allocations/branch probes/dedup visits and shell/command lock hold/wait. | current source re-read |
| M1 | Command and extension menu structural generation, indexed owner lifecycle and contextual row-state patches. | Editor08 + Optimize08/50 |
| M2 | Layout/session/pane generation with stable page/tab/window handles and no shell payload copies. | EditorUI08 + EditorLayout07/08 |
| M3 | Domain invalidation DAG, frame coalescing, lock-free projection and generation-checked commit. | Optimize01 + EditorUI08 |
| M4 | Current-source managed Cargo/F4 plus WPR/ETW CPU, allocation, lock, latency, RSS and package-power matrix. | M0-M3 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| model domains | pages/tabs/drawers/windows `1/100/1k/10k`, payload `0/4KiB/1MiB` | stable generation build/visit/owned clone bytes `=0`; structural change touches affected rows; shell payload clone bytes `=0` |
| base menus | commands `1/100/1k/10k`, seven top-level buckets, context changes `0/1/10%/100%` | stable structure registry scan/row build `=0`; one structural compile per generation; context work near affected rows; no lock spans build/publish |
| extension menus | menu items/views `1/100/1k/10k`, depth `1/4/16`, owner enable/disable/reload | no per-build sort/path Vec/tree dedup; one indexed owner-generation update; stale rows/callbacks `=0`; deterministic order and capability parity |
| invalidation | pointer/typing/selection/status/viewport/layout storms `1/1k/1M` | unchanged domain build `=0`; each dirty domain builds at most once per frame; changed row/presenter patches at most once per generation |
| product | F4 idle and menu open/close, selection, typing, viewport, dock/float and plugin reload, 31 runs | WPR/ETW CPU, allocation, lock hold/wait, input-to-pixel p50/p95/p99, RSS and package power reported on identical hardware/config; artifacts only on D/E/F |

RenderDoc is conditional on rendering-visible changes. It can validate draw/pixel parity after the
domain cut, but it cannot prove CPU menu complexity, owned clone bytes, lock contention or package
power by itself.

## Static gates executed

- Read 44/44 production files and all 7/7 focused test files; reproduced line, byte, test counts and
  both current-source fingerprints above.
- Traced both product build paths and proved the classic path holds shell plus command locks across
  full model/reflection publication, while retained full recompute holds the command lock across the
  complete model build.
- Reproduced the seven-registry-scan base menu, extension sort/path/tree insertion, per-view recursive
  dedup, active-page clone and duplicate drawer/document projections from current source.
- Read the cited Unreal invalidation and ToolMenus generation/owner-lifecycle implementations.
- Source and focused test paths were clean at review time. Optimize01/08/50 were foreign dirty and
  were not edited.
- No managed Cargo lane, F4 launch, WPR/ETW, package-power or RenderDoc capture was run. Active shared
  Cargo/validation lanes prevented a noncompeting current-source product run; this CPU ownership cut
  is not a RenderDoc-first problem.

## Completion rule

This module remains pending until M0-M4 pass against a current source fingerprint. Small fixture
parity, a monolithic `Arc`, or a lower allocation count without generation/lock/invalidation proof
is not acceptance. No milestone commit or WeCom completion message is permitted before quantified
product evidence.
