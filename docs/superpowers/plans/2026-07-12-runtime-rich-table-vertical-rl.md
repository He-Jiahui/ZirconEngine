# Runtime Rich Table VerticalRl Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add true logical-axis VerticalRl rich-table layout with spans, physical cell padding, resolved backgrounds/borders, surrounding block flow, and a real WGPU product framebuffer.

**Architecture:** Parser row/column coordinates remain neutral. A shared table-axis mapper converts logical column/row tracks to physical HorizontalTb or VerticalRl frames; cell layout continues to recurse through the existing rich/vertical shaping owners, and the renderer consumes only resolved physical boxes. The 606-line root table owner is hard-split into folder-backed axis, layout, cell, sizing, and source-slice responsibilities before new behavior lands.

**Tech Stack:** Rust, Zircon Runtime rich-text layout, vertical shaped-run providers, Runtime Interface resolved text boxes, WGPU screen-space renderer, Windows coordinator-managed Cargo validation.

---

## Milestone RT-M7: VerticalRl rich-table logical axes

### S0: Structure-first test contracts

**Files:**

- Delete: `zircon_runtime/src/ui/text/layout_engine/tests/rich_table.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/tests/rich_table/mod.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/tests/rich_table/horizontal.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/tests/rich_table/vertical_rl.rs`

- [x] Move all ten accepted RT-M4–RT-M6 horizontal tests without changing assertions into `horizontal.rs`; keep `mod.rs` limited to shared imports and `find_line`.
- [x] Add `vertical_rl.rs` contracts before production behavior:

```rust
#[test]
fn text_rich_bbcode_vertical_table_maps_columns_down_and_rows_left() {
    let layout = vertical_table("[table=2][cell]甲[/cell][cell]乙[/cell][cell]丙[/cell][cell]丁[/cell][/table]");
    let a = find_line(&layout, "甲");
    let b = find_line(&layout, "乙");
    let c = find_line(&layout, "丙");
    let d = find_line(&layout, "丁");
    assert_eq!(a.frame.x, b.frame.x);
    assert_eq!(c.frame.x, d.frame.x);
    assert!(a.frame.x > c.frame.x);
    assert!(b.frame.y > a.frame.y);
    assert!(d.frame.y > c.frame.y);
}
```

- [x] Add separate tests for vertical colspan/rowspan physical extents, physical top/right padding, and surrounding `before/table/after` right-to-left block order.
- [x] Do not run Cargo during S0; the current `VerticalRl => None` table guard makes these contracts fail in the RT-M7 testing stage.
- [x] Append exactly one RT-M7 S0 row to `docs/plans/zircon_runtime/text/07/2026-07-11-rich-text-html-bbcode-output-records.md`.

### S1: Hard-split the rich-table owner and add logical axes

**Files:**

- Rewrite: `zircon_runtime/src/ui/text/layout_engine/rich_table.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/rich_table/axes.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/rich_table/source_slice.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs`
- Keep: `zircon_runtime/src/ui/text/layout_engine/rich_table/grid.rs`

- [x] Reduce `rich_table.rs` to child declarations and the single `pub(super) use layout::layout_rich_tables_with_provider` export.
- [x] Move source slicing, nested table depth descent, and source-range shifting unchanged into `source_slice.rs`.
- [x] Replace width/height-specific internal names in `sizing.rs` with hard-cut axis-neutral names:

```rust
pub(super) struct PreferredColumnExtent {
    pub column: usize,
    pub column_span: usize,
    pub extent: f32,
}

pub(super) struct RowExtentConstraint {
    pub row: usize,
    pub row_span: usize,
    pub extent: f32,
}

pub(super) fn resolve_column_extents(
    columns: &[RichTableColumn],
    preferred_cells: &[PreferredColumnExtent],
    available_track_extent: f32,
    gap: f32,
    minimum: f32,
) -> Vec<f32>;

pub(super) fn resolve_row_extents(
    row_count: usize,
    cells: &[RowExtentConstraint],
    minimum: f32,
) -> Vec<f32>;
```

- [x] Add `TableAxes` in `axes.rs` with the only logical-to-physical mapping:

```rust
pub(super) enum TableAxes {
    HorizontalTb,
    VerticalRl,
}

impl TableAxes {
    pub fn from_style(style: &UiResolvedStyle) -> Self;
    pub fn inline_extent(self, frame: UiFrame) -> f32;
    pub fn block_extent(self, frame: UiFrame) -> f32;
    pub fn physical_frame(
        self,
        container: UiFrame,
        inline_start: f32,
        block_start: f32,
        inline_extent: f32,
        block_extent: f32,
    ) -> UiFrame;
}
```

- [x] Horizontal mapping is `(x + inline_start, y + block_start, inline_extent, block_extent)`; VerticalRl mapping is `(right - block_start - block_extent, y + inline_start, block_extent, inline_extent)`.
- [x] Keep the old horizontal implementation deleted after extraction; do not retain a compatibility module or per-mode solver.
- [x] Append exactly one RT-M7 S1 output row after the structure/axis slice is complete.

### S2: Shared measure/arrange, spans, padding, and block flow

**Files:**

- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table/axes.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs`

- [x] Remove the existing `VerticalRl => None` guard.
- [x] In `cell_layout.rs`, resolve physical padding once and expose logical column/row padding sums:

```rust
impl ResolvedCellPadding {
    fn column_sum(self, axes: TableAxes) -> f32 {
        match axes {
            TableAxes::HorizontalTb => self.left + self.right,
            TableAxes::VerticalRl => self.top + self.bottom,
        }
    }

    fn row_sum(self, axes: TableAxes) -> f32 {
        match axes {
            TableAxes::HorizontalTb => self.top + self.bottom,
            TableAxes::VerticalRl => self.left + self.right,
        }
    }
}
```

- [x] Measure preferred logical column extents with `wrap=None`: Horizontal uses resolved layout width, VerticalRl uses resolved layout height.
- [x] Lay out each prepared cell inside its final logical column span. Horizontal gives the content an unconstrained physical height; VerticalRl gives it a bounded physical width derived from source length × line advance and clamped to the shared unconstrained ceiling.
- [x] Resolve row extents from prepared cell block extents plus physical block padding. Arrange final cell frames with `TableAxes::physical_frame`.
- [x] Translate prepared layouts by top-left anchor in HorizontalTb and top-right anchor in VerticalRl; translate nested lines and boxes together, then apply the final clip.
- [x] Emit one physical `UiResolvedTextBox` per styled cell. Vertical `colspan` changes box height and vertical `rowspan` changes box width toward the left.
- [x] Make source-order orchestration in `layout.rs` consume physical height for HorizontalTb and physical width for VerticalRl, so surrounding blocks flow down or right-to-left respectively.
- [x] Preserve table-depth recursion, parser bounds, odd/even row background selection, and the existing renderer-neutral box contract.
- [x] Append exactly one RT-M7 S2 output row.

### S3: Real product framebuffer source and pre-render gates

**Files:**

- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs`
- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_commands.rs`
- Create: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_commands/rich_table.rs`
- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs`

- [x] Extract the accepted horizontal RT-M6 markup from the 794-line product root into `proof_commands/rich_table.rs`; add `proof_horizontal_rich_table()` and `proof_vertical_rich_table()` so the product root remains below the 800-line review budget.
- [x] Place the new VerticalRl table in `UiFrame::new(690.0, 1120.0, 348.0, 300.0)` beside the accepted horizontal table. Use styled CJK/Latin cell text, colspan, rowspan, physical padding, alternating backgrounds, and cyan/amber borders.
- [x] Set `UiRichTextFormat::BbCode`, `UiTextWritingMode::VerticalRl`, `UiTextRenderMode::Sdf`, a real CJK system family, glyph wrapping, and an explicit resolved layout.
- [x] Add a pre-render gate that requires:

  - `writing_mode == VerticalRl`;
  - the expected resolved box count and matching background/border decoration counts;
  - same-row boxes share physical x and advance down by column index;
  - later row boxes move left;
  - colspan increases physical height and rowspan increases physical width;
  - first content column respects physical top/right padding;
  - at least one Latin run remains present so the accepted vertical-orientation renderer path is exercised.

- [x] Keep the existing per-command changed-pixel gate, background-only comparison, vertical column pixel checks, and real WGPU readback.
- [x] Change the canonical proof path to `docs/tests/runtime/text/runtime_text_multilingual_rich_table_vertical_rl_product_framebuffer_20260712.png`; never write a proof below any target directory.
- [x] Append exactly one RT-M7 S3 output row, explicitly marking WGPU pending until RT-M7-T.

### RT-M7-T: Windows testing, correction, screenshot acceptance, and docs

**Files:**

- Modify: `docs/zircon_runtime/graphics/text.md`
- Modify: `docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
- Modify: `docs/plans/zircon_runtime/text/07/2026-07-11-rich-text-html-bbcode-output-records.md`
- Modify: `.codex/sessions/20260711-1900-runtime-text-goal.md`

- [x] Apply `prefer-windows-validation`; use only coordinator-managed D/E/F target roots and never repository-local Cargo output.
- [x] Run exact-file rustfmt, scoped `git diff --check`, conflict-marker, trailing-whitespace, compatibility-path, and file-budget scans.
- [x] Run locked current-source production validation for `zircon_runtime_interface` and `zircon_runtime`.
- [x] Run the Runtime `text_rich` filter and the exact rich-table renderer filter; expected focused total is the existing 68 tests plus the four RT-M7 layout contracts. (Current-source rebuild is externally E0283-blocked; the pre-existing artifact supplied 71/72 diagnostic evidence, not a false green.)
- [x] Run the exact ignored target-client WGPU product test with `--locked --offline --jobs 1 -- --ignored --exact --nocapture --test-threads=1` through a coordinator GPU lane.
- [x] Diagnose failures bottom-up from axis mapping, track sizing, cell layout, resolved boxes, paint projection, then renderer. Do not weaken geometry, decoration, changed-pixel, or readback gates.
- [x] Inspect the accepted PNG at original resolution and record width, height, byte size, SHA-256, visible VerticalRl table evidence, and zero same-name copies across all nine approved target roots plus repository `target`.
- [x] Update module metadata and the detailed RT-M7 section, update Text07's concise current status, append exactly one RT-M7-T evidence row, and refresh the active session note.
- [x] Keep Text07 and the Goal `in_progress` because table interaction, Text03 vertical paragraph gaps, and Text05 Native/SDF vertical parity remain open.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| RT-M7 | S0 structure-first test contracts | completed-testing-stage-pending | 2026-07-12 | Existing 10 horizontal regressions moved unchanged to `tests/rich_table/horizontal.rs`; four VerticalRl logical-axis contracts added in `vertical_rl.rs`; navigational `mod.rs` is 29 lines. Exact-file rustfmt and scoped diff-check pass; Cargo is intentionally deferred to RT-M7-T. |
| RT-M7 | S1 hard split and logical axes | completed-testing-stage-pending | 2026-07-12 | The 606-line root is now an 8-line navigation owner backed by `axes`, `layout`, `cell_layout`, `grid`, `sizing`, and `source_slice`; old width/height solver names and monolithic implementation were deleted. Exact-file rustfmt and scoped diff-check pass; Cargo remains deferred to RT-M7-T. |
| RT-M7 | S2 shared VerticalRl measure/arrange | completed-testing-stage-pending | 2026-07-12 | One shared logical-axis pipeline now measures columns, resolves row extents, maps spans to physical frames, preserves physical padding, translates nested lines/boxes, and flows surrounding blocks down or right-to-left. The old VerticalRl bypass is deleted; exact-file rustfmt and scoped diff-check pass, with Cargo deferred to RT-M7-T. |
| RT-M7 | S3 real product source and proof gates | completed-wgpu-pending | 2026-07-12 | The accepted horizontal sample moved into a 39-line table proof owner and the 791-line product root remains below budget. A styled CJK/Latin SDF VerticalRl table now occupies the right product panel; pre-render gates cover five boxes/decorations, logical-axis ordering, spans, physical top/right padding, and retained Latin runs. Canonical output points only to `docs/tests/runtime/text/runtime_text_multilingual_rich_table_vertical_rl_product_framebuffer_20260712.png`; WGPU is explicitly pending RT-M7-T. |
| RT-M7 | RT-M7-T Windows and WGPU acceptance | completed-focused-lib-rebuild-external-blocker-recorded | 2026-07-12 | Current-source lib check and product no-run passed; current-source lib tests are externally E0283-blocked in advanced lighting, while an old artifact's 71/72 run exposed and drove the final anchor fix. Exact WGPU passed 1/1 in 197.86s. Accepted PNG is 1080×1450, 257649 bytes, SHA256 `82EC5035EDB80AC4F6D894C9A1A000279F23B75B95D4FE1881B0AC70655813DE`, with zero target copies and original-resolution VerticalRl evidence. |
