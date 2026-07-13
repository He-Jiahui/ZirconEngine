# Runtime Rich Table VerticalRl Design

Date: 2026-07-12
Owner plan: `docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
Milestone: RT-M7

## Goal

Extend the accepted RT-M4–RT-M6 rich-table path to `UiTextWritingMode::VerticalRl` without creating a rotated renderer path, a vertical-only parser, or a second span solver. The result must lay out real vertical cell text, colspan/rowspan, physical padding, backgrounds, borders, nested rich content, and surrounding blocks through the existing neutral DTO and resolved-layout contracts.

## Current Boundary

`layout_parsed_text_with_provider` already gives rich tables first ownership of parsed table blocks. `rich_table::layout_rich_tables_with_provider` currently returns `None` for `VerticalRl`, so the ordinary vertical layout receives stripped cell text without table geometry. Horizontal table layout already owns deterministic parser coordinates, span-aware track solving, per-cell padding, resolved boxes, nested-table depth descent, and renderer-neutral paint projection.

The change is a leaf extension inside the existing Runtime text abstraction:

- `zircon_runtime::core::framework::render` continues to own neutral `RichTable` DTOs;
- `zircon_runtime::graphics::text` continues to own parsing and vertical shaping;
- `zircon_runtime::ui::text::layout_engine::rich_table` owns physical table layout;
- Runtime Interface continues to transport only resolved physical frames;
- the screen-space renderer continues to paint decorations and glyphs without table syntax or grid knowledge.

## Reference Evidence

- Godot `RichTextLabel::ItemTable` keeps table cells as nested frame owners and performs measure before final table sizing. Zircon keeps this frame/table separation and the accepted parser DTO.
- Fyrox `Grid` separates row/column measurement from final arrangement and measures constrained children before arranging tracks. Zircon keeps the same two-axis measure/arrange discipline.
- Slint grid layout likewise treats cell constraints and final physical placement as distinct phases.
- Zircon's existing `graphics/text/layout/vertical_layout.rs` remains the authority for VerticalRl column placement inside each cell. The table layer maps grid axes only; it does not duplicate glyph orientation or vertical line layout.

Godot does not provide the required VerticalRl table behavior as a drop-in rule. Zircon therefore uses CSS-style logical writing axes while retaining Godot-aligned BBCode table attributes.

## Considered Approaches

### A. Keep the physical horizontal grid and only make cell text vertical

This preserves current geometry but makes rows continue top-to-bottom and columns left-to-right. It is easy to implement, but the table is not a VerticalRl formatting context: colspan and rowspan remain bound to the wrong physical axes, and surrounding blocks do not flow right-to-left.

Rejected.

### B. Lay out horizontally and rotate the completed table

This can transpose simple frames, but physical padding, nested table clips, inline object placement, source-range boxes, and surrounding block order would require post-layout reconstruction. It creates a second geometry truth and encourages renderer special cases.

Rejected.

### C. Shared logical inline/block axes

Keep parser columns and rows stable, solve their logical extents once, and map logical frames to physical frames by writing mode:

- HorizontalTb: inline `x` grows right, block `y` grows down.
- VerticalRl: inline `y` grows down, block `x` grows from right to left.

Recommended and selected under the user's standing instruction to execute the recommended option without another confirmation gate.

## Axis Semantics

`RichTableColumn` remains a logical column policy in both writing modes.

| Logical concept | HorizontalTb physical mapping | VerticalRl physical mapping |
|---|---|---|
| column track / colspan | width / x span | height / y span |
| row track / rowspan | height / y span | width / right-to-left x span |
| inline start | left | top |
| block start | top | right |
| column gap | horizontal gap | vertical gap |
| row progression | down | left |

The first parser row is the rightmost physical row stripe in VerticalRl. Cells within that row progress from top to bottom by column index. `colspan` increases physical height; `rowspan` increases physical width toward the left.

## Padding and Cell Content

BBCode `padding=left,top,right,bottom` stays physical. It is not reinterpreted as logical padding.

- Horizontal preferred column extent adds `left + right`; the row constraint adds `top + bottom`.
- Vertical preferred column extent adds `top + bottom`; the row constraint adds `left + right`.

Cell content is laid out with the existing writing mode. In VerticalRl the content frame is bounded by the resolved logical column span height and provisionally given enough block width to measure all wrapped vertical columns. After row extents are solved, prepared content is translated by its block-start anchor: top-left for HorizontalTb, top-right for VerticalRl. Lines and nested resolved boxes move together and are clipped only after final placement.

## Surrounding Block Flow

Top-level rich content keeps source order around tables.

- HorizontalTb consumes measured block height and advances `y`.
- VerticalRl consumes measured block width and moves the available right edge left.

`before[table]...[/table]after` therefore yields rightmost `before`, then the table, then leftmost `after` in VerticalRl. The table owner reuses the ordinary vertical layout for non-table ranges.

## Module Structure

The existing 606-line `rich_table.rs` must not grow. It becomes a wiring file:

```text
ui/text/layout_engine/
  rich_table.rs                 # child declarations + exported entry only
  rich_table/
    axes.rs                     # logical-to-physical mapping and block flow
    cell_layout.rs              # padding, preferred extents, prepared cells, boxes
    grid.rs                     # bounded parser coordinate projection
    layout.rs                   # source-order orchestration and table measure/arrange
    sizing.rs                   # column/row span constraint solving
    source_slice.rs             # rich range slicing and source-range shifting
  tests/
    rich_table/
      mod.rs                    # shared fixtures/helper only
      horizontal.rs             # accepted RT-M4–RT-M6 regressions
      vertical_rl.rs            # RT-M7 logical-axis regressions
```

No compatibility module or old horizontal implementation survives beside the shared logical-axis path.

## Failure and Bounds Behavior

- Existing parser column/row/span bounds remain authoritative.
- Zero available inline or block extent produces bounded zero/minimum tracks and `overflow_clipped`; it does not panic.
- Non-finite intermediate constraints are sanitized by the existing sizing owner.
- Nested tables still require strictly greater parser depth than the parent table.
- Prepared layout block budgets are derived from bounded source length and line advance, then clamped to the shared unconstrained-layout ceiling; hostile input cannot request an infinite frame.
- Physical boxes outside the final clip are removed together with out-of-clip lines.

## Tests

### Focused layout contracts

1. A 2x2 VerticalRl table places columns top-to-bottom and rows right-to-left.
2. Vertical `colspan` increases physical height and `rowspan` increases physical width.
3. Physical top/right padding controls the first vertical content column while the resolved box retains authored background/border.
4. Surrounding VerticalRl blocks preserve source order along the right-to-left block axis.
5. Existing ten horizontal table regressions remain unchanged and pass through the same shared axis path.

### Product framebuffer

Add a styled VerticalRl BBCode table beside the accepted horizontal RT-M6 table in the existing multilingual product exporter. The pre-render gate must assert writing mode, row/column physical ordering, span geometry, physical padding, resolved box count, and paint decoration count before WGPU creation. The existing per-command changed-pixel comparison and framebuffer readback remain mandatory.

The accepted PNG path is:

`docs/tests/runtime/text/runtime_text_multilingual_rich_table_vertical_rl_product_framebuffer_20260712.png`

No proof image may be written below a repository or external Cargo target.

## Acceptance

- exact leased Rust files pass rustfmt and scoped diff checks;
- Runtime Interface production/test remains green;
- current-source Runtime `text_rich` and rich-table renderer filters pass;
- the exact ignored target-client WGPU exporter passes through the real renderer;
- original-resolution inspection confirms a right-to-left table row flow with top-to-bottom columns, real vertical glyphs, backgrounds, borders, padding, and spans;
- dimensions, byte size, SHA-256, and zero target-copy scan are recorded;
- Text07 remains `in_progress` afterward because table interaction and linked Text03/Text05 vertical parity work remain open.

## Out of Scope

- table cell selection/editing/navigation and accessibility grid semantics;
- authoring logical padding aliases;
- HTML table syntax;
- completing Native/SDF VerticalRl parity owned by Text05;
- Text03's full vertical paragraph indent/alignment matrix outside table flow.
