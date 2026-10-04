---
title: Editor retained document tab native receipt hard cutover performance review
date: 2026-08-23
module: zircon_editor retained-host document_tab_pointer
priority: MVP-P0 document activation close and floating-document navigation
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine SDockingTabStack and FTabManager
---

# Goal

Consume the document-tab action already resolved by the native chrome hit authority. A confirmed
body or close press must not be translated into a second pointer event, dispatched through a second
tree, or depend on a separately measured mirror geometry before it can activate or close the tab.

## Reviewed source

- pre-M0 owner Rust files: 19/19
- pre-M0 physical lines: 740
- pre-M0 bytes: 29,998
- pre-M0 LF path-tab-file-SHA manifest SHA256:
  `867ce802e385b37612229943e9313ea7b4381ba9151bca535ca68b2bb6c0e43b`
- post-M0 owner Rust files: 12/12
- post-M0 physical lines: 240
- post-M0 bytes: 8,501
- post-M0 LF path-tab-file-SHA manifest SHA256:
  `e74b7dfcecfc308f0d12dd1082be2168f770cacab9e9190c2d90f83f4141c64e`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`

All owner files were read in full. The review also traced native chrome route construction for tab
body and close controls, native button dispatch, both host callbacks, pointer-layout recompute,
floating-window projection, typed workbench tab identities, layout commands and all retained
document-tab tests. The July 17 report correctly fixed repeated measured-frame rebuilds, but it
assumed the editor mirror surface was authoritative. Current native source disproves that premise.

## Correct foundations to retain

1. Native chrome routing checks the committed painted tab and close frames before emitting a route.
2. The native route already distinguishes body from close and carries surface key plus item index.
3. Runtime activation and close use typed `FocusView`/`CloseView` command paths.
4. Workbench tabs already own `ViewInstanceId`; the pointer projection need not degrade it to a
   String and then reconstruct it.

## Structural findings

### P0: one native hit is repeated against an editor mirror tree

`route_document_tab_body` and `route_document_tab_close` have already tested the committed native
frames and emitted `ChromePointerRoute::DocumentTab { surface_key, index, close }`. The callback then
updates a second measured-frame store, translates the point, and dispatches another Down event
through `UiSurface`, `UiPointerDispatcher` and `EditorRouteIntentMap`. This is redundant CPU work and
a correctness split: an already confirmed native click can be rejected when mirror geometry lags.

M0 treats the native action as a receipt, validates only surface/index/closeability against the
committed document identity projection, and directly emits a compact typed route. The mirror tree,
dispatcher, measured-frame store, geometry patching and route-intent bindings are deleted.

### P0: rebuild and click paths clone the same identity repeatedly

Projection clones each surface key and tab instance String. A closeable tab then clones both into
activate and close routes during every mirror rebuild, and route lookup clones the selected route
again on every click. This duplicates identity already stored in the layout.

M0 keeps typed `ViewInstanceId` once in the receipt layout. The Copy route stores only
`surface_index + item_index`; callback dispatch borrows the target and materializes a String only if
the external template-binding fallback requires it.

### P0: pointer identity projection is coupled to paint geometry

The builder accepts workbench and floating-window frames solely to construct mirror hit rectangles.
That forces every relevant host recompute to rebuild Vec/String geometry projection before equality
can reject it, even though native chrome already owns the frame used for input.

M0 removes all frame inputs and strip frames from this owner. M1 extends the native receipt with the
generation-owned `ViewInstanceId`, allowing the duplicate receipt layout and bridge to disappear.

### P1: measured-frame updates allocate and patch a second topology

When a reported tab frame changes, `projected_frame_patches` allocates a Vec, walks a suffix, checks
nodes, patches constraints and invokes `rebuild_dirty`. The native route was computed from that same
reported frame before the callback, so this entire feedback loop is post-hit reconstruction.

M0 deletes it rather than optimizing its allocation. Native paint/input generation remains the only
geometry authority.

## Zircon and Unreal source basis

Direct Zircon source read:

- `host_contract/native_pointer/routing/chrome/tabs/document/body.rs` and `close.rs` test the
  committed body/close frame and emit the exact surface, row and `close` action.
- `host_contract/native_pointer/button_dispatch/chrome_press/tabs/document.rs` splits the receipt
  into distinct body and close callbacks before `document_tab_pointer` is entered.

Direct Unreal source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp`
  - `BringToFront` passes the retained `SDockTab` directly to the tab well;
  - `CloseTab` calls `RequestCloseTab` on the exact retained tab;
  - `OnTabClosed` obtains the stable `FTabId` from that tab and updates persistent layout state.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp`
  resolves and reuses live tabs through the manager rather than reconstructing a second hit owner.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp` owns one retained
  hit path for the widget tree.

The transferable rule is that the hit owner returns a stable tab/action receipt and the docking
owner consumes it; another geometry tree must not re-verify the same click.

## Target architecture

1. Native chrome owns document-tab paint geometry and hit testing.
2. Its receipt carries action, stable tab identity and publication generation.
3. Editor command dispatch validates that generation once and borrows the typed identity.
4. Activation/close executes one typed layout transaction; no pointer mirror or string route exists.

## Instrumentation and acceptance

Matrix: surfaces `1/4/64`; tabs per surface `1/16/100/1K`; action `body/close`; topology
`stable/add/remove/reorder/floating`; event rate `10/125/500 Hz`; stale/current receipt generation.

Acceptance requires:

- editor mirror hit dispatches per confirmed native click: `1 -> 0` at M0;
- mirror `UiSurface` rebuilds and measured-frame dirty rebuilds: removed at M0;
- route-owned Strings and route lookup String clones: `2 -> 0` per click;
- per-closeable-tab rebuild String clones: `4 -> 0`;
- stable recompute duplicate receipt projection allocations: `>0 -> 0` at M1;
- receipt validation is O(1) at M1 and rejects stale generations deterministically;
- p95 command receipt-to-dispatch below 0.02 ms at 1K tabs on the recorded host;
- WPR shows no mirror layout/hit/allocation wakeups and interaction behavior remains equivalent.

RenderDoc is irrelevant to this CPU/input owner except final product pixel parity. WPR, allocator,
build and capture artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Hard-cut to native action receipts; compact route and typed borrowed identity; delete mirror geometry/surface. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | Carry generation-owned `ViewInstanceId` in native receipt and delete duplicate receipt layout/bridge. | zero stable projection allocation, O(1) generation validation |
| M2 | Make focus/close one typed layout transaction with stale-receipt diagnostics. | one authority lock/transaction and parity tests |
| M3 | Scale/storm/WPR/power and interaction/pixel parity. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 19/19 current Rust files.
- Native route/callback/command and all retained document-tab tests: read and mapped.
- Unreal tab stack/manager and Slate hit-grid source: read and mapped.
- M0 implementation: applied. Seven production mirror-tree files were removed; the surviving owner
  is a typed identity receipt projection plus O(1)-shape surface/index validation. Geometry inputs,
  `UiSurface`, `UiPointerDispatcher`, measured-frame patching and document-tab route-intent binding
  are absent from the current owner.
- Exact static owner delta: files `19 -> 12`, physical lines `740 -> 240` (-500, 67.6%), bytes
  `29,998 -> 8,501` (-21,497, 71.7%). These are source-size facts, not runtime timing claims.
- Focused static contract:
  `tools/tests/test_editor_retained_document_tab_native_receipt_performance_contract.py`, 118 lines,
  4,668 bytes, SHA256
  `01d379be5a94da3a222bb4adf166aef83644e954ac43481566c456aa06a508ca`; RED 1/6 to GREEN 6/6.
- Retained-host performance contracts: GREEN 23/23. Broad `test_*performance_contract.py`
  discovery on the current shared worktree: GREEN 192/192. Rustfmt on the touched focused Rust
  files and scoped `git diff --check` passed.
- Managed Cargo remains unavailable. The recorded Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is no longer addressable by `session show`,
  and a later session-list query timed out; raw Cargo is not an allowed bypass.
- M1-M3 and dynamic evidence remain pending; this owner stays in `pending.md`.
