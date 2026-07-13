# Runtime Rich-Table Interaction Design

## Goal

Close Text07's table-specific interaction gap by making rich links inside HorizontalTb and VerticalRl table cells activate through the existing surface input pipeline, without adding table syntax or cell reconstruction to input dispatch.

## Root cause

`hit_test_text_layout` currently selects a candidate line from only the writing-mode block axis before it checks the full line frame. In a horizontal table, multiple cells share physical y; in a vertical table, multiple logical columns can share physical x. The first resolved line on that axis can therefore win even when the pointer lies inside a later cell, after which `inside_line == false` suppresses the valid link.

## Selected architecture

1. `UiResolvedTextLayout.lines` remains the single geometry source.
2. Shared hit testing first searches for a line whose complete physical frame contains the pointer.
3. Only when no physical line contains the point does it retain the existing nearest-row or nearest-column caret fallback.
4. `link_at_layout_point` continues to map the resulting source caret and affinity onto parser-owned `LinkRef` ranges.
5. Surface dispatch continues to emit the existing controlled `RequestLinkActivation` effect and host request.
6. `UiResolvedTextBox` remains paint/cell geometry. Cell padding, background, and border are not implicitly clickable.

This matches the reference-engine boundary: Godot descends into table cell frames for click discovery before resolving metadata, while Unreal hyperlink runs consume layout-block hit indices rather than rebuilding container structure.

## Required contracts

- A HorizontalTb BBCode table link in the second cell of a shared row activates.
- A VerticalRl BBCode table link in the second physical inline slot activates.
- A point inside cell padding but outside the linked line does not activate.
- Surface primary-release dispatch emits one existing host activation request for a table-cell link.
- Ordinary paragraph affinity and non-table link behavior remain unchanged.

## Non-goals

- No new table-cell click event.
- No cell selection/editing model.
- No clickable background or border semantics.
- No renderer branch, parser branch, compatibility facade, or public interface expansion.

