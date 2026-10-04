---
title: Editor native pointer menu geometry generation-stack performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer menu_geometry
priority: MVP-P0 editor menu input correctness, stable-event allocation and geometry reuse
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate FMenuStack
---

# Goal

Make one opened-menu stack the generation-owned authority for popup placement, hit testing and
damage. Stable pointer facts must query retained frames without cloning menu DTOs or rebuilding the
open submenu path, while menu-state changes update only the changed suffix.

## Reviewed source

- pre-M0 owner Rust files: 27/27
- pre-M0 physical lines: 732
- pre-M0 bytes: 23,598
- pre-M0 path-and-file-SHA manifest SHA256:
  `4a990b8d33360c2f7d8d0b2d1da680cf00b0185a896f29be1cfb88d4edf68578`
- post-M0 owner Rust files: 27/27
- post-M0 physical lines: 698
- post-M0 bytes: 22,591
- post-M0 path-and-file-SHA manifest SHA256:
  `2886418fd584dbcb1c69bde2a390674468cd982ddd3a9d51a71c78c50d55c649`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

Owner scope is `native_pointer/menu_geometry.rs + menu_geometry/**`. All files were read in full.
Button, move and scroll callers, split `HostMenuStateData`, `ModelRc` storage, menu popup viewport
rules and the earlier 2026-07-17 report were traced. The old report is stale: the owner grew from
627 to 732 physical lines and it did not include the required Unreal-primary source basis.

## Correct foundations to retain

1. Popup traversal follows only the open submenu path, so its current asymptotic cost is O(depth),
   not O(total menu-tree rows).
2. Root and nested frames preserve viewport scrolling, work-area clamping, alternate left/right
   submenu placement and popup blocking against fall-through.
3. Button, move and scroll event callers capture a generation and pass its explicit menu
   interaction snapshot to the `_with_state` APIs.
4. `ModelRc::get/iter` can borrow rows from contiguous, shared-row and overlay storage.

## Structural findings

### P0: stable events rebuild the same popup stack

Move and scroll first reconstruct the root and every open nested popup to test containment, then
reconstruct the path again for damage. Button press computes damage before and after state changes,
so one fact can rebuild two complete stacks. Even when state and layout are unchanged, popup
position, row geometry, blocking and bottom bounds are recomputed from DTOs.

M1 publishes `HostMenuPopupGeometryStack` with the menu generation. It owns root/nested popup
frames, visible row ranges, blocking bounds and the union damage frame. A stable event performs one
topmost stack query; an open/hover/scroll delta recomputes only the changed suffix.

### P0: seven hot-path accesses deep-clone wide rows

Top-bar hit testing, root damage, root popup source and both nested traversals call
`ModelRc::row_data`, which is implemented as `get(row).cloned()`. Menu rows carry labels, command
ids and child models; rejected top-bar candidates and each selected branch therefore clone DTO
payloads before simple geometry checks. M0 replaces all seven accesses with borrowed `iter/get`
and clones only the child `ModelRc` handle where the current return type still owns it.

### P0: three zero-call compatibility entries read the wrong state owner

`menu_handles_point`, `menu_popup_handles_point` and `menu_damage_frame` read
`presentation.menu_state`. The current structure/interaction publication splits menu interaction
out of the structural presentation, while every real event caller already uses the explicit
`_with_state` entry. The wrappers have no call sites and can silently route against reset state if
reintroduced. M0 hard-cuts these entries and their re-exports.

### P1: hit and damage have independent geometry authorities

`popup/**` and `damage/**` separately encode root construction and nested placement. Their current
formulas are similar but not the same artifact, so future placement changes can make hit, paint and
damage disagree. M1 makes all three consume the same retained stack produced by layout/projection.

### P1: top-bar hit still scans all controls

Borrowing removes allocation but top-bar hit remains O(menu count). M2 publishes ordered bar
intervals and uses binary search or a compact interval index. Acceptance is based on visited
candidates, not merely lower allocation.

### P1: menu state is path-oriented but unversioned

Open and hovered paths are independent vectors without a stack generation or longest-common-prefix
update record. M2 adds a monotonically increasing menu interaction generation and an explicit
changed-prefix/suffix plan. It must not create a second open-state authority beside the menu bridge.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/MenuStack.h`
  - `FMenuStack` is explicitly the stack of open menus; the last item is topmost.
  - it retains `Stack: TArray<TSharedPtr<FMenuBase>>` and `CachedContentMap` rather than rebuilding
    menu identity from source DTOs on each pointer fact.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/MenuStack.cpp`
  - `PrePush` performs prepass, desired-size and fitted popup placement once when a menu is pushed;
    `PushPopup` installs that menu into the popup panel.
  - `PostPush` inserts the retained menu, updates `CachedContentMap`, and dismisses only descendants
    after the insertion point.
  - `DismissInternal` removes children in reverse order; `FindMenuInWidgetPath` uses the retained
    content map; `FindMenuWindowUnderCursor` queries the retained stack topmost-first.
  - the menu content wrapper declares invalidation support, so unchanged presentation is retained.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Input/SMenuAnchor.cpp`
  - the anchor pushes hosted/window menu content into the application menu stack and later queries
    that retained menu object instead of reconstructing a separate geometry tree per event.

The applicable standard is not a literal C++ port. Zircon should preserve its model and viewport
rules while adopting one retained, invalidation-driven open-menu stack and topmost query authority.

## Target architecture

1. Publish `{ structure_generation, menu_interaction_generation, popup_stack }` atomically.
2. Store root and nested popup frames, row metrics, visible ranges, blocking frame and damage union
   once per affected menu generation.
3. Derive the next stack by longest common prefix and rebuild only the changed path suffix.
4. Query one topmost menu hit result and reuse it for move/scroll/button dispatch.
5. Share the same frames with paint and damage; do not duplicate placement formulas.
6. Index top-bar x intervals and retain typed/stable command identity.
7. Count stack builds, rebuilt levels, queried levels, borrowed rows, DTO clones, allocations,
   damage calculations and generation ids.

## Instrumentation and acceptance

Matrix: menu rows `0/1/100/1,000/10,000`; submenu depth `0/1/10/100`; event
`move/scroll/press/release`; state `stable/root-change/suffix-change/close`; viewport
`unscrolled/mid/max`; scale `1x/1.5x/2x/4K`.

Acceptance requires:

- after M0, owner `row_data` calls and legacy reset-state entry points = 0;
- stable move/scroll stack builds, DTO clones and menu String copied bytes = 0;
- one path change rebuilds exactly the changed suffix;
- hit, paint and damage consume the same generation and frames;
- top-bar visited candidates are logarithmic/bounded rather than proportional to total menus;
- p95 stable menu hit below 0.05 ms at 10K rows and p95 suffix update below 0.10 ms for depth 10
  on the recorded host;
- scroll/clamp, separators, disabled items, placement, blocking, z order, damage and pixels remain
  equivalent.

WPR owns CPU/allocation/power evidence. RenderDoc is used only after a current-source executable
exists, and only to prove draw/scissor/pixel parity. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Borrow all menu rows and hard-cut reset-state compatibility entries. | applied; focused RED 1/3 to GREEN 3/3 |
| M1 | Publish one generation-owned popup geometry stack shared by hit/paint/damage. | zero stable rebuilds and one geometry authority |
| M2 | Add suffix updates, top-bar interval index and menu generation counters. | bounded candidate visits and exact rebuilt levels |
| M3 | Run scale/WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 27/27 current Rust files.
- Event callers, split state and `ModelRc` clone semantics: read and mapped.
- Unreal menu-stack implementation: read and mapped.
- M0 implementation: applied. All seven `row_data` accesses now borrow through `iter/get`; the
  three zero-call reset-state wrappers and re-exports are removed. Owner size decreased by 34
  physical lines and 1,007 bytes.
- Focused static contract:
  `tools/tests/test_editor_native_pointer_menu_geometry_generation_performance_contract.py`, 56
  lines, 2,016 bytes, SHA256
  `94539ea963e2239c4d7eb3f5c53a59e77d493ffa8aa7db093086021238651022`; RED 1/3 to GREEN 3/3.
- Adjacent keyboard/button/move/scroll/routing/damage/drag/popup/page/context-menu contracts:
  GREEN 39/39. Rustfmt and scoped `git diff --check` passed; owner searches for `row_data(` and
  legacy entry definitions returned zero.
- Broad performance static discovery: 175/181 passed. The six failures are pre-existing external
  drift: two removed test-fixture paths, missing `available_slots`, two UI-surface root clones and
  Runtime 07 source/telemetry/owner-gate documentation. No menu-geometry contract regressed.
- Managed Rust tests, M1-M3, current-source launch, WPR and RenderDoc remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until all milestones pass on one source/executable fingerprint.
