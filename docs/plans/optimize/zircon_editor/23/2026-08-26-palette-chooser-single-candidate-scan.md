---
title: Editor23 Palette Chooser Single Candidate Scan
category: zircon_editor
report_id: Editor23-palette-chooser-single-candidate-scan-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Palette Chooser Single Candidate Scan

## Scope

This slice removes repeated candidate comparison and selected-target lookup from palette target
chooser reconciliation. Sticky retention, candidate equality fields and order, manual selection,
invalid selected-index fallback, changed reporting, and chooser construction remain unchanged.

## Change

- Compute candidate-set equality once for sticky and manual-selection branches.
- Reuse the previous selected index after positional candidate equality has already been proven.
- Require the previous selected target to remain valid before preserving manual selection.

## Deterministic Performance Evidence

| 16,384 candidates, 64 reconciliations per sample | Before | After |
|---|---:|---:|
| Candidate visits per sample | 3,145,728 | 1,048,576 |
| Full candidate scans per reconciliation | 3 | 1 |
| Selected-target position scans | 64 | 0 |
| Candidate collections cloned | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_PALETTE_CHOOSER_SINGLE_CANDIDATE_SCAN_BENCH_V1`. Acceptance requires single-scan
reconciliation P95 to be at least 50% below repeated comparison and lookup. Exact Windows timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826aw_palette_chooser_preserves_manual_sticky_and_invalid_selection`
  covers manual reuse, sticky mismatch retention, and invalid-selection fallback.
- `optimization_batch_20260826aw_palette_chooser_scans_candidate_set_once` requires one candidate
  equality check, direct index reuse, and no position scan.
- `optimization_batch_20260826aw_palette_chooser_single_candidate_scan_p95` reports paired P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation, preview
fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only converges
palette target chooser reconciliation.
