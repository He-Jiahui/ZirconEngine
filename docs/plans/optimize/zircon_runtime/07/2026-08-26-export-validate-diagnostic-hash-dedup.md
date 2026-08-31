---
title: Runtime07 Export Validate Diagnostic Hash Dedup
category: zircon_runtime
report_id: Runtime07-export-validate-diagnostic-hash-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Export Validate Diagnostic Hash Dedup

## Scope

This slice removes repeated output-vector scans while export validation diagnostics are
deduplicated. First-occurrence order, exact diagnostic text, fatal classification, report schema,
and caller-visible `Vec<String>` ownership remain unchanged. It advances Runtime07 report
projection without claiming completion of plugin resolution, trust, ABI compatibility, isolation,
generation publication, or export product acceptance.

## Change

- Build one borrowed `HashSet<&str>` over the immutable input vector.
- Record admission decisions, release the borrowed index, and move accepted `String` values into
  the output without cloning diagnostic text.
- Pre-size the hash index, admission bitmap, and final output from known input/accepted counts.
- Keep the index local to one report projection; no borrowed value escapes the function.

## Deterministic Performance Evidence

| 4,096 distinct export diagnostics | Before | After |
|---|---:|---:|
| Pairwise string comparisons | 8,386,560 | 0 |
| Diagnostic hash probes | 0 | 4,096 |
| Accepted diagnostic text clones | 0 | 0 |
| Output order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME07_EXPORT_VALIDATE_DIAGNOSTIC_HASH_DEDUP_BENCH_V1`. Acceptance requires hash admission P95
to be at least 75% below the legacy repeated Vec scan. Exact Windows P50/P95 timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826bh_export_validate_hash_dedup_preserves_first_order` covers
  duplicate removal and first-occurrence order.
- `optimization_batch_20260826bh_export_validate_hash_dedup_eliminates_pairwise_work` locks the
  8,386,560-comparison model and rejects the repeated output scan.
- `optimization_batch_20260826bh_export_validate_hash_dedup_p95` reports paired release P50/P95
  samples and enforces the 75% P95 reduction gate.

## Remaining Parent-plan Work

Runtime07 still owns package resolution, catalog generations, native ABI conformance, trust and
isolation, capability policy, hot-reload leases, VM execution budgets, typed marshalling,
debugging, and stress/fault evidence. This slice only converges export validation report
diagnostic projection.
