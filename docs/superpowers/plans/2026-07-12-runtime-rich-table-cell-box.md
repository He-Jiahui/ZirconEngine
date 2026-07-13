# Runtime Rich Table Cell Box Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Godot-aligned rich-table cell padding, alternating backgrounds, and borders, then prove them through the real WGPU product framebuffer.

**Architecture:** BBCode parsing stores a neutral cell box style, rich-table layout resolves content insets and final span frames, and the runtime-interface text layout carries those frames to paint decorations. The screen-space renderer consumes resolved decorations with the existing rectangle and border geometry paths; it does not reconstruct table layout.

**Tech Stack:** Rust, serde DTOs, Zircon Runtime text layout, Zircon Runtime Interface UI render contracts, WGPU screen-space UI renderer, Windows coordinator-managed Cargo validation.

---

## Milestone RT-M6: Rich-table cell box model

### S0: Test contracts before production behavior

**Files:**

- Modify: `zircon_runtime/src/graphics/text/rich/tests/table.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/tests/rich_table.rs`
- Modify: `zircon_runtime_interface/src/tests/render_contracts.rs`
- Create: `zircon_runtime_interface/src/tests/render_contracts/rich_table.rs`
- Create: `zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_table.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs`

- [x] Add parser tests that express the desired DTO before implementation:

```rust
#[test]
fn text_rich_bbcode_table_cell_box_options_parse_without_renderer_semantics() {
    let parsed = parse_rich_text(
        "[table=2][cell border=#73D7FF bg=#12202C,#182F3D padding=8,4,12,6]A[/cell][/table]",
        RichTextFormat::BbCode,
    );
    let cell = &parsed.tables[0].cells[0];
    assert_eq!(cell.box_style.padding.unwrap().left, 8.0);
    assert!(cell.box_style.border_color.is_some());
    assert_ne!(cell.box_style.odd_row_background, cell.box_style.even_row_background);
}
```

- [x] Add hostile-input parser coverage: an invalid tuple falls back as a whole, negative sides clamp to zero, finite sides cap at `4096.0`, invalid colors remain unset, and a single `bg` color populates both parities.
- [x] Add layout tests proving authored padding changes content origin/measure and a styled colspan/rowspan cell emits one final box covering every resolved track.
- [x] Add interface render-contract coverage proving resolved boxes become `TableCellBackground` and `TableCellBorder` decorations with exact frame/range/color and 1px thickness.
- [x] Add renderer planning coverage proving a background contributes a filled rectangle before text and a border contributes outline geometry after text.
- [x] Do not run Cargo yet; the expected pre-implementation state is compile failure because the new DTO/types do not exist.
- [x] Append one S0 row to `docs/plans/zircon_runtime/text/07/2026-07-11-rich-text-html-bbcode-output-records.md` after the test contracts are complete.

### S1: Neutral cell-style DTO and BBCode decoding

**Files:**

- Modify: `zircon_runtime/src/core/framework/render/text/rich.rs`
- Modify: `zircon_runtime/src/graphics/text/rich/bbcode_table/attributes.rs`
- Modify: `zircon_runtime/src/graphics/text/rich/bbcode_table.rs`

- [x] Add explicit serializable box types; keep missing padding distinct from authored zero padding:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichTableCellPadding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichTableCellBoxStyle {
    pub padding: Option<RichTableCellPadding>,
    pub odd_row_background: Option<Vec4>,
    pub even_row_background: Option<Vec4>,
    pub border_color: Option<Vec4>,
}
```

- [x] Add `#[serde(default)] pub box_style: RichTableCellBoxStyle` to `RichTableCell` and keep its manual `Default` compatible.
- [x] Extend `CellAttributes` to carry `box_style`; parse `border`, `bg`, and `padding` using the existing hexadecimal color decoder and constants `MAX_CELL_PADDING_PX = 4096.0` and `CELL_PADDING_COMPONENT_COUNT = 4`.
- [x] Apply the parsed style when `RichTableCell` is created. Do not add aliases, named colors, border widths, or renderer types to the parser.
- [x] Append one S1 row to the Text07 child output record.

### S2: Resolved text-box transport and paint projection

**Files:**

- Modify: `zircon_runtime_interface/src/ui/surface/render/text_layout.rs`
- Modify: `zircon_runtime_interface/src/ui/surface/render/text_shape.rs`
- Modify: `zircon_runtime_interface/src/ui/surface/render/command.rs`
- Modify: `zircon_runtime_interface/src/ui/surface/render/mod.rs`
- Modify: `zircon_runtime_interface/src/ui/surface/mod.rs`
- Modify every current `UiResolvedTextLayout` literal reported by `git grep -n 'UiResolvedTextLayout {' -- '*.rs'` to initialize `boxes: Vec::new()` without changing unrelated fields.

- [x] Add the resolved, renderer-neutral transport:

```rust
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextBox {
    pub range: UiTextRange,
    pub frame: UiFrame,
    pub background_color: Option<UiRgbaColor>,
    pub border_color: Option<UiRgbaColor>,
    pub border_width: f32,
}
```

- [x] Add `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub boxes: Vec<UiResolvedTextBox>` to `UiResolvedTextLayout` and export the type through the existing surface render facade.
- [x] Extend `UiTextPaintDecorationKind` with `TableCellBackground` and `TableCellBorder`; add a serde-defaulted `thickness: f32` to `UiTextPaintDecoration`, preserving existing selection/caret constructors.
- [x] In `UiRenderCommand::text_paint()`, append background and border decorations from resolved boxes. Convert `UiRgbaColor::to_u8()` deterministically to `#RRGGBBAA`; do not reparse BBCode or change editable decoration generation.
- [x] Append one S2 row to the Text07 child output record.

### S3: Padding-aware measure/arrange and final span boxes

**Files:**

- Modify: `zircon_runtime/src/ui/text/layout_engine.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table/grid.rs` only if the placed-cell projection needs a focused accessor.

- [x] Initialize `boxes: Vec::new()` in ordinary horizontal and vertical layout constructors.
- [x] Replace global-only cell padding use with a per-cell helper:

```rust
struct ResolvedCellPadding {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

fn resolved_cell_padding(cell: &RichTableCell, font_size: f32) -> ResolvedCellPadding {
    // authored logical pixels or the existing 0.35em/0.2em fallback
}
```

- [x] Include left/right padding in preferred width and top/bottom padding in height constraints. Store each prepared cell's provisional content Y so final translation uses its own top inset.
- [x] After column widths and row heights are solved, emit one `UiResolvedTextBox` for every cell with a background or border. Frame width includes every spanned column and internal gap; frame height includes every spanned row.
- [x] Merge boxes from each top-level table into the final rich-text layout while preserving absolute frames and clipping them to the text clip frame.
- [x] Select odd/even background by the resolved start row (`row % 2 == 0` uses odd-row color, matching the Godot contract).
- [x] Append one S3 row to the Text07 child output record.

### S4: Screen-space rendering and product proof

**Files:**

- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`
- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs`
- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs`

- [x] Update decoration ordering so `Selection` and `TableCellBackground` paint before glyphs, while caret/composition/outline and `TableCellBorder` paint after glyphs.
- [x] Branch `TableCellBorder` to `push_border(vertices, frame, thickness, color, viewport)`; every other decoration continues through `push_rect`.
- [x] Update the real rich-table product sample to show:
  - distinct odd/even backgrounds;
  - visible cyan/amber border colors;
  - asymmetric authored padding;
  - existing colspan, rowspan, wrapped cell, multilingual text, and inline icon.
- [x] Strengthen proof assertions with resolved decoration count/kinds and non-transparent cell-box colors. Do not replace the final framebuffer pixel/readback assertions.
- [x] Set the accepted output path to `docs/tests/runtime/text/runtime_text_multilingual_rich_table_cell_box_product_framebuffer_20260712.png`; never write proof images below any `target` directory.
- [x] Append one S4 row to the Text07 child output record.

### RT-M6-T: Testing, correction, documentation, and acceptance

**Files:**

- Modify: `docs/zircon_runtime/graphics/text.md`
- Modify: `docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
- Modify: `docs/plans/zircon_runtime/text/07/2026-07-11-rich-text-html-bbcode-output-records.md`
- Modify: `.codex/sessions/20260711-1900-runtime-text-goal.md`

- [x] Applied `prefer-windows-validation`; every Cargo run used coordinator-managed targets under `D:\cargo-targets` or `E:\cargo-targets`. Repository-local Cargo output was not used.
- [x] Attempted workspace-wide `cargo fmt --all -- --check`; the shared checkout timed out, then exact leased Rust files passed `rustfmt --check` and scoped `git diff --check`.
- [x] Locked Windows production validation passed for `zircon_runtime_interface`; `zircon_runtime` production build passed before the final shared-layer correction and a current-source locked lib check passed afterward.
- [x] Current-source Runtime lib-test `text_rich` passed 68/68 with 7,695 filtered; the exact renderer cell-box test passed 1/1 with 7,762 filtered.
- [x] Exact ignored WGPU product framebuffer test passed 1/1 in 125.10s under GPU job `bdb1cd9c78c44822af5c98f21cdcb2ad`.
- [x] Inspected the PNG at original resolution: 1080x1450, 242,151 bytes, SHA-256 `CAE02C87B1AD0196C426E880B925B63B22C89F907785E7BFDF72D1A433233B18`; all approved target roots contain zero same-name copies.
- [x] Diagnosed the first product failure bottom-up: owner-table ranges were recursively re-laid out inside a single cell, and the authored top padding made text baselines intentionally differ. Fixed table-depth slicing and changed only the geometry proof to compare final cell boxes; framebuffer/readback gates stayed intact.
- [x] Updated module docs, Text07 status, one RT-M6-T evidence row, milestone acceptance, and the active session note.
- [x] Kept Text07 and the Goal `in_progress` because VerticalRl tables, table interaction, Text03 vertical block gaps, and Text05 Native/SDF vertical parity remain open.

## 状态与产出记录

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| RT-M6 | Cell padding/background/border | accepted | 2026-07-12 | Runtime Interface passed; Runtime `text_rich` 68/68; renderer exact 1/1; real WGPU 1/1; accepted PNG 1080x1450, SHA-256 `CAE02C87B1AD0196C426E880B925B63B22C89F907785E7BFDF72D1A433233B18`; target copies 0. |
