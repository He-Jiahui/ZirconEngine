---
title: Runtime18 M4 Ability Metric Bucket Dispatch
category: zircon_runtime
report_id: Runtime18-m4-ability-metric-bucket-dispatch-2026-08-26
date: 2026-08-26
session_id: root-runtime18-ability-metric-buckets-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime18 M4 ability metric bucket dispatch

## Scope

- Parent gaps: `CONTENT-P1-026` and `CONTENT-G07`, plus the M4 portion of `CONTENT-P1-063` / `CONTENT-G13`.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: the M4 Zr generator, generated ability catalog, existing catalog package test, source/performance contract, and this record.
- This slice optimizes only the integer-index dispatch inside `metric(index, level, field)`. String `indexOf`, field-name dispatch, other accessors, effects, other catalogs, BuildSet work, and complete Runtime18 qualification remain open.

## Change

- Kept the 117 catalog entries and all rank/field result bodies byte-for-byte generated from `m4_abilities.json`.
- Replaced the single 117-entry metric index chain with 15 generated buckets of at most eight entries.
- The public function computes `index / 8`, selects one bucket, and delegates without allocating or copying catalog data.
- Preserved the existing rank admission, unknown-index, unknown-field, and returned metric semantics.
- Updated the existing Zr catalog contract from its stale 21-entry digest to the current 117-entry digest.

Worst-case index comparisons fall from 117 to at most 23: 15 bucket selections plus eight local comparisons. This is an 80.342% structural reduction before field and rank comparisons, which are unchanged.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime18_m4_ability_metric_performance_contract -v` initially failed 3/5 because metric buckets were absent and the Zr contract still expected 21 entries.
- GREEN: the same command passes 6/6, including byte-for-byte comparison of all 117 metric result branches against the pinned baseline.
- `node examples/woc/tools/m4_ability_zr_codegen.mjs --check` passes and proves both generated projections match the generator and current JSON source.
- `node --check examples/woc/tools/m4_ability_zr_codegen.mjs` and scoped `git diff --check` pass apart from Git's existing LF/CRLF checkout notice.
- The broader WOC Node batch is already red outside this candidate: typed command payload generation reports 157 while a legacy pet guard expects 148. Of 70 historical guards that directly read the M4 generator, 68 still pin older 79/96-entry generations. These are recorded as pre-existing Runtime18 test-inventory drift and are not counted as candidate failures or passes.

The deterministic Node dispatch model first checks all 117 valid indexes, then measures 31 alternating sample pairs over 250,000 fixed pseudo-random queries per sample using nearest-rank percentiles. Each branch comparison increments an observable counter so the model represents the Zr interpreter's sequential branch work instead of a JavaScript jump-table optimization.

| Metric | 117-entry linear dispatch | 8-entry bucket dispatch | Change |
|---|---:|---:|---:|
| P50 | 40.3478 ms | 15.2435 ms | -62.220% |
| P95 | 53.3723 ms | 17.8608 ms | -66.535% |
| comparisons / 250k queries | 14,739,083 | 3,064,366 | -79.209% |

These timings isolate index dispatch. They do not claim end-to-end WOC tick, ZrVM startup, field-name lookup, or full catalog query latency.

## Async validation

One coordinator batch must run the six Python source/performance contracts, generator syntax/freshness checks, the parity/performance model, candidate diff checks, and `woc_m4_ability_catalog_tests.zrp` against pinned external ZrVM commit `60f6bcf4dd22bb6f5247e353bd0d97964758f157` in one managed Cargo group. The already-red full WOC inventory batch remains separate failure evidence and must not be relabeled as a candidate regression.

Acceptance requires the Zr package to compile and return zero, all 117 indexes to remain covered exactly once, and P50/P95 reductions to remain at least 35%. Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 and comparison-count row and label it as metric-index-dispatch-only evidence.
