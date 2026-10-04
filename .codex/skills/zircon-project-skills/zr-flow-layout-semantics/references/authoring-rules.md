# Flow Layout And Text Semantics

## Translate the reference before drawing it

For each visible region, write its role and relationship first:

| Region meaning | Usual flow treatment | Alignment |
| --- | --- | --- |
| Heading plus supporting copy | One vertical flow | Start / top |
| Row label and comparable value | Horizontal flow with a spacer or end slot | Label start, value end |
| Status with independent action | Horizontal flow | Status follows its subject; action occupies its own end slot |
| Sentence with emphasis | One rich-text source and one text flow | Source reading order; top unless the parent has a semantic baseline |
| Dense collection | One scroll owner with bounded rows | Content semantic, not visual symmetry |

Do not infer that two pieces of text are separate merely because one is bold, colored, or visually near the other. Split only when the reader can focus, activate, reorder, localize, or understand the pieces independently.

## Layout boundaries

`UiFrame` arithmetic in a native painter is the renderer's resolved output of an authored retained flow. It may derive child rectangles only from its parent frame, semantic spacing/inset tokens, content order, and fractions of available space. It must not encode viewport coordinates or make a child appear correct through an unrelated absolute offset. Raw pixel literals belong only in spacing/inset tokens; semantic control and icon sizes must be named tokens or tiers.

Prefer these authoring choices:

- Shell and ordinary panels: `HorizontalBox` or `VerticalBox` with `stretch`, explicit collapse rules, and bounded slots.
- Responsive shells: use `Stack` with breakpoint-valued `direction`/`spacing`, or breakpoint-valued `display` on semantic regions, so wide columns become a narrow vertical flow without duplicating the content tree.
- Wrapping controls/chips: `FlowBox`/`FlexBox` or `WrapBox`.
- Long logs, forms, and collections: a single `ScrollableBox` with clipping and visible scrollbar policy.
- Large or unbounded fixed-row collections: add `container.virtualization = { item_extent = ..., overscan = ... }` and materialize only the visible window.

Use an explicit-position exception only for a true anchored overlay, transient drag preview, diagram/canvas, or rendering surface. Record the anchor owner and focus/escape behavior with the component.

## Review checks

1. No coordinate-style properties appear in ordinary authored layout.
2. Parent containers, not offsets, express left/center/right and top/middle/bottom relationships.
3. Each text alignment matches the role a reader would infer.
4. Inline style differences resolve as runs of one text command.
5. Narrow, wide, empty, and overflow states preserve reading order and do not create accidental nested scrolling.

For a two-axis review, name both axes explicitly: inline `start`/`center`/`end` and block `start`/`center`/`end` (top/middle/bottom in a vertical writing mode). A value that is right-aligned because it is comparable does not become vertically centered unless its row semantics also call for that; resolve the two axes independently through the parent flex alignment and the text's own alignment.
