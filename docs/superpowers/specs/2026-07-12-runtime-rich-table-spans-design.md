---
related_code:
  - zircon_runtime/src/core/framework/render/text/rich.rs
  - zircon_runtime/src/graphics/text/rich/bbcode_table.rs
  - zircon_runtime/src/graphics/text/rich/bbcode_table/attributes.rs
  - zircon_runtime/src/graphics/text/rich/bbcode_table/placement.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/grid.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/text/rich.rs
  - zircon_runtime/src/graphics/text/rich/bbcode_table.rs
  - zircon_runtime/src/graphics/text/rich/bbcode_table/attributes.rs
  - zircon_runtime/src/graphics/text/rich/bbcode_table/placement.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/grid.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs
plan_sources:
  - user: 2026-07-12 select deterministic auto-placement for colspan/rowspan and continue without further confirmation
  - docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/text/rich/tests/table.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_table.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs
doc_type: milestone-detail
---

# Runtime Rich Table Spans Design

## Outcome

Text07 RT-M5 adds deterministic `[cell colspan=N rowspan=N]` behavior to the existing V2 rich-table path. The parser resolves placement once, stores language-neutral grid coordinates and spans in `RichTableCell`, and the UI table owner performs span-aware track sizing and arrangement without understanding BBCode syntax.

This milestone does not claim the remaining Text07 table work: complete cell border/background/padding options, `VerticalRl` tables, or table-specific interaction remain open.

## Fixed product semantics

The user selected deterministic row-major auto-placement:

1. `[cell colspan=N rowspan=N]` is the supported syntax. Attribute names are ASCII case-insensitive through the existing BBCode attribute owner.
2. Missing, malformed, negative, or zero span values resolve to `1`.
3. `colspan` is bounded by the table's validated column count. `rowspan` is bounded to `64` rows so hostile markup cannot create unbounded virtual height.
4. Placement starts from the cursor after the previous cell and skips columns occupied by earlier row spans.
5. At the first free slot, `colspan` is reduced to the contiguous free run before the next occupied slot or table boundary.
6. If the current row contains no free slot, placement continues at column zero of the next row.
7. `expand` and `shrink` on a spanning cell configure every covered column. Existing one-column behavior is unchanged.
8. Parser output stores the resolved row, column, column span, and row span. UI layout does not repeat conflict resolution.

## Alternatives considered

### Selected: parser-resolved placement plus neutral coordinates

The BBCode owner parses attributes and applies the degradation policy. `RichTableCell` then carries `row_index`, `column_index`, `column_span`, and `row_span`. This keeps malformed-markup semantics beside syntax handling, makes serialized/debug output deterministic, and leaves UI responsible only for layout.

### Rejected: UI-only placement from sequential cells

Keeping cells as only a byte range plus requested spans would force every consumer to reproduce the parser's collision policy. It would also make the neutral DTO ambiguous and allow different UI consumers to place the same parsed document differently.

### Rejected: persist a dense occupancy matrix

A dense matrix would make placement explicit, but it duplicates information derivable from resolved cells, scales with rows multiplied by columns, and exposes a parser implementation detail through the framework DTO. A per-column `occupied_until_row` cursor is sufficient during parsing and is not retained.

## Layer ownership

### Neutral framework DTO

`RichTableCell` gains resolved coordinates and spans. Defaults are explicitly `row=0`, `column=0`, `column_span=1`, and `row_span=1`; serde defaults preserve old serialized data. No BBCode attribute names enter the DTO.

### BBCode table owner

`bbcode_table.rs` remains orchestration-only:

- `attributes.rs` validates column configuration and span attributes.
- `placement.rs` owns the row-major cursor and per-column `occupied_until_row` state.
- `ActiveCell` captures resolved placement when opened; closing a cell only attaches its stripped-text byte range.

The placement cursor uses one `u32` occupancy value per table column, so memory remains bounded by the existing 64-column cap regardless of requested row span.

### UI grid and sizing owners

`rich_table.rs` continues to orchestrate block slicing, nested rich layout reuse, and final layout aggregation.

- `grid.rs` projects resolved DTO cells into bounds-checked `PlacedTableCell` values and computes the track count.
- `sizing.rs` owns horizontal and vertical track constraints.

Column sizing runs in this order:

1. Measure every cell without wrapping.
2. Apply one-column preferred widths.
3. Apply spanning-cell deficits so the covered column widths plus internal gaps meet the cell's preferred width.
4. Shrink or expand the resulting tracks against available width using existing column policies.

Row sizing runs after cells are laid out at their final span width:

1. One-row cells establish base row heights.
2. Empty rows receive the existing minimum line-height plus block padding.
3. Row-spanning cells distribute any height deficit evenly across their covered rows.
4. Prefix sums produce stable row origins; prepared cell layouts are translated to those origins without reshaping.

This mirrors Slint's two-phase span-constraint treatment while retaining Godot's rich cell/frame separation and the existing Zircon text-layout reuse.

## Failure and boundary behavior

- Tables with no columns remain guarded as one column by the UI projection, although the parser already guarantees at least one.
- Corrupt or manually constructed DTO coordinates are clamped to existing columns and at least one row/column span. The UI does not allocate from raw row indices beyond the number implied by actual cells.
- A spanning cell may still overflow when fixed non-shrinking columns exceed the frame. The existing table-level `overflow_clipped` signal remains authoritative.
- Nested tables continue to work through byte-range slicing; resolved cell coordinates remain local to their own `RichTable`.
- The renderer receives ordinary resolved text lines. No span-specific renderer path, compatibility shim, or BBCode branch is added.

## Verification contract

Parser tests cover defaults, malformed values, hostile bounds, row-span skip, collision-time colspan reduction, and covered-column configuration.

UI tests cover combined span width, row-span slot reservation, row-height deficit distribution, ordinary row/column compatibility, wrapping, nested rich content, and source-range preservation.

The product exporter must render a real table containing both a merged heading and a row-spanning label through the existing WGPU UI pass. Structural assertions run before renderer creation, and the accepted PNG is written only to `docs/tests/runtime/text`. The repository `target` directory and managed external Cargo targets must contain no same-name PNG.

## Structure budget

New behavior is placed in named leaf owners instead of extending the already multi-responsibility `rich_table.rs` or `bbcode_table.rs`. Production files should remain below the repository 800-line soft limit, tests below the test-file budget, and no root `mod.rs`, compatibility facade, or re-export shim is introduced.
