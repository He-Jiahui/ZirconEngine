---
title: Editor retained host page native action receipt hard cutover performance review
date: 2026-08-23
module: zircon_editor retained-host host_page_pointer
priority: MVP-P0 main-page activation close and overflow
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine SDockTab and SDockingTabStack
---

# Goal

Make the committed native host-page chrome the sole paint and hit authority. A confirmed tab-body,
close-button, or overflow-button press must carry an explicit action receipt into editor dispatch;
the editor must not reconstruct the page strip, measure every title, patch a second frame store, or
dispatch through a second hit tree to rediscover the action.

## Reviewed source

- pre-M0 owner Rust files: 21/21
- pre-M0 physical lines: 933
- pre-M0 bytes: 33,176
- pre-M0 LF path-tab-file-SHA manifest SHA256:
  `244176a8ef08d5e437df1d6076d1791880da0b06fe4c372254e871e88d9c527c`
- post-M0 owner Rust files: 11/11
- post-M0 physical lines: 171
- post-M0 bytes: 5,749
- post-M0 LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `4411ea27a6059acd857d2a87c389c27cf23c9b614c5cee30b4ab0ab9b2e7da3a`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`

All owner files were read in full. The review also traced the five retained host-page test modules,
host recompute and callback wiring, native chrome routing/button dispatch, overflow pointer and
keyboard selection, scene/template projection, page-tab frame indexing, typed workbench page/view
identities, shared activation/close commands, and relevant Unreal docking widgets.

The initial concern that overflow replacement loses the original page index was disproved. Zircon's
fallback layout replaces the last visible row with the active original row, but `tab_frames` still
projects the complete page list in original order and gives non-visible rows zero frames. Native
`enumerate()` therefore currently preserves the original page index. This is not recorded as a bug.

## Structural findings

### P0: one committed native hit is repeated against a complete editor mirror

`route_host_page_tabs` iterates committed `HostChromeTabData.frame` values and emits a route only
after native `contains` succeeds. The callback then forwards tab geometry and a local point into
`HostPagePointerBridge::handle_click`, which updates measured frames, translates the point and sends
another Down event through `UiSurface`, `UiPointerDispatcher` and `EditorRouteIntentMap`.

The second hit tree is not a validation boundary: it can disagree with the already painted native
generation. M0 makes native routing test `close_frame` before `frame`, includes an explicit close
action bit in the receipt, and reduces the editor owner to typed page/view identity lookup.

### P0: every host recompute repeats the page-strip layout algorithm

`build_host_page_pointer_layout` derives strip geometry from outer-shell frames and metrics, clones
page ids/titles/close ids, measures every title and reruns visible/overflow allocation. Native scene
projection separately composes the same page chrome, measures the same titles, selects visible tabs,
publishes tab/close/overflow frames, and paints them. Stable bridge equality occurs only after the
duplicate O(P) projection and allocations have already happened.

M0 removes geometry, title and overflow allocation from the editor receipt projection. The compact
projection remains O(P) because it snapshots typed page/view identities for stale-receipt checking.
M1 moves those identities under the same native presentation generation so stable recompute becomes
O(1) and allocates nothing.

### P0: native close identity is degraded to coordinates and inferred again

`HostChromeTabData` already contains the tab body frame, close frame, stable tab id and closeability.
Nevertheless, host-page routing ignores `close_frame` and returns body geometry plus local pointer
coordinates. The editor mirror reconstructs a close frame and decides whether the press meant close.
This is both redundant and weaker than the native result: authored and fallback layout changes can
make the two frame generations disagree.

M0 follows the existing document-tab hard cut: native close-before-body routing emits one explicit
action, while the editor validates the typed target and closeability without geometry.

### P1: selected actions allocate owned String route payloads

The current mirror route owns `page_id: String` or `instance_id: String`, and route-intent lookup
clones the selected route. M0 makes the route Copy with `item_index + action`, retains `MainPageId`
and `ViewInstanceId` in the projection, and borrows the target until an owned layout command is
required.

### P1: overflow re-hit creates data that the consumer discards

Native overflow routing already confirms the committed overflow frame. The editor mirror re-hits a
second overflow node and clones `hidden_page_indices`, but the app ignores that vector and only
toggles `HostPageOverflowMenuStateData`; presentation-owned popup layout already reads native
`overflow_hidden_tab_indices`. M0 makes overflow an explicit native receipt and removes the editor
overflow geometry/hit payload. Pointer and keyboard row selection continue to pass the original
page index.

## Zircon and Unreal source basis

Direct Zircon source read:

- `workbench_host_window/chrome_template_projection.rs` owns visible-row selection, page title
  measurement, close-node placement, overflow placement and complete indexed `tab_frames`.
- `host_contract/native_pointer/routing/chrome/tabs/host_page.rs` already hits the committed native
  tab body; its failure to test the committed close frame is the missing action distinction.
- `host_contract/native_pointer/button_dispatch/page_overflow_menu.rs` and native keyboard dispatch
  already select overflow rows by their original `page_index`.
- `host_page_pointer/**` independently repeats geometry, measurement, surface build, dirty rebuild,
  hit dispatch and owned route construction after the native result.

Direct Unreal source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Docking/SDockTab.cpp`
  - tab pointer handling calls `ActivateInParent` on that `SDockTab` instance;
  - the nested `SButton` binds `OnClicked` directly to `SDockTab::OnCloseButtonClicked`;
  - `OnCloseButtonClicked` calls `RequestCloseTab` on the same tab instance.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp`
  receives the exact retained `SDockTab` for close and activation operations.

The transferable rule is that the arranged widget which won hit testing emits a typed action on its
retained tab identity. A downstream subsystem must not reconstruct geometry to distinguish body
from close or rediscover the target.

## Target architecture

1. Native page chrome remains the sole page-tab, close-button and overflow hit authority.
2. Native route distinguishes `activate`, `close` and `overflow` before crossing the callback ABI.
3. The receipt carries original item index plus presentation generation; geometry is damage-only.
4. Editor dispatch validates generation/index once and borrows typed `MainPageId`/`ViewInstanceId`.
5. Activation or close executes one typed layout transaction; overflow only updates popup state.

## Instrumentation and acceptance

Matrix: pages `1/16/100/1K`; width `320/720/1280/3840`; action
`activate/close/open-overflow/select-overflow`; topology `stable/add/remove/reorder`; rate
`10/125/500 Hz`; receipt generation `current/stale`.

Acceptance requires:

- editor mirror hit dispatches per confirmed native press: `1 -> 0` at M0;
- editor page-strip title measurements per stable recompute: `P -> 0` at M0;
- mirror surface rebuild and measured-frame dirty-rebuild paths: removed at M0;
- callback geometry scalars: `4 -> 0`; explicit action field: `0 -> 1`;
- selected route-owned String payloads: `1 -> 0`;
- overflow hidden-index clone in editor dispatch: `1 -> 0`;
- stable recompute receipt projection allocations: `>0 -> 0` at M1;
- stale receipts are rejected deterministically under one native publication generation;
- p95 native-receipt-to-command dispatch below 0.02 ms at 1K pages on the recorded host;
- WPR shows no mirror layout/text-measure/hit/allocation wakeups and interaction remains equivalent.

RenderDoc is relevant only to final product pixel/draw parity because M0 does not change paint
geometry. WPR, allocator, build and capture artifacts must remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Hard-cut to explicit native action receipt; compact typed target projection; delete mirror geometry/surface/overflow re-hit. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | Publish generation-owned page/view identities beside native page chrome and delete duplicate bridge projection. | zero stable projection allocation, stale-generation tests |
| M2 | Make activation/close one typed authority transaction with exact invalidation/damage receipt. | one transaction and behavior parity |
| M3 | Run scale/storm/WPR/power plus interaction and RenderDoc pixel/draw parity. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 21/21 Rust files.
- Related retained tests, native routing/callback, model/command chain and overflow paths: read and
  mapped.
- Unreal `SDockTab` and `SDockingTabStack` action/identity source: read and mapped.
- Architecture report: recorded before implementation.
- M0 implementation: applied. Native page chrome tests the committed close frame before the tab
  body and emits `index + close`; the callback ABI no longer carries four geometry scalars. The
  editor owner retains typed page/view identities only, and ten mirror geometry/measurement/hit
  files plus the host-page route-intent binding are deleted.
- Exact static owner delta: files `21 -> 11`, physical lines `933 -> 171` (-762, 81.7%), bytes
  `33,176 -> 5,749` (-27,427, 82.7%). These are source-size facts, not timing claims.
- Focused static contract:
  `tools/tests/test_editor_retained_host_page_native_action_receipt_performance_contract.py`, 139
  lines, 5,350 bytes, SHA256
  `d2ce2132f7ada558b43333c85bdc83276868c26427944d35befd113b6c9920a7`; RED 0/8 to GREEN 8/8.
- Retained-host performance contracts: GREEN 40/40. Broad `test_*performance_contract.py`
  discovery on the current shared worktree: GREEN 209/209. Rustfmt on focused Rust files and scoped
  `git diff --check` passed.
- The stale boundary test that required document/drawer/host-page mirror surfaces was updated to
  reject those surfaces, dispatchers and route-intent maps instead.
- M1-M3 and dynamic evidence remain pending; this owner stays in `pending.md`.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is archived and returns
  `cargo_session_not_executable`; raw Cargo is not an allowed bypass.
