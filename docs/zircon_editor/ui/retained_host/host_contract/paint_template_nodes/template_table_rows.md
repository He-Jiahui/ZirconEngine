---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests.rs
source_plan:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
validation:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - template table row root ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Template Table Rows

`template_table_rows.rs` is the retained-host Workbench table-row painter entry. It keeps the two external call sites used by `template_nodes.rs`: full row paint for Workbench table rows and text-only fallback for table-like template nodes. Detailed recognition, layout, style, surface, and glyph work is now folder-backed.

## Ownership

- `template_table_rows.rs` owns only the dispatch flow: recognize whether a node should be handled, collect cells, delegate row surface/cell/action commands, and expose the test module.
- `template_table_rows/identity.rs` owns table-family recognition and Workbench header/tail/selected control-id checks.
- `template_table_rows/cells.rs` owns declared option cells, legacy text splitting, column ratios, content offsets, per-cell rects, and text command emission.
- `template_table_rows/surface.rs` owns row paint rect adjustment, row background/border command emission, separator command emission, and the row radius.
- `template_table_rows/actions.rs` owns the right-side table action rect plus gear/kebab glyph command emission.
- `template_table_rows/style.rs` owns access to `WorkbenchTableRowStyle` and the derived background, border, border width, and cell text colors.

## Boundary

This subtree remains a retained-host software paint leaf. It does not own template traversal, command replay, Workbench table data production, or runtime UI extraction. `template_nodes.rs` remains the single-node dispatcher; `render_commands.rs` remains the `HostPaintCommand` DTO/replay owner; style selection remains in `style_selector`.

## 2026-06-18 Split

The 2026-06-18 slice reduced `template_table_rows.rs` from 380 lines to 61 lines. Child line counts after formatting are `actions.rs` 108, `cells.rs` 127, `identity.rs` 37, `style.rs` 22, and `surface.rs` 55. `template_table_rows_tests.rs` now imports the owner modules explicitly for helper coverage.

Validation remained feature-first per the user's request. `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, the root ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone testing stage.
