---
title: Editor57 table row benchmark P95 sample coverage
category: zircon_editor
report_id: Editor57-table-row-P95-samples-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
implementation_status: benchmark_source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: release_p95_not_measured
---

# Editor57: make the table-row P95 benchmark sample its tail

The ignored Release benchmark for exact table-row capacity used 11 paired samples and nearest-rank P95. `ceil(11 x 0.95) - 1` selects index 10, the maximum, so the reported P95 was actually the worst observed pair. It now uses 101 alternating retired/optimized pairs. The same estimator selects sorted index 95, leaving five higher observations. Each pair still runs 32 synchronizations of a 2048-row cold table, and the existing optimized-P95-at-most-90%-of-retired gate is unchanged. This is benchmark-estimator repair; it does not change production table synchronization.

The original 2026-08-25 optimization record's sample count was updated to reflect the current source. `rustfmt --edition 2021 --check` and scoped `git diff --check` passed. The Release benchmark has not run in the managed batch, so the 10% threshold and product Asset Browser performance remain unverified. The larger sample count raises benchmark work from 11 to 101 paired measurements; it is an ignored Release lane rather than an unfiltered unit-test cost.
