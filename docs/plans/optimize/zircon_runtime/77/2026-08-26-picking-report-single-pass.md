---
title: Runtime77 Picking Report Single Pass
category: zircon_runtime
report_id: Runtime77-picking-report-single-pass-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Picking Report Single Pass

## Scope

This slice combines pointer hit reporting into one traversal of the already-sorted hit list.
Blocking precedence, inclusion of the blocking hit in hover counts, all-list non-hoverable counts,
top-target selection, empty inputs, and aggregate report fields remain unchanged.

## Change

- Accumulate hoverable-before-block, non-hoverable-all, and first-blocking-target fields together.
- Replace the blocker-position scan plus two filter/count scans with one summary helper.
- Keep top-target access as the existing constant-time first-element lookup.

## Deterministic Performance Evidence

| 65,536 hits, 64 summaries per sample | Before | After |
|---|---:|---:|
| Hit visits per sample | 12,582,912 | 4,194,304 |
| Full hit scans per summary (no blocker) | 3 | 1 |
| Temporary hit collections | 0 | 0 |
| Output fields computed | 3 | 3 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME77_PICKING_REPORT_SINGLE_PASS_BENCH_V1`. Acceptance requires single-pass summary P95 to be
at least 50% below three scans. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `runtime77_picking_report_preserves_blocked_hover_and_non_hover_counts`
  covers blocking, hover, non-hover, top-target, and unblocked behavior.
- `runtime77_picking_report_uses_one_hit_scan` requires one hit loop and rejects
  position/filter scans in the owned summary path.
- `runtime77_picking_report_single_pass_p95` reports paired P50/P95 samples and
  enforces the 50% P95 reduction gate.

These tests are grouped with analog-navigation single normalization in one two-task asynchronous
coordinator batch. Terminal timings, integration, record finalization, and automatic WeCom
delivery remain pending.

## Remaining Parent-plan Work

Runtime77 still owns the broader event/UI interaction stack, capture, focus, navigation, dispatch,
and product-scale performance receipts. This slice only converges picking diagnostic projection.
