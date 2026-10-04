---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/438/2026-09-09-table-column-resize-token-batch.md
  - docs/plans/optimize/zircon_editor/01/2026-08-28-table-column-resize-scalar-authority.md
  - docs/plans/optimize/zircon_editor/01/2026-08-29-table-column-resize-scalar-authority.md
---

# Table Column Resize Token and Batch Projection

The Runtime resize route now retains a typed press-time token and skips unchanged width samples.
Compatible `column_widths` and `columns` projections are prepared from one metadata snapshot and
committed as a batch, while the legacy serialized token remains only as a snapshot fallback.
Resize routing and width-specific batch mutation are kept in named `resize.rs` and
`width_mutation.rs` children so the table root and generic mutation owner remain within their
existing structure budgets.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime438/Table resize | typed token, same-width no-op, batched aggregate projection, and owner split | `implemented_pending_validation` | Batched Runtime/table static checks pass `41/41`, `py_compile`, Rustfmt, source contracts, scoped diff check, and standalone operation model pass; included in the Runtime/Editor merged performance-contract batch `1723/1723` (Runtime `1143/1143`, Editor `580/580`). One unrelated contact-shadow fixture remains a documented stale count (3 current fields vs 2 expected). Managed Windows Cargo, table pointer regressions, and release CPU/allocation/RSS/input-to-present p50/p95/p99 remain pending under the dirty external `E:/Git/zr_vm` checkout. |
