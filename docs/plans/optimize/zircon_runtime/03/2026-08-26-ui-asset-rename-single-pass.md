---
title: Runtime03 UI Asset Rename Single Pass
category: zircon_runtime
report_id: Runtime03-ui-asset-rename-single-pass-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 UI Asset Rename Single Pass

## Scope

This slice combines current-item lookup and duplicate-ID validation for UI stylesheet/rule rename
operations. Missing items, blank-ID rejection, duplicate errors, same-ID reconciliation, first
match behavior, and mutation results remain unchanged.

## Change

- Scan style rules once while retaining the first current position and whether another rule owns
  the requested ID.
- Apply the same single-scan projection to stylesheet rename validation.
- For same-ID reconciliation, compare each populated ID once while still detecting later invalid
  duplicates of the current ID.

## Deterministic Performance Evidence

| 16,384 rules, 128 same-ID reconciliations per sample | Before | After |
|---|---:|---:|
| Full rule passes per reconciliation | 2 | 1 |
| Rule visits per sample | 4,194,304 | 2,097,152 |
| ID comparisons per sample | 4,194,304 | 2,097,152 |
| Temporary indexes/maps | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME03_UI_ASSET_RENAME_SINGLE_PASS_BENCH_V1`. Acceptance requires the single scan P95 to be at
least 20% below locate-plus-duplicate scans. Exact Windows timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826as_ui_asset_rename_preserves_results_and_duplicates` covers rule and
  stylesheet success, missing, duplicate, and same-ID behavior.
- `optimization_batch_20260826as_ui_asset_rename_uses_single_scan` requires both rename entry points
  to use their single-pass helpers and rejects `position` in the owned paths.
- `optimization_batch_20260826as_ui_asset_rule_rename_single_scan_p95` reports paired P50/P95
  samples and enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Runtime03 still owns UI pipeline architecture, scheduling, cache reuse, invalidation, diagnostics,
and product-scale performance receipts. This slice only converges UI asset rename validation.
