# Runtime Rich Table Cell Box Design

Date: 2026-07-12
Owner plan: `docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
Milestone: RT-M6
Status: approved through the user's standing instruction to execute the recommended option without further confirmation

## Objective

Complete the first visual box-model slice for BBCode rich-table cells. A cell may define padding, alternating row background colors, and a border color; layout must resolve the final box after colspan/rowspan sizing, and the production WGPU renderer must paint that box around the real shaped text.

This slice does not add VerticalRl tables, table interaction, rounded borders, gradients, theme-token lookup, or CSS/HTML table syntax.

## Reference contract

Godot `RichTextLabel` is the BBCode semantic authority for this slice:

- `[cell border=<color>]` selects a cell outline color.
- `[cell bg=<color>]` uses one color for every row.
- `[cell bg=<odd>,<even>]` selects the background by the cell's resolved start-row parity.
- `[cell padding=<left>,<top>,<right>,<bottom>]` supplies four logical-pixel insets.

Zircon intentionally accepts only the existing hexadecimal color grammar (`#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA`). It does not add named colors or compatibility aliases.

## Chosen architecture

### Neutral parser DTO

`RichTableCell` owns a serializable `RichTableCellBoxStyle` containing:

- optional left/top/right/bottom padding;
- optional odd-row and even-row background colors;
- optional border color.

Missing padding preserves the current font-relative table padding. A valid authored padding replaces that fallback. Invalid color or padding input does not partially mutate the style.

### Hostile-input rules

- `padding` must contain exactly four finite numbers.
- Negative padding becomes zero.
- Each side is capped at 4096 logical pixels.
- A malformed/non-finite tuple is ignored as a whole and uses the existing default padding.
- One valid `bg` color populates both row colors.
- Two valid `bg` colors populate odd/even colors in order.
- Any malformed background tuple or malformed border color becomes transparent/unset.
- Border thickness is fixed at 1 logical pixel for RT-M6; no BBCode width attribute is introduced.

### Layout ownership

The rich-table layout owner resolves padding before measurement:

1. preferred cell width = unwrapped content width + left + right;
2. required cell height = wrapped content height + top + bottom;
3. colspan/rowspan track solving remains unchanged;
4. after final track sizes are known, layout emits one `UiResolvedTextBox` for each styled cell using the complete span frame;
5. the box background is selected from the cell's resolved start-row parity.

`UiResolvedTextLayout` carries `boxes: Vec<UiResolvedTextBox>` as resolved geometry metadata. Ordinary text layouts leave it empty. Nested/top-level layout composition preserves absolute box frames just as it preserves line frames.

### Render ownership

`UiRenderCommand::text_paint()` projects each resolved box into up to two `UiTextPaintDecoration` records:

- `TableCellBackground`, painted before glyph batches with `push_rect`;
- `TableCellBorder`, painted after glyph batches with `push_border` and the resolved 1px thickness.

The renderer never reparses padding or reconstructs table geometry from line bounds. Parser owns authored semantics, layout owns geometry, and renderer owns draw ordering only.

## Rejected approaches

1. Reconstruct cell boxes in the renderer from line frames. This cannot represent empty cells or recover exact row-span height and padding.
2. Emit independent UI quad commands during text layout. This couples text layout to surface-command orchestration and creates a second ordering/clip truth.
3. Paint only run-level `bgcolor`. That covers glyph runs rather than the complete cell and does not satisfy table box semantics.

## Validation

The milestone requires all of the following:

- parser tests for one/two backgrounds, border, padding, invalid tuples, and clamping;
- layout tests proving padding affects content origin and measure, and span boxes use the final combined frame;
- render-contract tests proving background-before-text and border-after-text projection;
- current `text_rich` regression suite green;
- a real ignored WGPU product-framebuffer test with visible cell backgrounds, borders, padding, wrapping, colspan, rowspan, and inline icon;
- accepted PNG stored only under `docs/tests/runtime/text` with dimensions, byte size, and SHA-256 recorded in the Text07 child output record.

## Structure constraints

- Keep parser attribute decoding in `graphics/text/rich/bbcode_table/attributes.rs` or a focused child module if that owner approaches its budget.
- Keep box projection separate from the rich-table sizing solver.
- Keep production owners below the repository review thresholds; do not stack new render behavior into an already oversized root.
- Do not add compatibility facades, duplicate cell-style types, renderer-side BBCode parsing, or test-only rendering paths.
