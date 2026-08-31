---
title: Runtime81 Kinsoku Match Dispatch
category: zircon_runtime
report_id: Runtime81-kinsoku-match-dispatch-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime81 Kinsoku Match Dispatch

## Scope

This slice removes six sequential linear character-table scans from the Japanese forbidden
line-start check used by text line breaking. The existing 95 authored entries, including the one
intentional overlap between categories, are preserved exactly in one compiler-dispatched
`matches!` expression. JLREQ inseparable pairs, forbidden line-end rules, chunk merging, mandatory
breaks, glyph fallback, and source/visual ranges are unchanged.

This is a narrow Runtime81 hot-path improvement. It does not close `RTS-P1-027`: locale-specific,
versioned JLREQ tailoring still needs a compiled data/provider authority instead of additional
hand-authored production tables.

## Deterministic Work Model

The release workload checks 4,096 ordinary Latin candidates, which exercise the legacy worst-case
miss path without changing the membership result.

| Work per batch | Legacy | Optimized |
|---|---:|---:|
| Candidate characters | 4,096 | 4,096 |
| Sequential scalar probes | 389,120 | 0 |
| Membership dispatches | 0 | 4,096 |
| Membership changes across 95 authored entries | 0 | 0 |

Deterministic membership work falls by 98.9474%. The ignored release gate runs 17 alternating
sample pairs and emits `RUNTIME81_KINSOKU_MATCH_DISPATCH_BENCH_V1`. Acceptance requires match
dispatch P95 to be at least 50% below the legacy table-scan implementation. Exact Windows P50/P95
timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bl_kinsoku_match_dispatch_preserves_membership` verifies every
  legacy forbidden character and representative allowed Japanese, Korean, CJK, and Latin input.
- `optimization_batch_20260826bl_kinsoku_match_dispatch_eliminates_table_scans` locks the
  389,120-to-4,096 work model and rejects table scans in the production dispatch.
- `optimization_batch_20260826bl_kinsoku_match_dispatch_p95` reports paired release P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Runtime81 still owns versioned Unicode/locale tailoring, dictionary and hyphenation providers,
typed virtual glyph mapping, shaping/layout budgets, and product-scale multilingual evidence. This
slice only converges the existing Japanese forbidden line-start membership hot path.
