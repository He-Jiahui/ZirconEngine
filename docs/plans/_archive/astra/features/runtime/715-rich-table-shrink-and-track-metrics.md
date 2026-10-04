---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/84/2026-08-31-allocation-free-rich-table-shrink-sum.md
  - docs/plans/optimize/zircon_editor/01/2026-08-28-rich-table-track-metrics-authority.md
implementation_files:
  - zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs
tests:
  - zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs
  - tools/tests/test_runtime_rich_table_sizing_performance_contract.py
  - tools/tests/test_runtime_rich_table_track_metrics_performance_contract.py
  - tools/tests/test_runtime_rich_table_track_metrics_pressure.py
  - tools/tests/test_runtime_text_rich_source_contract.py
---

# Rich-table shrink and track-metrics completion

The Runtime rich-table layout path now combines two bounded hot-path improvements from the
optimize plans: shrink-budget admission folds directly over borrowed width iterators, and every
placement phase reads one gap-aware `TrackMetrics` prefix authority per axis. This removes the
repeated temporary width vectors and repeated span-slice summation while preserving minimum-width,
gap, direction, clamping, and malformed-range behavior.

## Plan completion list

| Work | Status | Local evidence |
|---|---|---|
| Runtime84 shrink-sum allocation removal | implemented_pending_validation | Sizing contract and rich-text source contract pass; deterministic model reaches 0 temporary vector allocation sites and 0 temporary `f32` writes in the 8,192-column/24-probe workload. |
| Editor01/RichTable track-metrics authority | implemented_pending_validation | Track-metrics geometry and pressure contracts pass; the 10,000-cell model reduces span track work from 1,040,000 to 50,000 while keeping placement queries linear in cell count. |

## Batched verification

One local invocation covering rich-table sizing, track-metrics pressure, rich-text source invariants,
text-decoration source-map paths, and the current Editor retained-UI projection/cache contracts
passed `54/54` tests in `0.039s`. Scoped source checks and the existing rich-table Rust regressions
remain in the managed validation set; no tooling source was changed.

The allocation and work figures above are deterministic algorithm models, not CPU, RSS, or
input-to-present measurements. Managed Windows Cargo execution and Release product evidence for
allocation/time plus p50/p95/p99 remain pending, so this record stays
`implemented_pending_validation`.
