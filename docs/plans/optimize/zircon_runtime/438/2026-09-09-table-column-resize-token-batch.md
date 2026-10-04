---
title: Runtime438 Table Column Resize Token and Batch Projection
category: zircon_runtime
report_id: Runtime438-table-column-resize-token-batch-2026-09-09
date: 2026-09-09
session_id: root-runtime-editor-optimize-20260909-table-resize
source_plan:
  - docs/plans/optimize/zircon_editor/01/2026-08-28-table-column-resize-scalar-authority.md
  - docs/plans/optimize/zircon_editor/01/2026-08-29-table-column-resize-scalar-authority.md
  - docs/plans/optimize/zircon_runtime/254/2026-08-26-table-width-in-place-update.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime438 Table Column Resize Token and Batch Projection

## Scope

The retained table resize route previously decoded a serialized drag token and allocated a new
field string on every pointer move. It then performed two independent aggregate property
transactions for `column_widths` and `columns`. This slice keeps the existing metadata and binding
semantics while removing the per-move token decode and coalescing the two projections into one
metadata batch.

## Implementation

- `UiSurfacePointerDragResizeState` stores the field in an `Arc<str>` and keeps the numeric start and
  minimum widths at press time. The old serialized property remains for snapshot compatibility and
  is used only as a fallback for older drag state.
- The table move and release paths clone the typed token (an `Arc` clone, not a string allocation)
  and no longer parse the serialized token in the normal path.
- Same-width samples return before constructing either aggregate value or recording dirty state.
- `apply_table_column_width_batch` builds the compatible map/array projections from one metadata
  snapshot and applies them through one metadata batch. Runtime style, component-state sync,
  invalidation, and one binding report per changed aggregate retain the legacy ordering and shape.
- Missing or malformed metadata continues through the existing aggregate helper fallback.
- Resize routing and width-specific batch mutation now live in the named `resize.rs` and
  `width_mutation.rs` children; the table root and generic mutation owner stay below their
  repository file-budget contracts.

## TDD and local evidence

- RED: typed-token and same-width contracts were added before the corresponding methods and guard;
  the pre-change source had no typed resize state and always entered aggregate mutation.
- Static GREEN: Rustfmt check, source contracts, and scoped `git diff --check` pass for all nine
  touched Runtime owners. The route and width-mutation responsibilities are kept in named child
  modules so the table binding owners remain within their existing file budgets. Pure projection
  tests cover both aggregates, missing columns, and map entry replacement. The broader Runtime
  Python contract batch remains independently green where previously recorded.
- The standalone model
  `.codex/state/session-coordinator/runtime438-table-resize-batch-model.rs` compiled with
  `rustc --edition 2021 -O` and emitted:

  `RUNTIME438_TABLE_RESIZE_BATCH_MODEL_V1 columns=256 metadata_fields_per_column=8 pointer_moves=2000 same_width_moves=500 changed_moves=1500 legacy_token_parses=2000 optimized_token_parses=0 legacy_projection_work=8192000 optimized_projection_work=6144000 legacy_transactions=4000 optimized_transactions=1500 legacy_work=8198000 optimized_work=6145500 reduction=2052500 reduction_ratio_x100=133`

  This is a deterministic operation/allocation model, not CPU or input-to-present evidence.

## Source fingerprints

| Owner | SHA-256 |
|---|---|
| `zircon_runtime/src/ui/surface/input/state/pointer_drag.rs` | `E3E2B640046C2D282981DC980E08520F7F80A9ED6505184195595F0BC10935C7` |
| `zircon_runtime/src/ui/surface/input/state/pointer_drag/hash_clear_tests.rs` | `E993BC65EE8914EEBFB6197DDF562E7E7C0EBF474170ABD0B68F60C1844D6BE4` |
| `zircon_runtime/src/ui/surface/input/state/mod.rs` | `EA76DD5521EC640B1CFBCDD709F26BDB3E90E9206B943861F443C060678EBA30` |
| `zircon_runtime/src/ui/surface/input/mod.rs` | `4587C1FC3CA2720148D172C7190FB4E80D9330D86E892881A99D6FE23AFEEFE4` |
| `zircon_runtime/src/ui/surface/surface/default_interactions/table/mod.rs` | `F7DB6FD1C265B098D61679582D4E9CFED5DE31E314A894B62F4FA25342AED670` |
| `zircon_runtime/src/ui/surface/surface/default_interactions/table/resize.rs` | `11499A6A09EEB9F0D7EA5978C462EF8DB93F1AC3EFA0B5DAAE2F757C5641AD52` |
| `zircon_runtime/src/ui/surface/surface/default_interactions/table/width_mutation.rs` | `84057A9805807E356E7311A70780764BB815C26105CD8C64B1158526271CDE8C` |
| `zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs` | `6B2F18A262140B5EF832334935E3DAC7F10B73BE38790B14C89E5F0CC6B9103F` |
| `zircon_runtime/src/ui/surface/surface/default_interactions/table/mutation.rs` | `84EBBE22A3A8F537194A2655BF86FCA19C68809A3BAD0502B8137EBA6754B37B` |

## Validation boundary

No workspace Cargo validation was started by this slice. Managed admission remains blocked by the
external dependency snapshot: `E:/Git/zr_vm` is at HEAD
`d717af6c8fefb4f0e1a5a69423904327296f1254` with 75 dirty status entries. The coordinator must
run current-source Windows tests and the ignored release benchmark after that prerequisite is
clean. Product CPU, allocation bytes, RSS, and input-to-present p50/p95/p99 are therefore still
pending; this record does not promote the design-ready scalar authority to a performance-qualified
state.
