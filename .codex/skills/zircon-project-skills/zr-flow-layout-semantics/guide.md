# Zircon Flow Layout Semantics

Use this skill for new or changed application-facing layouts in Hub CSS/TSX, `.zui` assets, and retained/native UI painters. It governs authored layout, not the renderer's final resolved `UiFrame` geometry.

## Authoring rules

- Read the selected ReactBits reference as an information architecture and interaction source before writing layout. Inspect its visible content and states; determine whether a region is a label, value, action, status, timestamp, or independent control before assigning alignment.
- When a reference is embedded in an iframe, inspect the iframe's DOM/HTML, resolved CSS, and relevant state/interaction script before treating a visible grouping as a layout rule. Do not infer separate text blocks, fixed offsets, or alignment merely from a screenshot crop.
- Use normal document flow: `HorizontalBox`, `VerticalBox`, `ScrollableBox`, `FlowBox`/`FlexBox`, `stretch`, min/max constraints, and semantic slots. In Hub, use flex flow with responsive constraints. Do not introduce CSS grid or a coordinate canvas for ordinary layout; preserve an existing semantic table/data-grid primitive only when the content is genuinely a comparable matrix and its local component contract already requires it. Do not position ordinary application content with `x`, `y`, `left`, `right`, `top`, `bottom`, transforms, `layout_offset_*`, `CanvasBox`, or absolute/fixed positioning.
- Raw pixel values are allowed only for padding, margin, gap/spacing, or an explicitly named local inset token. Border widths, control extents, and icon extents must come from existing semantic tokens or named tiers rather than a new guessed number. No pixel value may encode a screen coordinate or compensate for a missing parent/child flow relationship. Intrinsic interaction bounds need an explicit semantic reason rather than a guessed screen measurement.
- A real overlay, popover, drag preview, canvas, or diagram may use explicit placement only when its owner, anchor, clipping, focus behavior, and fallback flow are documented. Do not use an overlay exception to arrange a normal panel, row, or text block.
- Give one scroll owner to a long region. State whether the region grows with content, is bounded and scrolls, pages, or virtualizes; do not clip a potentially long collection merely to preserve a screenshot.

## Text semantics

- Default text alignment is start/left and top. Use end/right for values intended to compare vertically (counts, money, duration, timestamps, completion/state values), and use middle/center only for a deliberately standalone badge, icon-only control, symmetric state, or control whose content is intentionally centered. Use bottom/end only when the content's baseline or bottom edge is the semantic anchor (for example, a baseline-aligned metric row); do not choose top/middle/bottom or left/center/right to balance whitespace in a screenshot. `text_align` and cross-axis alignment follow meaning, not empty-space balancing.
- Treat adjacent words as one text flow unless they have separate focus, action, localization, reading order, or status semantics. A bold word inside a sentence is a rich-text run, not a second label placed beside the first.
- Use the existing versioned rich-text pipeline (`rich_text_format = "markdown_inline_v1"`, `bbcode_v1`, or `html_subset_v1`) for inline emphasis. Keep one source string and verify its resolved runs; never fake emphasis with spacing or duplicated text nodes.
- Never rely on manual spaces to push a value to the right. Use a flow spacer, end-aligned child slot, or the component's alignment contract.

Read [the detailed rules](references/authoring-rules.md) when selecting a container, translating a ReactBits layout, or reviewing a native painter. Pair this skill with `zr-ui-layout-reference`, `zr-icon-size-scale`, and `zr-i18n-content-contracts` for application UI work.
