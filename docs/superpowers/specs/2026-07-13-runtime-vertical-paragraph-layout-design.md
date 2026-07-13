# Runtime VerticalRl Paragraph Layout Design

## Goal

Close Text03's remaining VerticalRl paragraph gap: BBCode paragraph alignment, first-column indent, nested indent, and hanging continuation constraints must operate on the vertical inline axis instead of being bypassed by `vertical.rs`.

## Architecture

- Parser `ParagraphOverride` remains writing-mode neutral.
- `paragraph_layout.rs` remains the single owner of merged overrides, hostile-indent clamping, list-prefix measurement, and first/continuation constraints.
- HorizontalTb maps the scalar paragraph inset to physical x/width.
- VerticalRl maps the same scalar inset to physical y/height.
- `vertical.rs` continues to own shaped VerticalRl columns and right-to-left physical column placement; it consumes paragraph constraints but does not parse BBCode.
- Alignment operates on the VerticalRl inline axis: Left/Start at physical top, Center centered in remaining height, Right/End at physical bottom. Justify retains the shared advance-distribution path.
- Measured content extents remain content-based; alignment offsets do not manufacture extra glyph advance.

## Required contracts

1. `[p indent=...]` offsets only the first physical column on y; a wrapped continuation returns to paragraph continuation origin.
2. `[indent]` offsets both first and continuation columns on y.
3. Center and Right paragraph alignment place a short VerticalRl column at the center/bottom of the available inline extent.
4. Horizontal paragraph regressions remain unchanged.
5. No renderer, parser, or interface branch is added.

## Deferred boundary

Vertical rich-inline objects currently use the dedicated `rich_inline_vertical.rs` metric path. Combining those objects with paragraph indent/alignment requires that owner to consume the same `ColumnConstraints`; the generic paragraph wrapper cannot replace it without losing inline-object advance/cross-extent data. This composition stays explicit and open instead of silently routing object replacement characters through the plain-text wrapper.
