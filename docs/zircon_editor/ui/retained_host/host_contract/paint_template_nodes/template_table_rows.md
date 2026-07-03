---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/allocation.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/tests.rs
  - zircon_editor/src/ui/retained_host/ui/template_layout_context.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - rustfmt --edition 2021 --check --config skip_children=true -- zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs (2026-07-03: passed)
  - cargo test -p zircon_editor --lib archived_asset_table --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-table-text-0703 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03: cargo wrapper timed out during Windows lib-test compilation/linking; not counted as wrapper green)
  - D:\cargo-targets\zircon-editor-table-text-0703\debug\deps\zircon_editor-e4a53279ee4b776d.exe archived_asset_table --nocapture --test-threads=1 (2026-07-03: passed, 4 passed)
  - D:\cargo-targets\zircon-editor-table-text-0703\debug\deps\zircon_editor-e4a53279ee4b776d.exe declared_table_cells_use_the_same_display_normalization --nocapture --test-threads=1 (2026-07-03: passed, 1 passed)
  - D:\cargo-targets\zircon-editor-table-text-0703\debug\deps\zircon_editor-e4a53279ee4b776d.exe capture_m3_gui_acceptance_visual_artifacts --ignored --nocapture --test-threads=1 (2026-07-03: passed, refreshed docs/tests/editor)
  - cargo fmt -p zircon_editor --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib table_columns_drop_numeric_cells_for_narrow_layout_context --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib table_header_and_tail_use_recessed_table_surface --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: red then passed)
  - cargo test -p zircon_editor --lib workbench_table_row --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 8 passed)
  - cargo test -p zircon_editor --lib template_table_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 17 passed)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never (2026-06-26: passed)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-26: passed, 1 passed; refreshed docs/tests/editor)
doc_type: module-detail
---

# Template Table Rows

`template_table_rows.rs` is the retained-host Workbench table-row painter entry. It keeps the external dispatcher thin: recognize table-like template nodes, collect cell text, then delegate surface, cell text, and action glyph commands to child owners.

## Ownership

- `identity.rs` owns table-family recognition plus Workbench header/tail/selected control-id checks.
- `cells/text.rs` owns declared option cells and legacy text splitting.
- `cells/metrics.rs` owns cell inset, action slot width, column ratios, readable minimum widths, and drop order.
- `cells/allocation.rs` owns the pure column allocation policy: proportional distribution, readable minimums, overflow recovery, low-priority column dropping, and the `layoutNarrow` responsive variant.
- `cells/geometry.rs` owns cell frames and consumes allocation output. It does not decide column policy directly.
- `cells/commands.rs` owns cell text command emission, S15.2 single-line ellipsis, and right alignment for numeric columns.
- `surface.rs`, `actions.rs`, and `style.rs` own row paint frame, action glyphs, and style lookup respectively.

## Display Text Normalization

`cells/text.rs` owns presentation-only cleanup for legacy and declared table cells before paint commands are emitted. It expands compact UI table labels such as `Rev` to `Revision`, compact asset type labels such as `Tex` / `Mat` to `Texture` / `Material`, compact revision values such as `r42` to `rev 42`, and compact sizes such as `12K` / `1.2M` to `12 KB` / `1.2 MB`.

This is deliberately not a font selection or data-source rewrite. The table painter still uses `METRICS.font_body` and the retained host runtime text path for measured rendering, while the asset/workspace snapshots and `.zui` authored text remain unchanged. Concrete font families, colors, and visual style choices must stay on the global appearance preference/token route rather than in this cell-text owner.

## Responsive Columns

The table painter supports two degradation inputs:

- Local available width: if the row cannot fit all readable column minimums, columns drop in `Rev -> Size -> Type -> Name` order while keeping the name column alive.
- Projection context: retained-host UI projection may append the `layoutNarrow` component variant for table nodes when their pane/window context is narrow. In that tier, numeric columns are hidden even if a single row has enough local width, so Asset Browser tables stay readable in collapsed/narrow compositions.

The painter only consumes the variant token. Context classification is owned by `ui/template_layout_context.rs`, keeping Workbench/pane width knowledge out of the paint leaf.

## Surface Semantics

Workbench table rows follow the Unreal Slate `TableView` precedent: row, header, and tail/empty-fill surfaces all use the same recessed table surface. `style_selector/workbench_table_row/palette.rs` owns those palette roles and maps `WORKBENCH_TABLE_HEADER_BG` and `WORKBENCH_TABLE_TAIL_BG` back to `WORKBENCH_TABLE_ROW_BG`/`PALETTE.surface_inset`. Selection remains a separate low-emphasis fill plus the 2 px left indicator in `surface.rs`; the header/tail change only removes the extra stacked black panels around the list.

## Boundary

This subtree does not own table data production, sorting, user-resizable column persistence, template traversal, command replay, or Workbench layout tier classification. It is a retained-host software paint leaf. User column resizing belongs to layout/persistence work, not this module.

## Validation Notes

The current evidence covers formatting, lib compile, the narrow-context column drop regression, selected-row border suppression, and the 2026-06-26 header/tail recessed surface regression. The latest S15.4aj/S15.6aa pass also ran package build plus the M3 screenshot harness with output in `docs/tests/editor/` and Cargo artifacts in `D:\cargo-targets\zircon-editor-components-0626`. Existing warning noise remains outside this module.

The 2026-07-03 display-normalization slice has scoped rustfmt evidence for `cells/text.rs`. Its `cargo test` wrapper attempts timed out during Windows lib-test compilation/linking while other runtime `cargo`/`rustc` jobs were active, so the wrapper invocations are recorded as blocked and not green. The produced editor test binary was then run directly: the four `archived_asset_table` tests and the declared-cell normalization test all passed, and the ignored M3 screenshot harness passed, refreshing `docs/tests/editor/editor-window-m3-asset-browser-list-900x620.png` at `2026-07-03 04:45:35 +08:00`. That screenshot now shows the Asset Browser Type column spelling `Texture` while the same rendered list keeps `Revision`, `64 KB` / `1.2 MB`, and `rev 42`.
