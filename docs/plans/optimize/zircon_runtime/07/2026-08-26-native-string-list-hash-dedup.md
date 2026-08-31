---
title: Runtime07 Native String List Hash Dedup
category: zircon_runtime
report_id: Runtime07-native-string-list-hash-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Native String List Hash Dedup

## Scope

This slice removes repeated scans while native plugin capability and diagnostic-tag strings are
split, trimmed, and deduplicated. Delimiters, blank filtering, first-occurrence order, UTF-8 input,
ABI ownership, and caller-visible `Vec<String>` output remain unchanged. It advances Runtime07
native string projection without claiming completion of plugin trust, ABI compatibility,
isolation, capability negotiation, generation ownership, or fault containment.

## Change

- Maintain one borrowed `HashSet<&str>` over tokens in the immutable ABI input string.
- Allocate an owned `String` only when a token is accepted for the first time.
- Preserve the original first-occurrence output order and delimiter behavior.
- Keep the hash index local to parsing; no borrowed ABI data escapes the call.

## Deterministic Performance Evidence

| 4,096 distinct native string tokens | Before | After |
|---|---:|---:|
| Pairwise string comparisons | 8,386,560 | 0 |
| Token hash probes | 0 | 4,096 |
| Accepted `String` allocations | 4,096 | 4,096 |
| Output order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME07_NATIVE_STRING_LIST_HASH_DEDUP_BENCH_V1`. Acceptance requires hash deduplication P95 to
be at least 75% below the legacy repeated Vec scan. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bg_native_string_list_hash_dedup_preserves_first_order` covers mixed
  delimiters, trimming, duplicates, blanks, and first-occurrence order.
- `optimization_batch_20260826bg_native_string_list_hash_dedup_eliminates_pairwise_work` locks the
  8,386,560-comparison model and rejects the repeated Vec scan.
- `optimization_batch_20260826bg_native_string_list_hash_dedup_p95` reports paired release P50/P95
  samples and enforces the 75% P95 reduction gate.

## Remaining Parent-plan Work

Runtime07 still owns package resolution, catalog generations, native ABI conformance, trust and
isolation, capability policy, hot-reload leases, VM execution budgets, typed marshalling,
debugging, and stress/fault evidence. This slice only converges native string-list parsing.
