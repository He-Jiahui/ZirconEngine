---
title: Runtime07 Export Build Fatal Diagnostic Hash Merge
category: zircon_runtime
report_id: Runtime07-export-build-fatal-diagnostic-hash-merge-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Export Build Fatal Diagnostic Hash Merge

## Scope

This slice removes repeated output-vector scans while required-plugin failures are merged into an
export build plan's fatal diagnostics. Existing diagnostic order and duplicates, new diagnostic
first-occurrence order, exact text, report ownership, and public serialization remain unchanged.
It advances Runtime07 export control-plane projection without claiming completion of plugin
resolution, trust, ABI compatibility, isolation, generation publication, or product acceptance.

## Change

- Index existing fatal diagnostic text with one borrowed `HashSet<&str>`.
- Track accepted generated diagnostics with a second hash set while retaining their insertion order
  in a separate vector.
- Pre-size both indexes and the accepted vector from existing lengths and iterator size hints.
- Release the borrowed existing-text index before extending the original owned output vector.

## Deterministic Performance Evidence

| 4,096 existing and 4,096 new fatal diagnostics | Before | After |
|---|---:|---:|
| Pairwise string comparisons | 25,163,776 | 0 |
| Existing-index visits | 0 | 4,096 |
| Existing/new hash probes | 0 | 8,192 |
| Existing output duplicates removed | 0 | 0 |

Deterministic lookup work falls by 99.9512%. The ignored release gate runs 17 alternating sample
pairs and emits `RUNTIME07_EXPORT_FATAL_DIAGNOSTIC_HASH_MERGE_BENCH_V1`. Acceptance requires hash
merge P95 to be at least 75% below the legacy repeated Vec scan. Exact Windows P50/P95 timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bi_fatal_diagnostic_hash_merge_preserves_existing_and_new_order`
  covers existing duplicate retention, cross-source deduplication, and new first-occurrence order.
- `optimization_batch_20260826bi_fatal_diagnostic_hash_merge_eliminates_pairwise_work` locks the
  25,163,776-comparison model and rejects the repeated output scan.
- `optimization_batch_20260826bi_fatal_diagnostic_hash_merge_p95` reports paired release P50/P95
  samples and enforces the 75% P95 reduction gate.

## Remaining Parent-plan Work

Runtime07 still owns package resolution, catalog generations, native ABI conformance, trust and
isolation, capability policy, hot-reload leases, VM execution budgets, typed marshalling,
debugging, and stress/fault evidence. This slice only converges export build fatal diagnostic
projection.
