---
title: Runtime54 Event Mirror Reader Count Hash Index
category: zircon_runtime
report_id: Runtime54-event-mirror-reader-count-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: efb97b91c4db45569d092ef412235536
---

# Runtime54 Event Mirror Reader Count Hash Index

## Scope

This slice replaces the scene event-mirror reader-count owner with `HashMap`. Subscribe,
unsubscribe, rollback, and lifecycle diagnostics now resolve event IDs through expected
constant-time lookup.

The registration owner remains a `BTreeMap`, preserving deterministic event-ID and clone traversal.
The Debug implementation projects reader counts through that registration order without cloning
event IDs. Counter overflow and underflow errors, per-event isolation, callbacks, subscription
slots, and reclaim behavior are unchanged.

## Performance Workload

The release workload fills 4,096 long shared-prefix event IDs and performs 4,096 stable hits for
the final reader-count key.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered reader-count lookups | 4,096 | 0 |
| Hash reader-count lookups | 0 | 4,096 |
| Registration-order policy changes | 0 | 0 |
| Allocations on reader-count hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME54_EVENT_MIRROR_READER_COUNT_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to
be at least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bz_event_mirror_reader_count_hash_index_isolates_events` covers
  independent increments, decrement, diagnostics, and deterministic Debug projection.
- `optimization_batch_20260826bz_event_mirror_reader_count_hash_index_preserves_registration_order_owner`
  locks the split hash-reader/tree-registration contract.
- `optimization_batch_20260826bz_event_mirror_reader_count_hash_index_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime54 still owns acknowledged cursors, gap and resync semantics, shared encoding, global queue
budgets, AI event exposure, consumer commit authority, and product integration. This slice only
converges per-event reader-count lookup.

## Current-source recovery batch

This task shares one managed ticket with Runtime52 dynamic-scene hash validation. The exclusive
`runtime_hash_recovery_batch_` filter runs four ordinary regressions and two ignored release P95
gates in two Cargo invocations; queue admission is not timing evidence.
The ticket is sealed against snapshot `2450` and source manifest
`c185bdebb641bd095aae18cfa4f624615f281cb8fa2f1e79d1c79d324dfa462c`.
