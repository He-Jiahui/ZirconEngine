---
title: Editor07 Play Hierarchy Sparse Capacity and Selection Record
category: zircon_editor
report_id: Editor07-play-hierarchy-sparse-capacity-selection-2026-10-13
date: 2026-10-13
implementation_status: source_reviewed
validation_status: managed_validation_pending
performance_status: deterministic_capacity_gate_retained
---

# Editor07 Play Hierarchy Sparse Capacity and Selection Record

## Scope

This change reserves the changed-row output from the actual number of changed hierarchy rows, preserves ordered patches, and keeps selection state coherent when a `NotModified` response forces a reflow. It retains the existing deterministic allocation-growth gate and makes no CPU percentile claim.

## Projection behavior

- For an advancing generation with unchanged topology, one pass compares row values and counts changes. The collector then clones changed rows in input order into `Vec::with_capacity(changed_count)`; hierarchy anchors reserve the same number of entries.
- If the count is known to be zero, the empty vector is still created and its capacity is recorded, while the row collector loop is skipped. For a same-generation non-forced response, the existing equality guard establishes equality, so that result is reused as zero without another count or collect scan.
- The same-generation mutation guard remains ahead of the zero-count and forced-reflow branches. A changed row at an unchanged generation is rejected whether `force_reflow` is false or true.
- `NotModified` with `force_reflow` publishes the selection revision and selected entities contained in the reflow and updates the retained snapshot to that same selection base.

## Regression and performance contract

The actual `PlayHierarchyProjection::apply` regression uses 4,096 rows for zero, one-row, and dense changes. A test-only spy reads the collector vector's initial and final capacities and counts each actual push that changes capacity. It verifies exact capacities of 0, 1, and 4,096 with zero growth, plus the empty-delta and ordered row/anchor behavior.

The ignored `EDITOR815_PLAY_HIERARCHY_CHANGED_ROW_CAPACITY_BENCH_V1` producer builds the legacy output with a real `Vec::new()` and counts its actual growth transitions. It retains the original gate of 11 legacy growth events versus 0 optimized growth events. This measures allocation shape only; there is no new P95 or elapsed-time budget, and no measured result is claimed here.

The selection regression covers revision 2 with entity 1 selected, forced reflow at revision 3 with entity 2 selected, a repeated revision 3 response returning no delta, then revision 4 producing a delta based on revision 3 with entity 1 added and entity 2 removed. A separate regression verifies same-generation mutation rejection on both forced and non-forced paths.

## Validation state

The exact R2 change guard passed with zero errors and zero warnings. Static source review, Rust formatting, scoped whitespace checks, and Python AST parsing are recorded as complete. Cargo tests, the ignored Release producer, and the four Python unittests were not executed locally. Functional and performance acceptance remain pending the next root-merged Jenkins batch.

The previous batch08 intake, record/summary, and response files were absent in root's local reconciliation; their absence does not establish why they are missing or whether a queue accepted the work. Keep that history unresolved.

Batch09 file handoff was written by root with exit code 0, then hash-checked with retained copies: 15 source files, 15 Cargo files, and 1 Python file. The retained handoff metadata is `.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.json` (SHA-256 `c934086320a3c4598f8b441b253fdbcf6d029348c7c00b745d5df1977d4a653f`) and `.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.md` (SHA-256 `d127f64409f02a97fcbdd71120cb48b53a06974e5816a655424ff442ae316e4b`). Queue acceptance was not observed. Native and performance validation remain pending; writing handoff files is not an acceptance result.

## Review artifacts

- Final cumulative patch: `.codex/outbox/reports/optimize-20261013-01a13451/editor-play-hierarchy-r2.patch` (SHA-256 `98fef1b51bfc38e7a8f92310040555e5a6b309724f3f3030fb0f84f2b5eee699`). Use R2 instead of R1; the current checkout already contains the changes.
- Evidence: `.codex/outbox/reports/optimize-20261013-01a13451/editor-play-hierarchy-r2-evidence.json` (SHA-256 `eb5017e4f9c05cfcba88bd0a6ffb96319a87faabf289515751f3490d6f01dbe3`).
- Exact change guard: `.codex/outbox/reports/optimize-20261013-01a13451/editor-play-hierarchy-r2-guard.json` (SHA-256 `2c508b9bfd59ec06f631b7bad5bab190351465695037ee8becd4d380bb56a8e0`; passed, 0 errors, 0 warnings).

The Python contract is an existing gitignored working-tree file, not a newly added tracked source file. Its R2 patch delta is based on the exact verified preexisting bytes with SHA-256 `2ce537dd6d3d165d227460ed34611684ed1ca43e1018299b0b1f76bfd4d5e99f`; the Python hunk is not based on original task HEAD. R1 remains historical evidence, and Editor63's frozen files were not changed.

## Exact validation filters

- Rust regression module: `ui::host::play_hierarchy_projection::tests::` (8 non-ignored tests).
- New focused Rust regressions: `ui::host::play_hierarchy_projection::tests::same_topology_changed_row_capacity_matches_zero_sparse_and_dense_changes`, `ui::host::play_hierarchy_projection::tests::not_modified_force_reflow_updates_the_selection_delta_base`, and `ui::host::play_hierarchy_projection::tests::same_generation_mutation_rejects_before_reflow_or_empty_delta`.
- Ignored Release producer: `ui::host::play_hierarchy_projection::optimization_batch_tests::editor815_play_hierarchy_changed_row_capacity_bench_v1`, with `--exact --ignored --nocapture --test-threads=1`.
- Python contract: `tools/tests/test_editor_play_hierarchy_changed_row_capacity_performance_contract.py`, class `EditorPlayHierarchyChangedRowCapacityPerformanceContractTests` (4 tests).
