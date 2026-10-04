---
title: Editor workbench projection generation-owned shell cache performance review
date: 2026-08-22
module: zircon_editor/src/ui/layouts/windows/workbench_host_window/{projection_cache.rs,shell_presentation.rs}
priority: MVP-P0 editor shell recompute and retained chrome
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate persistent widgets, tab wells and invalidation roots
---

# Goal

Make the workbench owner publish immutable model and presentation segments with exact generations.
An unchanged shell recompute must perform O(1) generation comparisons and clone shared owners only;
it must not rebuild a complete `WorkbenchViewModel`, recursively compare copied menu/tab/stack trees,
query the template store mutex, or reconstruct unrelated pane/floating/status segments.

## Reviewed source

- owner files: `projection_cache.rs` and `shell_presentation.rs`
- Rust files: 2/2
- current lines: 643
- current bytes: 23,336
- joined current source-bytes SHA256:
  `72e455542293d88f85f706dbbeb5849937e2185f2cc826fe2bf8f70f0cbbf3f7`
- joined pre-change source-bytes SHA256:
  `0c1533e10e7b9ef1369958158d7ec5f3dff7eefbace2b8d484753143b78b6451`
- owning commit before review: `bee4c707b714738346b49bba15c59468b8bd9b39`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `projection_cache.rs` | 442 | `aa92965ddc276a5b983204689df0486da2a78c185f46384e08a6b51d2f5df3c0` |
| `shell_presentation.rs` | 201 | `1ec669606ffff135de94b12dedb9b96bcffd08854c7134bf69a207d41a260593` |

Both files were read in full. Production ownership was followed through retained-host recompute,
viewport fallback and `apply_presentation`. Related `WorkbenchViewModel` construction, menu model,
template resource generation and store-cache code were read to establish cost and generation
availability; those related modules are not counted as fully reviewed here.

## Existing foundations to retain

The host owns one persistent `HostChromeProjectionCache` across retained applies. Stable derived tab,
preset and menu models return shared `ModelRc` owners, resource generation uses immutable compiled
document/design-token identities, and tests assert stable chrome model identity across independent
shell rebuilds. The template store already uses file watchers and a compiled source cache. These are
valid pieces, but they are joined above and below the wrong ownership boundary.

## Structural findings

### P0: upstream reconstructs the full model before the cache can answer

Every full retained recompute builds a new chrome snapshot and calls
`WorkbenchViewModel::build_with_context`. That constructor recreates the complete recursive menu bar,
host strip, tool-window map, document tabs, floating windows, document workspace and status model.
Only after this work does `ShellPresentation` ask `HostChromeProjectionCache` whether copied tab/menu
values equal the previous copy.

The cache therefore cannot make an unchanged recompute cheap. It only avoids some downstream DTO
projection after the source model has already been allocated. Drag/drop and reflection routes also
build complete workbench models for narrower questions. The owner needs source-domain generations
before model construction, not an equality cache after it.

### P0: stable cache hits are linear or recursive deep comparisons

Host tabs compare the complete `Vec<HostPageTabModel>` plus active page. Document tabs compare every
wide `DocumentTabModel`, including paths, strings and empty state. Side tabs walk stacks and compare
nested tab vectors. Preset names compare every string. Menu chrome compares a recursive
`MenuBarModel` tree. A miss then clones the same full source into the cache in addition to building the
derived model.

For P pages, D document tabs, S stack/tab records, R presets and M recursive menu items, a stable hit
is O(P+D+S+R+M), not O(1). Hashing these rebuilt values would retain the same traversal and add another
authority; exact source generations or shared immutable owners are required.

### P0: menu cache probes the global template store on every lookup

`menu_chrome` calls `view_template_resource_generation` before checking model/input equality. That
function calls `load_view_v2_store`, which locks the process-global store mutex. The stable path creates
a request with owned path strings, clones its source path vector, traverses watcher roots and asks the
compiled file cache for the store before returning Arc identities. Watcher invalidation is event
driven, but the generation read itself is not a cheap atomic/owner snapshot.

The template resource owner must publish a lock-light immutable generation token. A consumer cache
must not enter the load path to ask whether a generation changed.

### P0: shell presentation remains a monolithic reconstruction

Even when tab models hit, `ShellPresentation::from_state_with_template_v2_data_and_cache` rebuilds
floating-window rows, all four pane selections/payloads, welcome/project/asset presentations,
`HostWindowShellData` and status strings. `HostChromeProjectionCache` covers only tab/preset/menu
fragments, while pane, floating, status and shell geometry products have separate or no receipts.
Stable chrome reuse therefore coexists with unrelated presentation reconstruction.

### P1: two public-within-crate constructors silently discard retention

`ShellPresentation::from_state` creates an empty template-data map and delegates to
`from_state_with_template_v2_data`. That second entry creates a fresh `HostChromeProjectionCache` and
delegates to the real cached constructor. Repository call analysis finds no caller of either wrapper;
production uses the cached entry directly. Keeping these compatibility-style entries permits a future
caller to discard all retained state without an explicit choice.

This is a simple hard-cut M1: remove both unused wrappers and rename the one authoritative cached
constructor to `from_state`, updating its production caller and owner test.

## M1 result

The two wrapper constructors were deleted. `ShellPresentation` now exposes one crate-local
`from_state` entry that requires both template-v2 data and the persistent
`HostChromeProjectionCache`. Production apply and the stable-identity test use that same entry; a
repository search finds no old constructor name.

The owner pair changes from 720 to 643 lines and removes the only
`HostChromeProjectionCache::default()` inside `shell_presentation.rs`. Constructor routes change from
three to one, so no crate-internal caller can silently opt out of retention through this owner. The
production caller `apply_presentation.rs` post-change SHA256 is
`01688c414bb8563d3b4c554e7bbd21dc7fa4ad9fbb9389e8bd5084536c64a5f7`.

M1 does not reduce the full-model construction, deep cache comparisons or template-store mutex work;
those remain explicit M2-M4 acceptance items.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

Unreal `SDockingTabWell` owns persistent live tab children (`40-57`), mutates the collection only on
tab lifecycle events (`60-96`) and arranges it once linearly (`100-180`). `TabManager` reuses live tabs
before spawning (`1711-1727`, `1766-1823`). An `SWindow` retains its invalidation root and hit-test grid
(`2077-2080`) and delegates paint to the root (`2120-2149`). The invalidation root uses cached element
data, selects the slow path only for explicit root/policy invalidation and does no update for an empty
fast-path list (`SlateInvalidationRoot.cpp:356-424`).

The transferable invariant is that persistent model/widget owners carry exact dirty state into a
retained window. Consumers do not rebuild a value tree and recursively compare it against yesterday's
copy to discover that no event happened.

## Target architecture

1. Publish a `WorkbenchProjectionReceipt` from the mutation owner with independent menu, host tabs,
   document tabs, drawer stacks, floating windows, pane content, status and command-state generations.
2. Store each source segment as an immutable shared owner. `WorkbenchViewModel` assembly clones owners;
   a segment is rebuilt only from its matching dirty generation.
3. Replace value snapshots in `HostChromeProjectionCache` with exact receipt plus output owner. A hit is
   O(1); a miss reads the already published source owner without cloning a second source copy.
4. Publish `ViewTemplateResourceGeneration` from the template store as a lock-light snapshot updated
   when watcher/load/token/font generations change. Generation reads must not call `load_store`.
5. Split `ShellPresentation` into generation-owned shell, chrome, dock/pane, floating and status
   artifacts. Full assembly clones shared handles; narrow invalidation replaces only one artifact.
6. Hard-cut all drag/drop, reflection, viewport and apply consumers to the same committed model receipt.
   Narrow queries must not construct a complete ad hoc `WorkbenchViewModel`.
7. Delete deep-value cache keys, uncached constructors and flat full-recompute compatibility paths.

Complexity targets:

- unchanged shell refresh: O(1), zero model/presentation/template-store builds;
- changed one tab group: O(changed tabs), unrelated menu/panes/floating/status zero;
- changed command state: O(changed command/menu branches), no tab/pane rebuild;
- template generation read: O(1), no global store mutex or path allocation;
- shell assembly: shared-owner clones only.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| workbench model/segment builds by generation | unchanged segment = 0 |
| source equality items/bytes visited | stable = 0 |
| source and output clones/allocated bytes | no duplicate source snapshot |
| template store mutex/load/request/path work | stable generation read = 0 |
| shell/pane/floating/status builds | changed domain only |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: pages/document tabs/stack tabs 0/1/16/1,000; menu items 7/100/10,000 with depth 1/8/64;
presets 0/16/1,000; panes/floating windows 0/4/1,000; stable refreshes 1/1,000; selection, close,
drag/drop, status, command enablement, template reload, font reload, resize and render-only changes.
Capture model/segment builds, equality visits/bytes, mutex wait/hold time, store loads, allocations,
main-thread CPU, latency, RSS and package energy on one source/executable fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for a
launchable current-source editor's GPU/pixel parity; it cannot validate model construction, recursive
equality or store-mutex contention.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add model/segment/equality/clone/store-mutex/load counters and capture baseline. | source-bound scale/profile evidence |
| M1 | Remove fresh-cache wrappers and hard-cut to one authoritative cached constructor. | RED-to-GREEN source contract and call-site audit |
| M2 | Publish immutable source segments and exact workbench projection receipts. | stable hit O(1), unchanged build = 0 |
| M3 | Publish lock-light template resource generation snapshots. | stable lookup has zero store mutex/load/path work |
| M4 | Split shell artifacts and convert narrow routes to committed receipts. | changed domain only; no ad hoc full model |
| M5 | Delete deep-value snapshots and full-recompute compatibility paths. | one model/presentation authority |
| M6 | Run scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 2/2 Rust files.
- Production recompute/apply callers, related model/resource ownership and Unreal references: read.
- M1 source implementation: complete. Its static contract moved RED 2/2 to GREEN 2/2.
- Changed Rust `rustfmt --check`, old-name repository search and scoped diff check: passed.
- M0 and M2-M6 implementation and dynamic acceptance: pending.
- Managed Cargo remains unavailable because the current validation Session is terminal `archived`.

The module stays in `pending.md` until M0-M6 pass on one source/executable fingerprint. An M1 API
cleanup alone is not end-to-end performance acceptance.
