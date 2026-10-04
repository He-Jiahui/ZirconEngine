---
title: Editor retained drawer header native receipt hard cutover performance review
date: 2026-08-23
module: zircon_editor retained-host drawer_header_pointer
priority: MVP-P0 tool-window drawer activation and collapse
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine STabSidebar and STabDrawer
---

# Goal

Consume the drawer-tab action already resolved by native chrome. A confirmed press must not update a
second measured-frame store, dispatch through a second hit tree, or clone String route payloads
before it can activate or collapse the selected tool window.

## Reviewed source

- pre-M0 owner Rust files: 21/21
- pre-M0 physical lines: 696
- pre-M0 bytes: 25,241
- pre-M0 LF path-tab-file-SHA manifest SHA256:
  `ec2a15059052d30008f31d2e2424ea6da6a0f382bfbcbf2a7c12296f424573d7`
- post-M0 owner Rust files: 12/12
- post-M0 physical lines: 224
- post-M0 bytes: 7,678
- post-M0 LF path-tab-file-SHA manifest SHA256:
  `ffb51ecd5fccded3fe4351649ff3d5dc14879891cc98225c6d5c8b1e29469565`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

All owner files were read in full. The review also traced all retained drawer-header tests, host
layout recompute, callback wiring, native route construction and button dispatch, typed workbench
drawer slots/instance ids, the shared drawer command path, and both activity-rail callers. The July
17 report improved the mirror tree's frame patching, but current source shows that the native chrome
tree had already resolved the exact tab; this report supersedes its mirror-preservation direction.

## Structural findings

### P0: one committed native hit is repeated against an editor mirror

Native `route_drawer_header` iterates the committed `HostChromeTabData.frame` values and emits
`ChromePointerRoute::DrawerHeaderTab { surface_key, index, ... }` only after `contains` succeeds.
The callback then feeds the same tab geometry and local point into `update_measured_frame`, translates
the point, and dispatches another Down event through `UiSurface`, `UiPointerDispatcher` and
`EditorRouteIntentMap`. This duplicates hit, layout and allocation work and can reject a valid native
receipt when the mirror projection is stale.

M0 validates only surface/index against a model-owned receipt projection and directly returns a
compact route. The mirror surface, dispatcher, measured-frame store and geometry patching are
deleted.

### P0: stable recompute constructs geometry and owned strings before equality can reject it

Every host recompute walks three regions, reads componentized layout frames, derives strip geometry,
allocates surface/item vectors, converts each slot to String and clones each instance id. A changed
layout then rebuilds formatted node paths and clones surface/slot/instance Strings into routes.
Stable equality happens only after that projection work.

M0 removes paint-frame and metrics inputs, uses typed `ActivityDrawerSlot`, typed `ViewInstanceId`
and static surface keys, and instruments projection/sync reuse. M1 publishes the identity projection
under the same native generation so stable recompute performs no duplicate projection.

### P0: every click allocates/copies a wide route and reparses a known slot

The mirror route owns three Strings (`surface_key`, `slot`, `instance_id`) and the route-intent
accessor clones all three for the selected click. The shared command path then parses the already
known slot string and creates another owned instance id.

M0 makes the route Copy with only `surface_index + item_index`, borrows the typed target, and changes
the shared drawer command boundary to accept `ActivityDrawerSlot` and `ViewInstanceId`. One instance
clone remains only where the owned layout command requires it.

### P1: changed measurement allocates a suffix patch list and rebuilds dirty layout post-hit

`update_measured_frame` allocates `Vec<MeasuredFramePatch>`, walks later unmeasured tabs, patches
constraints and calls `rebuild_dirty`. The native callback is emitted only after the original native
frame matched, so this is post-hit reconstruction rather than necessary input work. M0 deletes the
path; it is not retained as an incremental-layout optimization.

### P1: drawer command dispatch linearly scans bindings for an identity mapping

The old shared command path calls `activity_binding_for_target`, which scans every template binding
and accepts only a payload whose slot and instance strings equal the supplied target. It then parses
those same strings back into the already known types. The lookup cannot transform the target and is
therefore pure per-click overhead.

M0 removes the scan and parser from both drawer-header and activity-rail callers. It also borrows the
selected active drawer from `activity_windows()` instead of cloning the active drawer map and
selected drawer. `runtime.current_layout()` still clones the complete layout snapshot; M2 replaces
that read-snapshot/write-command sequence with one authority transaction.

## Zircon and Unreal source basis

Direct Zircon source read:

- `host_contract/native_pointer/routing/chrome/tabs/drawer.rs` hits the committed tab frame and
  returns the exact surface/index action.
- `host_contract/native_pointer/button_dispatch/chrome_press/tabs/drawer.rs` forwards that receipt
  before `drawer_header_pointer` is entered.
- `callback_dispatch/layout/drawer_toggle.rs` already owns activation/collapse semantics and typed
  layout commands; hit testing does not belong there.

Direct Unreal source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabSidebar.cpp`
  - `STabDrawerButton::Construct` binds `OnPressed` to the retained `TSharedRef<SDockTab>`;
  - `OnTabDrawerButtonPressed` passes that exact tab to `TryToggleSidebarDrawer`;
  - `OpenDrawerInternal` constructs `STabDrawer` from that same tab and retained button.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabDrawer.cpp`
  retains `ForTab`, and open/close/active-tab behavior operates on it directly.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp`
  moves/restores the same retained `SDockTab` between stack and sidebar state.

The transferable rule is that the hit widget supplies a stable tab object to the docking owner;
another geometry tree must not rediscover that tab.

## Target architecture

1. Native chrome remains the sole drawer-tab paint and hit authority.
2. Its action receipt carries stable drawer slot, view identity and publication generation.
3. Editor dispatch validates generation once and borrows the typed target.
4. Activation/collapse executes one typed layout transaction with exact dirty/damage domains.

## Instrumentation and acceptance

Matrix: regions `left/right/bottom`; tabs per region `1/16/100/1K`; action
`activate/collapse/switch`; topology `stable/add/remove/reorder`; rate `10/125/500 Hz`;
receipt generation `current/stale`.

Acceptance requires:

- editor mirror hit dispatches per confirmed native press: `1 -> 0` at M0;
- mirror surface rebuild and measured-frame dirty rebuild paths: removed at M0;
- route-owned Strings and selected-route String clones: `3 -> 0` per press;
- per-tab rebuild route String clones: `3 -> 0`;
- stable recompute duplicate identity projection allocations: `>0 -> 0` at M1;
- stale receipts are rejected deterministically under the native publication generation;
- p95 receipt-to-command dispatch below 0.02 ms at 1K tabs on the recorded host;
- WPR shows no mirror layout/hit/allocation wakeups and behavior remains equivalent.

RenderDoc is relevant only to final product pixel parity, not this CPU/input owner. WPR, allocator,
build and capture artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Hard-cut to native receipt; compact route, typed target and command boundary; delete mirror geometry/surface. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | Publish generation-owned drawer slot/view identity in the native receipt and delete duplicate bridge projection. | zero stable projection allocation, stale-generation tests |
| M2 | Make drawer toggle one typed layout transaction with exact dirty/damage receipt. | one authority transaction and behavior parity |
| M3 | Run scale/storm/WPR/power plus interaction/pixel parity. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 21/21 pre-M0 Rust files.
- Native route/callback, model/command chain and all retained drawer-header tests: read and mapped.
- Unreal sidebar/button/drawer/tab-stack source: read and mapped.
- M0 implementation: applied. Nine mirror geometry/hit files were deleted; the surviving owner is a
  typed receipt projection plus surface/index validation. Paint frames, metrics, `UiSurface`,
  `UiPointerDispatcher`, measured-frame patching and drawer-header route-intent binding are absent.
- Exact static owner delta: files `21 -> 12`, physical lines `696 -> 224` (-472, 67.8%), bytes
  `25,241 -> 7,678` (-17,563, 69.6%). These are source-size facts, not timing claims.
- Shared click-path source delta: template binding scans `1 -> 0`, known-slot parses `1 -> 0`,
  active drawer map clones `1 -> 0`; complete layout snapshot clones remain `1` and are assigned to
  M2.
- Focused static contract:
  `tools/tests/test_editor_retained_drawer_header_native_receipt_performance_contract.py`, 123
  lines, 4,786 bytes, SHA256
  `8f00b2feef44a132ff9b8bce341db291d289ea7478a3905b6934b18cdcc72d4a`; RED 1/7 to GREEN 7/7.
- Retained-host performance contracts: GREEN 30/30. Broad `test_*performance_contract.py`
  discovery on the current shared worktree: GREEN 199/199. Rustfmt on focused Rust files and scoped
  `git diff --check` passed.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is no longer addressable; raw Cargo is not
  an allowed bypass.
- M1-M3 and dynamic evidence remain pending; this owner stays in `pending.md`.
