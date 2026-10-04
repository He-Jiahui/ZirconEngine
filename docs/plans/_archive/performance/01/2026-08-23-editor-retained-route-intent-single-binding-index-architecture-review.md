---
title: Editor retained route intent single binding index performance review
date: 2026-08-23
module: zircon_editor retained-host route_intent
priority: MVP-P0 retained pointer lookup and typed route ownership
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate FHittestGrid bubble path
---

# Goal

Resolve a retained hit node to its typed editor intent with one stable owner and one hot lookup.
Pointer dispatch must not traverse a node-to-route table and then a route-to-intent table when no
product consumer performs route-id lookup, and stable movement must not deep-clone String/Vec route
payloads as the long-term contract.

## Reviewed source

- pre-M0 owner Rust files: 2/2
- pre-M0 physical lines: 200
- pre-M0 bytes: 6,920
- pre-M0 LF path-tab-file-SHA manifest SHA256:
  `755789807291b0a54cfb3b273a7130dca4d2c93d9c1d541b14e5368b7fab26d7`
- post-M0 owner Rust files: 2/2
- post-M0 physical lines: 196
- post-M0 bytes: 6,597
- post-M0 LF path-tab-file-SHA manifest SHA256:
  `207eaee1e4539a8bd27819eb4608909ca9c5c11d7db49c072a2e82969b9e99b4`
- current post-document-receipt owner Rust files: 2/2
- current post-document-receipt physical lines: 184
- current post-document-receipt bytes: 6,150
- current post-document-receipt LF path-tab-file-SHA manifest SHA256:
  `d5f486e273d8976c34a85cef2ca7b3c135ce74a142e470105154765e7a66677b`
- current post-drawer-receipt owner Rust files: 2/2
- current post-drawer-receipt physical lines: 173
- current post-drawer-receipt bytes: 5,707
- current post-drawer-receipt LF path-tab-file-SHA manifest SHA256:
  `867198883baf866b256b0d2bcb5bc3cf7d670a7485bae18157e2fe0ae2985adb`
- owning commit at review: `d7941242fe6adfeac865bf00ff3fa18a630e5662`

Both owner files were read in full. All typed route definitions and all activity, document-tab,
drawer-header, menu, hierarchy, detail, host-page, viewport-toolbar, welcome and shell pointer
accessor call sites were traced. The 2026-07-17 report correctly replaced ordered maps but now
preserves a redundant two-index architecture and uses Bevy rather than the required Unreal-primary
source basis; this report supersedes it for current source.

## Correct foundations to retain

1. Node and route ids are compact hash keys; ordered lookup is not required.
2. `handled_by` correctly takes precedence over the raw hit target after pointer handlers run.
3. The map keeps typed domain routes instead of parsing path strings during each pointer fact.
4. Accessors reject a mismatched domain variant rather than letting one bridge consume another
   bridge's route payload.

## Structural findings

### P0: every pointer intent performs two hash probes through a dead route-id join

`intent_for_pointer_dispatch` calls `route_id_for_pointer_dispatch`, which maps hit node to
`UiRouteId`, then maps `UiRouteId` to `EditorRouteIntent`. Repository-wide source tracing finds no
external caller of `route_id_for_node`, `intent_for_route_id`, `intent_for_node`, component
`intent_for` or `route_id_for_pointer_dispatch`; all product callers request the typed intent for a
node/dispatch. The intermediate route-id join therefore doubles hot hash probes and retains a second
hash table without an active consumer.

M0 replaces the two maps with one node-to-`EditorRouteBinding { route_id, intent }` map. Route id is
retained temporarily for source/test compatibility, while pointer intent performs one node lookup.
M1 removes route id from editor bridge binding once runtime route identity owns the contract.

### P0: typed accessors clone wide route payloads on stable facts

Nine typed accessors clone their route. Document tabs, drawer headers, activity rail, hierarchy,
host page, toolbar and welcome routes contain owned Strings; host-page overflow and menu routes also
contain Vec paths. Move-only dispatch can therefore allocate/copy payloads after the one/two hash
lookups. Returning a borrowed enum directly is not a safe local fix because callers return owned
dispatch records and often mutate the bridge after lookup.

M1 makes generation-owned route records use stable typed ids and shared immutable payloads. Event
replies carry a compact handle/action receipt; owned Strings are materialized only at the command or
external host boundary.

### P1: route ids are generated and stored but do not participate in runtime dispatch

Every bridge calculates a `UiRouteId` and passes it to `bind_node`, yet the runtime pointer result
contains node/path identity and current editor product code never reads the route id. M1 chooses one
authority: either runtime publishes route identity in its dispatch result, or editor binds typed
intent directly to stable node id. Keeping both indefinitely is rejected.

### P1: generic component/editor intent surface is dormant

`EditorRouteIntent::Editor(EditorIntent)` and `intent_for(UiComponentEventReport)` have no current
product call site. They enlarge the generic enum/API but do not currently cost per pointer fact.
M1 verifies plugin/component reachability and hard-cuts or reconnects these contracts; M0 does not
remove them without a compiled behavior gate.

### P1: maps are rebuilt independently by every pointer bridge

Activity, detail, tabs, drawers, hierarchy, pages, menu, toolbar, welcome and shell owners each
construct/replace an `EditorRouteIntentMap` during surface rebuild. This duplicates lifecycle and
prevents one atomic runtime topology transaction. M2 merges route binding with the runtime surface
generation and updates only changed nodes/subtrees.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`
  - `FHittestGrid` retains one `WidgetMap` from widget identity to stable widget-array index;
    `GetBubblePath` returns the actual event bubble path from the hit owner.
  - explicit `AddWidget`, `RemoveWidget` and `UpdateWidget` mutate that retained identity/index.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
  - `GetBubblePath` selects the retained hit widget data and produces the path used for routing.
  - `AddWidget` performs one `WidgetMap.Find`; unchanged cell coverage updates metadata in place,
    while changed coverage removes/reinserts the widget.

The transferable rule is one retained identity lookup yielding the route path/owner. Zircon should
keep typed editor semantics, but it should not add an editor-only route-id join that no caller uses.

## Target architecture

1. M0 uses one node binding hash map and one pointer lookup.
2. Runtime surface topology publishes stable route identity and typed owner handles under the same
   generation as arranged/hit data.
3. Editor consumers store compact stable ids or shared immutable route payloads, not cloned Strings
   and paths per event.
4. Insert/remove/update of node, handler and intent is one atomic topology transaction.
5. Typed interaction replies carry the matched owner/action plus exact changed/damage domains.

## Instrumentation and acceptance

Matrix: bound nodes `1/100/1K/10K/100K`; route payload `copy/String/Vec/nested`; event
`move/scroll/down/up`; result `handled/raw-target/missing/mismatched`; topology
`stable/add/remove/update`; rate `125/500/1,000 Hz`.

Acceptance requires:

- M0 pointer intent hash probes `2 -> 1` and retained hash maps `2 -> 1`;
- stable pointer fact route String/Vec clones and copied bytes = 0 after M1;
- missing/mismatched routes allocate nothing and preserve fallback semantics;
- node/handler/intent generation is identical and stale receipts are rejected;
- topology update visits only changed nodes and does not rebuild unrelated surface indexes;
- p95 typed route resolution below 0.02 ms at 100K bindings on the recorded host;
- WPR shows no sustained route-map allocation/wakeup and input behavior remains equivalent.

RenderDoc is not a CPU/hash/allocation profiler; use it only for final current-source hit/paint pixel
parity. WPR, allocator and build artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Collapse node/route two-map join into one binding map. | applied; focused RED 0/3 to GREEN 3/3 |
| M1 | Replace owned String/Vec event clones with generation-owned stable route handles. | zero stable route payload clones |
| M2 | Publish route bindings in runtime atomic surface topology transactions. | changed-node-only update reports |
| M3 | Run scale/storm/WPR/power and behavior parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 2/2 current Rust files.
- All typed route definitions and product accessor call sites: read and mapped.
- Unreal hit-grid identity/bubble-path implementation: read and mapped.
- M0 implementation: applied. `EditorRouteBinding` retains the compatibility route id and typed
  intent together in one node-keyed map. Pointer dispatch resolves handled/raw target directly to
  `intent_for_node`; the second map and unused route-id reverse/pointer lookup methods are removed.
- Follow-on hard cutover: the document-tab owner now consumes the native action receipt directly,
  so the dead `EditorRouteIntent::DocumentTab` variant and cloning accessor were removed. This is a
  consequence of the document-tab authority change, not a generic enum-size micro-optimization.
- Drawer-header follow-on: native drawer hits now use the same receipt boundary, so the dead
  `EditorRouteIntent::DrawerHeader` variant and cloning accessor were also removed.
- Static code-path delta: retained hash maps `2 -> 1`, typed pointer intent hash probes `2 -> 1`,
  reverse/pointer route-id lookup APIs `2 -> 0`. These are exact source-path counts, not elapsed-time
  profiler claims. Owned route payload clones remain explicitly assigned to M1.
- Focused static contract:
  `tools/tests/test_editor_retained_route_intent_single_binding_performance_contract.py`, 50 lines,
  1,778 bytes, SHA256
  `77d5bb3dd15a1f193364db96ffa8907723dc2588aaae7903de4d3d60ff255b35`; RED 0/3 to GREEN 3/3.
- Adjacent retained-menu/menu-geometry/visible-row/context-menu/host-interaction contracts: GREEN
  22/22. Rustfmt and scoped `git diff --check` passed.
- Broad performance-contract discovery on the current shared worktree: GREEN 199/199.
- Current owner size versus the reviewed pre-M0 source decreased by 27 physical lines and 1,213
  bytes; the original single-index M0 delta was 4 lines and 323 bytes.
- Managed Rust tests, M1-M3 and dynamic evidence: pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until all milestones pass on one source/executable fingerprint.
