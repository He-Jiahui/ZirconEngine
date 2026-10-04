---
title: Editor359 Reconcile Borrowed Asset ID Admission
category: zircon_editor
date: 2026-09-26
implementation_status: implemented
validation_status: batched_validation_pending
---

# Editor359 Reconcile Borrowed Asset ID Admission

## Scope and target

After UI asset watcher overflow, the Editor scans open authoring sessions and their imports under a bounded reconcile cursor. Many fragment-qualified references can normalize to the same asset ID. The scan previously owned a new ID string before each set admission. It now shares the existing change-set normalization path, which probes the ordered set with the borrowed normalized ID and owns only first occurrences.

For `R` references resolving to `N` distinct normalized assets, the deterministic allocation target is **`N` owned ID strings instead of `R`**, without changing scan budget, cursor progression, fragment removal, or sorted output. The existing ignored Windows Release `EDITOR359_BORROWED_UI_ASSET_CHANGE_DEDUP_BENCH_V1` exercises this shared helper through duplicate-heavy change-set normalization and retains its pending optimized p95 target at most 70% of the legacy baseline. `EDITOR359_UI_ASSET_CHANGE_UNIQUE_MIXED_BENCH_V1` adds 128-event all-unique and 50%-unique workloads with a p95 non-regression limit of 110% of baseline for each. This exposes the extra ordered-set lookup cost when few entries repeat. A product overflow-reconcile percentile remains pending; no dynamic pass is claimed here.

## Behavior and validation

- Ordinary change-set normalization and bounded reconcile now call one `insert_normalized_ui_asset_id` helper.
- A focused regression verifies two fragment aliases stage one asset while a separate style asset retains its ordered entry.
- The existing normalize regression and borrowed-key benchmark, plus the all-unique/mixed Release comparison with baseline equality checks, remain part of the next combined Editor package batch.

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/host/asset_editor_sessions/refresh/{normalize,reconcile}.rs` | Rustfmt, scoped diff check, and borrowed-ID structural check passed |
| Cargo | one managed Windows `zircon_editor` check and focused normalization/reconcile regressions in the shared Editor batch | pending coordinator batch |
| Performance | ignored duplicate-heavy p95 ≤ 70% and all-unique/mixed p95 ≤ 110% helper gates, plus overflow-reconcile product p50/p95/p99 | pending coordinator batch; all three p95 gates unmeasured |

This is an implemented optimization candidate until the shared validation and performance gates reach a terminal pass.
