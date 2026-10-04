---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-runtime-authored-geometry-delta-publication.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
related_records:
  - docs/plans/astra/features/runtime/681-hit-test-stacked-output-capacity.md
  - docs/plans/astra/features/runtime/687-hit-grid-cell-batch-compile-repair.md
  - docs/plans/astra/features/runtime/727-runtime-hit-grid-entry-capacity.md
  - docs/plans/astra/features/runtime/728-runtime-pointer-state-single-node-accumulator.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
tests:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - tools/tests/test_runtime_ui_hit_grid_cell_iterator_contract.py
  - tools/tests/test_runtime_ui_hit_grid_budget_contract.py
  - tools/tests/test_runtime_ui_projected_hit_grid_budget_contract.py
---

# Runtime hit-grid cell projection iterator

The full base and projected hit-grid builders previously materialized a temporary `Vec<usize>`
for every entry only to append those indices immediately into the backing cells. The cell span is
already bounded and deterministic, so the projection now returns a lazy row-major iterator. Full
rebuilds consume it directly and avoid one short-lived allocation per entry. Incremental geometry
patches explicitly collect only the changed entry's cells because those owned vectors are retained
in the patch transaction and entry-cell index.

The shared finite-frame, axis, cell-count, coarsening, ordering, and fail-closed behavior is
unchanged. No second spatial authority or unbounded iterator is introduced; an invalid span
produces an empty iterator.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime11C hit-grid publication | Replace per-entry full-rebuild cell-index vectors with a bounded lazy iterator; retain explicit collection only for incremental changed-entry ownership. | RED/GREEN iterator source contract, in-file row-major/invalid-span regression, existing hit-grid budget/route contracts, and one-process Runtime/Editor batch; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

For `E` entries and `M` cell memberships, full rebuild remains `O(E + M)` but removes `E`
temporary index-vector allocations and their intermediate writes. The iterator's span is derived by
the existing checked geometry mapper and is bounded by the shared grid limits. Incremental patches
still own `Vec<usize>` values for previous/next membership comparison and publication; they collect
only entries in the changed set and preserve atomic preflight behavior.

## Local evidence

- TDD source contract first failed against the eager `Vec::with_capacity`/`push` implementation,
  then passed after the lazy `impl Iterator<Item = usize>` projection and explicit patch-side
  collection were installed.
- The in-file regression verifies row-major indices and an invalid-frame empty result.
- The combined Runtime/Editor performance-plus-pressure loader covered `349` modules and passed
  `1341/1341` tests in `8.415s` with zero failures or errors, including the new iterator contract.
- `rustfmt --edition 2021 --check` passed for all three production owners; scoped diff and Python
  syntax checks passed.
- Current source SHA-256:
  - `zircon_runtime/src/ui/tree/hit_test.rs`: `C8C088865DE54400323FD64DF1E9D90D70BBE0347C9D22F70EBC76042E288A36`
  - `zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs`: `986E26B9560C503E95AFC0012AD607E3F7B8E30440FAEA5D77AFD41371047C8E`
  - `zircon_runtime/src/ui/surface/frame_hit_test.rs`: `FD8D2AFEDE53A21580A4C6A18F9275E9CEE63C149C3B64CCE15A34B92952C218`
  - `tools/tests/test_runtime_ui_hit_grid_cell_iterator_contract.py`: `395F7A1F5466FD9AAAB8AD1D9CBFA058CCF3F66ECEA6646D71F8C2D497952881`

This is source/contract evidence, not managed Cargo execution or product CPU, allocator, RSS, or
p50/p95/p99 acceptance evidence.

## Managed acceptance gate

This slice joins the deferred owner-attributed Runtime/Editor admission recorded in `696`. No new
coordinator request or status query is issued for this local change. Keep the row pending until a
managed Windows Release run measures full-rebuild hit-grid allocation and input latency at the
declared UI-node workloads.
