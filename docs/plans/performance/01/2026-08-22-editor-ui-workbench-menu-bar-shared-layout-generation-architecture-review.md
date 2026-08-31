---
title: Editor Workbench menu bar shared layout generation performance review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/menu_bar
priority: MVP-P1
status: source_reviewed_static_pass_dynamic_pending
parent_plan: 2026-08-19-editor-ui-workbench-model-domain-generation-menu-compilation-architecture-review.md
reference_engine: Unreal Engine ToolMenus and Slate
---

# Goal

Keep menu-slot sizing as a pure constant-time policy, while making chrome projection and retained
pointer routing consume one immutable menu/layout generation. An unrelated Host recompute must not
rebuild and then compare an equivalent menu tree.

## Reviewed source

- folder: `zircon_editor/src/ui/workbench/menu_bar`
- Rust files: 2/2
- lines: 52
- bytes: 1,551
- joined UTF-8 SHA256: `1d2ea325dbe1090fdf9fed359d1a76eda203a263101e966e85b8a9ed1e8bde9b`
- `metrics.rs`: `70cbb0075fcf927a1d8f23e5b9a385a2ad7bc439c982982d8f0f9284649f70a5`
- `mod.rs`: `8124636469f8e06aa0f33cd23dc55cc8e8cde189cdfad130b54c34b5c2e57e0d`
- owning commit before review: `08094b9b9e17f6c80372e15c17b01204038b305b`

Both files were read in full. The related production call chain was read through menu chrome
projection, retained menu pointer layout/sync and Host recompute.

## Result

### Source module

`workbench_menu_slot_width_from_label_width` is `O(1)`, allocation-free, rejects negative/non-finite
input and clamps to the shared minimum/maximum. Two local tests cover typography ownership and
finite/min/max behavior. `rustfmt --edition 2021 --check` passes for 2/2 files. No direct code change
is justified in this folder.

### P0 parent dependency: menu structure is rebuilt broadly

The existing parent plan already proves that Workbench model construction repeatedly scans command
registries and recompiles extension menus. This folder must not add a local cache that hides that P0.
It depends on the parent's immutable command/menu structural generation and indexed owner lifecycle.

### P1: retained pointer layout performs work before its equality guard

Every slow Host recompute reaches `sync_menu_pointer_layout`. Before `HostMenuPointerBridge::sync`
can compare old/new layouts, `build_host_menu_pointer_layout` has already:

- collected top-level label references and projected the menu chrome asset;
- measured each menu label and allocated button-frame vectors;
- recursively allocated a second `Vec<Vec<MenuItemSpec>>` tree and cloned leaf action IDs;
- separately cloned root labels/shortcuts/preset strings into popup measurement rows and measured
  them;
- cloned preset names and active/resolved preset strings;
- cloned the completed layout once more when passing it to the bridge.

For `N` menu nodes, `M` top-level menus and `P` presets, the stable unrelated-recompute path is at
least `O(N + M + P)` allocations/visits before equality can reject the result. Menu chrome projection
also builds text override maps and measures top-level labels for its own geometry. The same structure
and geometry therefore have multiple materialization owners.

This is product-reachable on slow full recompute, but not an idle-frame claim: `recompute_if_dirty`
returns early when no invalidation is pending. Dynamic counters must determine its share of real
interaction cost before implementation.

## Unreal source basis

Direct source read under `dev/UnrealEngine`:

- `Engine/Source/Developer/ToolMenus/Private/ToolMenus.cpp`
- `Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`
- `Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`

Relevant behavior:

- `UToolMenus::GenerateWidget` generates the menu and widget at an explicit request boundary
  (`2888-2913`), rather than rebuilding every menu interaction tree during unrelated editor
  presentation work.
- `RefreshAllWidgets` only sets a next-tick refresh flag (`3072-3078`). `HandleNextTick` consumes the
  flag once and refreshes live instances (`3081-3109`), coalescing multiple requests.
- Owner removal sets `bNeedsRefresh` only when entries/sections actually changed, then requests one
  refresh (`3391-3437`).
- Slate stores desired sizes through `SWidget::CacheDesiredSize` and returns the cached value from
  `GetDesiredSize`, as detailed in the document-tabs review.

The applicable rule is explicit generation/refresh boundaries plus cached layout state. Zircon must
not copy Unreal's UObject or widget implementation.

## Target architecture

Extend the parent command/menu structural generation with a derived immutable menu presentation
generation containing:

- stable top-level menu IDs/labels and compiled action tree;
- enabled/checked/context state revision separated from immutable structure;
- measured top-level button frames and root popup widths keyed by shell/font/style generation;
- preset catalog generation/identity, not cloned strings per recompute;
- shared route/action metadata used by both projection and pointer dispatch.

The Host keeps a generation receipt. Unrelated presentation recompute reuses it. Shell metrics or font
changes recompute only geometry; command/plugin/preset structural changes recompile only the affected
generation; selection/project state patches only affected enabled/checked rows. The pure helper in
`menu_bar/metrics.rs` remains the sizing policy and must not own global state.

## Instrumentation and acceptance

Add before/after counters for:

| Counter | Unchanged unrelated recompute target |
| --- | --- |
| menu pointer layout builds | 0 |
| menu chrome asset projections | 0 |
| top-level label and popup text measures | 0 |
| `MenuItemSpec` nodes allocated/action IDs cloned | 0 |
| preset strings cloned | 0 |
| layouts built then rejected by equality | 0 |

Use the parent plan's command/menu scale matrix (`1/100/1k/10k`, depth `1/4/16`) and add shell widths
640/900/1260, presets `0/1/100/10k`, unrelated/full/menu-open/plugin-reload invalidations. Report
median/p95 main-thread cost, visits, allocations/bytes, lock wait/hold, input-to-pixel latency, RSS
and package energy from the same current-source build and hardware.

Windows WPR/ETW artifacts and build outputs must be on `D:`, `E:` or `F:`. RenderDoc is not a menu
CPU/allocation profiler; use it only for current-source draw/pixel parity after a launchable renderer
is available.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add counters and capture stable/full/menu-open baseline. | Counter plus WPR artifact |
| M1 | Land parent immutable command/menu structural generation. | Parent plan M1-M3 |
| M2 | Derive shared menu layout/route generation and split structural, geometry and context invalidation. | Focused generation tests |
| M3 | Make chrome and pointer consumers reuse it; remove compare-after-rebuild path. | Source ownership scan and focused behavior tests |
| M4 | Run scale/product matrix and real-window parity. | Quantified WPR/allocation/power and screenshots |

## Validation state

- Full folder review: passed, 2/2 files.
- Static formatting: passed, 2/2 files.
- Existing local tests identified: 2.
- Managed Cargo: pending while shared Runtime Cargo lanes are active.
- Dynamic CPU/allocation/power and current-source real-window evidence: pending.

Keep the folder in `pending.md`; do not add it to `review.md` until M0-M4 and the parent plan gates pass.
