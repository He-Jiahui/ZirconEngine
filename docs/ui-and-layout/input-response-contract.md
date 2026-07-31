---
related_code:
  - zircon_runtime_interface/src/ui/tree/node/pointer_events.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md
doc_type: architecture-contract
---

# Editor Input Response Contract

This document defines the node-level pointer and cursor vocabulary, the authoritative hit-path shape, and the logical three-phase event order for the editor. It is the shared contract for editor input routing; it does not introduce a second runtime dispatcher or a visual-style side channel.

## Scope And Ownership

Plan 18 is deliberately staged. S1 establishes serializable DTOs and the single-source hit-path contract in `zircon_runtime_interface`. S2 will make the arranged-tree/hit-grid path and the eleven existing pointer bridges consume that contract. S3 owns pointer capture and the common drag threshold state machine. Until those later slices land, the S1 declarations do not by themselves change live grid matching or editor dispatch behavior.

`UiTreeNode` owns authored `pointer_events` and optional `cursor` declarations. Layout visibility remains a separate concern: collapsed or hidden nodes are rejected by visibility/layout rules before pointer-event policy is applied. The input layer produces semantic hover, press, active, and focus outcomes for the focus and style owners; it does not set colors, borders, or other visual state directly.

## Pointer Event Policy

`UiPointerEvents` is serialized as kebab-case and defaults to `auto` for legacy node payloads.

| Token | Self Target | Descendants | Consumption Rule |
| --- | --- | --- | --- |
| `auto` | Eligible | Eligible | Normal hit and dispatch behavior. |
| `none` | Ineligible | Ineligible | The subtree is not a pointer target; routing continues to an eligible lower layer. |
| `self-none` | Ineligible | Eligible | The container itself is skipped while its descendants remain eligible. |
| `pass` | Eligible | Eligible | The node may occur in the path but must not consume the event; an unhandled result continues to its parent. |

The public helpers make the split explicit: `allows_self_hit_test`, `allows_child_hit_test`, and `is_passthrough`. S2 is responsible for applying those helpers while creating the arranged-tree/grid hit result; S1 does not duplicate that filtering in unrelated bridge code.

## Authoritative Hit Path

`UiHitPath::root_to_leaf` is the only authored propagation order. `target` is derived as its last node and `bubble_route` is derived as its reverse. `UiHitPath::from_root_to_leaf` and `with_root_to_leaf` enforce this derivation.

`with_route` remains for existing internal callers, but accepts a supplied target and bubble route only when both agree with `root_to_leaf`; disagreement is rejected rather than silently normalized. This prevents callers from constructing contradictory target, capture, and bubble views of the same hit.

S2 will construct the path from arranged nodes and must preserve clipping, scrolling offsets, paint order, and scope checks. Node-level hit testing ends at the selected text node. Text caret placement, glyph/cluster lookup, drag selection, word selection, and line selection are delegated to the Runtime Text hit-test contract in `docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`.

## Dispatch Order

The logical hit-path sequence is fixed by `UiDispatchPhase::hit_path_sequence()`:

```text
capture (root to target) -> target (deepest node) -> bubble (target to root)
```

`capture` lets ancestors intercept before the target; `target` is the deepest node selected by the hit path; `bubble` lets ancestors observe or take over work that remains unhandled. `UiDispatchReply::merge_route` preserves ordered effects for passthrough replies and stops later route work for handled or blocked replies.

`Preprocess`, `PreviewTunnel`, `Direct`, and `DefaultAction` remain distinct pipeline phases around this logical sequence. `Capture` is appended to the enum declaration so existing serialized and ordered representation of those earlier phases remains stable; code that needs event routing order must use `hit_path_sequence`, not enum ordinal order.

## Cursor Resolution

`UiCursor` is serialized as kebab-case: `default`, `pointer`, `text`, `resize-ew`, `resize-ns`, `grab`, and `grabbing`. A `UiTreeNode.cursor` of `None` is deliberately not a cursor declaration. During S2 routing, the nearest eligible declaration on the winning hit path resolves the cursor, while an unhandled cursor query continues to the parent.

## Later Integration Rules

- S2 moves pointer bridges to the arranged-tree hit-path source without duplicating hit logic in popup, overlay, preview, or widget code.
- S3 gives capture a weak target lifetime and releases it automatically when the target disappears. Dragging follows `press -> threshold -> start -> move -> drop`, with a logical-unit threshold adjusted by the resolution context.
- Wheel input follows the same capture/target/bubble sequence. Its x and y components may be consumed independently by the nearest scrollable ancestor; Ctrl+wheel is a viewport zoom capability, not implicit scrolling.
- Popup dismissal, overlay/preview ordering, focus-path keyboard routing, multi-click timing, and tooltip timing remain owned by their respective route or timer components. They must consume the path contract rather than create a competing hit order.

## Current Contract Coverage

`zircon_runtime_interface/src/tests/input_response_contracts.rs` fixes the S1 invariants with JSON token round trips, legacy node defaults, pointer-policy distinctions, derived hit paths, inconsistent legacy-path rejection, and the explicit capture/target/bubble sequence. Cargo validation is recorded only after the managed reservation for the matching source manifest completes.
