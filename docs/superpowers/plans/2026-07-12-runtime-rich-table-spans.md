---
related_code:
  - zircon_runtime/src/text/model/rich.rs
  - zircon_runtime/src/text/rich/bbcode_table.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
implementation_files:
  - zircon_runtime/src/text/model/rich.rs
  - zircon_runtime/src/text/rich/bbcode_table.rs
  - zircon_runtime/src/text/rich/bbcode_table/attributes.rs
  - zircon_runtime/src/text/rich/bbcode_table/placement.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/grid.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs
plan_sources:
  - docs/superpowers/specs/2026-07-12-runtime-rich-table-spans-design.md
  - docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/text/rich/tests/table.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_table.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs
doc_type: milestone-detail
---

# Runtime Rich Table Spans Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete Text07 RT-M5 deterministic `colspan`/`rowspan` parsing, span-aware rich-table layout, and real WGPU framebuffer acceptance.

**Architecture:** The BBCode layer resolves row-major placement once and writes neutral coordinates/spans into `RichTableCell`. Folder-backed parser and UI leaf owners isolate attribute validation, placement, grid projection, and track sizing; the existing rich layout and renderer paths are reused unchanged for cell contents.

**Tech Stack:** Rust, serde, Zircon runtime text DTOs, existing rich-text parser/layout providers, Windows-native Cargo validation, WGPU framebuffer capture.

---

## Current baseline

- RT-M4 first slice already parses `[table]`/`[cell]`, lays out one cell per track, passes current-source `text_rich` 56/56, and has a real WGPU product screenshot.
- `RichTableCell` currently stores only a byte range, so spans cannot be represented without ambiguity.
- `bbcode_table.rs` uses `cells.len() % columns.len()` and cannot skip row-span occupancy.
- `rich_table.rs` chunks sequential cells by column count and owns both orchestration and track sizing; adding span behavior inline would violate the structure-convention priority.
- Global coordinator baseline is degraded by foreign Cargo jobs, but no active lease or Failure graph entry overlaps Text07.

## Milestone RT-M5: deterministic rich-table spans

- **Goal:** `[cell colspan=N rowspan=N]` produces deterministic resolved grid cells and lays them out through the normal rich-text/WGPU path.
- **In-scope behaviors:** default spans, invalid/zero/negative degradation, hostile bounds, row-span occupancy skip, collision-time colspan reduction, covered-column policy, span-aware preferred widths, wrapping at final span width, row-height deficit distribution, nested rich content, real WGPU pixels.
- **Dependencies:** RT-M4 neutral table DTO/parser/layout, Text02 shaping/BiDi, Text03 wrapping/block layout, Text05 renderer/atlas, existing UI image/icon consumers.
- **Out of scope:** cell border/background/padding option set, `VerticalRl` tables, table-specific interaction.
- **Promotion gate:** every listed parser/UI boundary test passes; production compile passes; exact product exporter produces a reviewed PNG under `docs/tests/runtime/text`; no same-name PNG exists under any target directory; docs/status/output records match the current behavior.

### Implementation slice RT-M5-S1: neutral DTO and parser placement

**Files:**

- Modify: `zircon_runtime/src/text/model/rich.rs`
- Modify: `zircon_runtime/src/text/rich/bbcode_table.rs`
- Create: `zircon_runtime/src/text/rich/bbcode_table/attributes.rs`
- Create: `zircon_runtime/src/text/rich/bbcode_table/placement.rs`
- Modify: `zircon_runtime/src/text/rich/tests/table.rs`

- [x] Add failing parser-level tests before production changes. Tests assert exact `(row_index, column_index, column_span, row_span)` values for:

```rust
fn cell_grid(table: &RichTable) -> Vec<(u32, u16, u16, u16)> {
    table
        .cells
        .iter()
        .map(|cell| {
            (
                cell.row_index,
                cell.column_index,
                cell.column_span,
                cell.row_span,
            )
        })
        .collect()
}

// Row 0: A occupies col 0 for two rows; B is clamped to cols 1..3.
// Row 1: C skips occupied col 0 and lands at col 1.
let parsed = parse_rich_text(
    "[table=3][cell rowspan=2]A[/cell][cell colspan=9]B[/cell][cell]C[/cell][/table]",
    RichTextFormat::BbCode,
);
assert_eq!(cell_grid(&parsed.tables[0]), vec![(0, 0, 1, 2), (0, 1, 2, 1), (1, 1, 1, 1)]);
```

```rust
// The first free slot has only one contiguous column before an occupied slot.
let parsed = parse_rich_text(
    "[table=3][cell]A[/cell][cell rowspan=2]B[/cell][cell]C[/cell][cell colspan=2]D[/cell][cell]E[/cell][/table]",
    RichTextFormat::BbCode,
);
assert_eq!(cell_grid(&parsed.tables[0])[3], (1, 0, 1, 1));
assert_eq!(cell_grid(&parsed.tables[0])[4], (1, 2, 1, 1));
```

- [x] Extend `RichTableCell` with explicit defaults and serde-safe fields:

```rust
pub struct RichTableCell {
    pub byte_range: (u32, u32),
    pub row_index: u32,
    pub column_index: u16,
    pub column_span: u16,
    pub row_span: u16,
}

impl Default for RichTableCell {
    fn default() -> Self {
        Self {
            byte_range: (0, 0),
            row_index: 0,
            column_index: 0,
            column_span: 1,
            row_span: 1,
        }
    }
}
```

- [x] Implement `attributes.rs` with `MAX_TABLE_ROW_SPAN = 64`, `CellAttributes`, `parse_cell_attributes`, and covered-column configuration. Malformed or zero spans return one; column spans clamp to the validated column count; row spans clamp to 64.

- [x] Implement `placement.rs` with a cursor that stores only `row`, `column`, and `occupied_until_row: Vec<u32>`. Its public-to-parent operation returns:

```rust
pub(super) struct CellPlacement {
    pub row_index: u32,
    pub column_index: u16,
    pub column_span: u16,
    pub row_span: u16,
}

pub(super) fn place(&mut self, requested_column_span: u16, row_span: u16) -> CellPlacement;
```

The algorithm skips occupied columns, selects the first free slot, reduces the column span to that slot's contiguous free run, marks every covered column occupied until `row + row_span`, and advances the cursor without allocating rows.

- [x] Refactor `bbcode_table.rs` into orchestration: parse attributes, obtain a placement on cell open, configure every covered column, retain placement in `ActiveCell`, and attach the byte range on close. The `cells.len() % columns.len()` behavior was removed without a compatibility branch.

- [x] Update the Text07 output record immediately after the slice with exact changed files and named tests; compile/tests remain pending until RT-M5-T.

### Implementation slice RT-M5-S2: span-aware UI grid and track sizing

**Files:**

- Modify: `zircon_runtime/src/ui/text/layout_engine/rich_table.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/rich_table/grid.rs`
- Create: `zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs`
- Modify: `zircon_runtime/src/ui/text/layout_engine/tests/rich_table.rs`

- [x] Add UI tests before production changes for a merged header, a reserved row-span slot, a collision-reduced cell, a tall row-spanning cell that pushes the next row below its content, and unchanged ordinary two-column behavior.

- [x] Implement `grid.rs` as a bounds-checked DTO projection:

```rust
pub(super) struct PlacedTableCell<'a> {
    pub cell: &'a RichTableCell,
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
}

pub(super) struct TableGrid<'a> {
    pub column_count: usize,
    pub row_count: usize,
    pub cells: Vec<PlacedTableCell<'a>>,
}
```

Clamp columns and spans to the table's available tracks, preserve parser order, and compute `row_count` from actual resolved cells.

- [x] Implement `sizing.rs` with two explicit phases:

```rust
pub(super) struct PreferredCellWidth {
    pub column: usize,
    pub column_span: usize,
    pub width: f32,
}

pub(super) struct CellHeightConstraint {
    pub row: usize,
    pub row_span: usize,
    pub height: f32,
}

pub(super) fn resolve_column_widths(
    columns: &[RichTableColumn],
    preferred_cells: &[PreferredCellWidth],
    available_width: f32,
    gap: f32,
    minimum: f32,
) -> Vec<f32>;

pub(super) fn resolve_row_heights(
    row_count: usize,
    cells: &[CellHeightConstraint],
    minimum: f32,
) -> Vec<f32>;
```

Apply single-track constraints before spanning constraints. Distribute horizontal span deficits by covered expand ratios when present, otherwise evenly. Establish one-row heights before distributing row-span deficits evenly.

- [x] Refactor `rich_table.rs` to:

  1. build `TableGrid`;
  2. measure preferred cell widths without wrapping;
  3. solve final column widths;
  4. lay out each cell once at its final span width in a neutral vertical origin;
  5. solve row heights;
  6. translate resolved line frames to final row origins;
  7. aggregate overflow and source ranges.

Keep block slicing and nested-table reuse in the existing root owner. Remove the old `cells.chunks(column_count)`, `index % preferred.len()`, and inline width-solver implementations.

- [x] Update the Text07 output record immediately after the slice, recording file sizes and the exact new behavior tests; RT-M5-T remains pending.

### Implementation slice RT-M5-S3: product framebuffer proof

**Files:**

- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs`
- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs`
- Modify: `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_commands.rs` only if viewport/command construction needs a focused helper
- Create after successful execution: `docs/tests/runtime/text/runtime_text_multilingual_rich_table_spans_product_framebuffer_20260712.png`

- [x] Replace the table sample with real visible span content while retaining ordinary wrapping and inline-icon coverage:

```text
[table=3]
  [cell colspan=3][b]BBCode V2 MERGED TABLE[/b][/cell]
  [cell rowspan=2][color=#64d8ff]Span owner[/color][/cell]
  [cell colspan=2]colspan shares measured width[/cell]
  [cell]wrapped detail alpha beta gamma delta[/cell]
  [cell][icon=★|Microsoft YaHei UI] real WGPU frame[/cell]
[/table]
```

- [x] Add pre-render structural assertions that prove the heading begins at the first column and spans beyond the third-column origin, the row-spanning label shares the first detail-row y but no later cell occupies its x/y slot, the colspan detail uses the combined two-column width, the following row advances below it, wrapping occurs, and the inline icon placeholder remains present.

- [x] Change `proof_path()` to the new canonical docs filename. The PNG is not produced until RT-M5-T and must not be retained in repository or external target directories.

- [x] Update the Text07 output record after the product-test source slice; the screenshot remains pending until RT-M5-T executes the ignored exact exporter.

### RT-M5-T testing stage

- [x] Run formatting and static structure gates on every touched Rust file:

```powershell
cargo fmt --all -- --check
git diff --check -- <all RT-M5 attributed paths>
```

Expected: exit `0`; no conflict markers, trailing whitespace, or production/test file budget regression.

- [x] Use the Windows-native managed validator or coordinator-managed Cargo lane with `--locked`. The crate-local production build passed, the current-source `text_rich` binary passed 64/64, and the exact target-client product exporter passed 1/1. No repository-local target directory was used.

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipTest
cargo test -p zircon_runtime text_rich --locked
cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer export_runtime_multilingual_text_product_framebuffer_png --locked -- --ignored --exact --nocapture
```

Expected: production compile succeeds; all current-source `text_rich` tests pass; ignored exact exporter passes `1/1` and writes the new PNG only under `docs/tests/runtime/text`.

- [x] Product assertion failures were corrected in the product sample/assertion owners after parser/UI 64/64 evidence; no renderer-only span bypass was added.

- [x] Inspect the PNG at original resolution. The merged heading, row-spanning label, combined-width cell, wrapped detail, and real icon glyph are visible. Dimensions, bytes, SHA256, elapsed time, and job identity are recorded.

- [x] Scan the repository and approved external targets for the exact PNG filename. The docs copy count is one, repository `target` is absent, and all nine approved external roots have zero copies.

### Implementation slice RT-M5-S4: synchronized docs and milestone status

**Files:**

- Modify: `docs/zircon_runtime/graphics/text.md`
- Modify: `docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
- Modify: `docs/plans/zircon_runtime/text/07/2026-07-11-rich-text-html-bbcode-output-records.md`
- Modify: `.codex/sessions/20260711-1900-runtime-text-goal.md`

- [x] Update the module document's `related_code`, `implementation_files`, `plan_sources`, and `tests` metadata for the new leaf owners. Current parser-placement, track-sizing, test, and screenshot evidence replaces the open-span statement.

- [x] Update Text07 status to mark `colspan/rowspan` complete after RT-M5-T. Border/background/padding options, `VerticalRl` tables, table interaction, Text03 vertical block gaps, and Text05 Native/SDF parity remain explicitly open.

- [x] Add one RT-M5-T acceptance row with exact commands, counts, PNG facts, hash, and target scan. The session note is updated in this slice.

- [x] Re-run scoped `git diff --check`, inspect `git status --short` for attributed paths, and verify no compatibility shim or stale no-span path remains. Final checks pass; only expected attributed tracked/untracked paths remain.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| RT-M5 | S0 设计与计划 | completed | 2026-07-12 | `docs/superpowers/specs/2026-07-12-runtime-rich-table-spans-design.md`; 本实施计划；用户选择确定性自动排布 |
| RT-M5 | S1 DTO 与 parser placement | completed | 2026-07-12 | `RichTableCell` resolved grid fields；`bbcode_table/{attributes,placement}.rs`；4 个新增 parser span tests；最终生产 186/134/85/82 行，test 144 行；RT-M5-T green |
| RT-M5 | S2 UI grid 与 track sizing | completed | 2026-07-12 | `rich_table/{grid,sizing}.rs`；4 个新增 UI span tests；最终 root/grid/sizing/test 470/60/175/209 行；旧 chunks/modulo/inline solver 删除；RT-M5-T green |
| RT-M5 | S3 产品 framebuffer source | completed | 2026-07-12 | 3-column merged heading + rowspan owner + colspan detail + wrapped detail + real icon；pre-render geometry assertions；exact WGPU 1/1 accepted |
| RT-M5 | RT-M5-T testing | completed | 2026-07-12 | managed production build；`text_rich` 64/64；exact WGPU 1/1，116.88s，job `15dbf21801754a618cfc0e2354c0ebd5`；PNG SHA256 `47A5C96C...D621F43`；target copies 0 |
| RT-M5 | S4 docs/status/closeout | completed; atomic commit preflight pending | 2026-07-12 | module doc metadata/behavior、Text07 current status、RT-M5 output/acceptance row、session note、final fmt/diff/status/compat scan |
