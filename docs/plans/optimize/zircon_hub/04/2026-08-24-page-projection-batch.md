---
title: Hub04 Page Projection Performance Batch
category: zircon_hub
report_id: Hub04-page-projection-batch-2026-08-24
date: 2026-08-24
session_id: optimize-hub04-page-projection-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Hub04 Page Projection Performance Batch

## Scope

This batch advances the HCTL-G36 10k projection budget through two independent page-level tasks.
It does not claim the wider generation/delta read model, history pagination, or full HCTL-G35/G36
allocation and RSS work is complete.

## Task 1: Delivery Action Projection

Cloud previously filtered the full action history once for package actions and again for install
actions. `collectDeliveryActions` preserves the source order and collects both groups in one pass.

## Task 2: Source Engine Choices

The Source Engine popover previously retained every non-active engine in a fallback array and then
rendered only its first two entries. `selectSourceEngineChoices` preserves explicit ID, configured
active engine, and first-engine fallback precedence while retaining at most the two displayed
fallbacks. The component memoizes the projection for unrelated rerenders.

## Performance Evidence

Windows-native Node 22.13.1, 21 alternating sample pairs, 20 iterations per sample, nearest-rank
percentiles, and 10,000 entries produced:

| Task | Before P50 | After P50 | Before P95 | After P95 | P95 change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Delivery action projection | 8.3071 ms | 1.9070 ms | 30.0552 ms | 2.8614 ms | 90.480% lower |
| Source engine choices | 14.4381 ms | 2.6604 ms | 22.9898 ms | 14.3148 ms | 37.734% lower |

Deterministic work counts are `20,000 -> 10,000` item checks for delivery projection and
`9,999 -> 2` retained fallback references for source-engine choices. Coordinator ticket
`0fae0452fa0344928f2e610430e4795f` recomputed the nearest-rank percentiles from all raw samples and
accepted both dynamic gates.

## Validation

- Red state: the batch test failed because both projection modules were absent.
- Focused behavior: 3/3 passed; the performance case is a separate gated fourth test.
- Local performance: 4/4 passed with both 10k P95 thresholds.
- Full Hub TypeScript typecheck: passed.
- Existing Hub Node behavior batch: 7 passed, 2 performance cases skipped as designed.
- Node 22 requires `--experimental-strip-types` for the repository's direct `.ts` test imports; the
  managed script fixes that command contract explicitly.
- Managed dependency restore, TypeScript typecheck, 4/4 focused Node tests, raw-sample percentile
  recomputation, and both 10k performance gates: passed.
- No local Cargo lane was launched and no Cargo process was terminated.

## Remaining Parent-plan Work

ZHUB-CTL-P1-44/P1-45, ZHUB-CTL-P2-12 through P2-14, HCTL-G35, and the remainder of HCTL-G36 still
require an authoritative incremental read model, bounded history/task/catalog paging, projection
metrics, and product-level RSS/I/O evidence.
