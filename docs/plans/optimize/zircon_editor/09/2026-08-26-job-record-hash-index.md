---
title: Editor09 Job Record Hash Index
category: zircon_editor
report_id: Editor09-job-record-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor09 Job Record Hash Index

## Scope

This slice replaces the unordered `JobId -> record` and `JobId -> terminal order` owners with
`HashMap`. Dependency admission, scheduler-handle resolution, terminal marking, pin/unpin checks,
and bounded history pruning now use expected constant-time JobId lookup.

Terminal publication and eviction order remain owned by the existing
`BTreeSet<(terminal_order, JobId)>` indexes. Category counts and mutex-group tails retain their
existing owners. The 256-record retention limit, pending dependency pinning, completion-handle
semantics, cancellation, and shutdown behavior are unchanged.

## Performance Workload

The release workload fills 1,024 JobId records and performs 4,096 stable hits for the final ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered JobId-record lookups | 4,096 | 0 |
| Hash JobId-record lookups | 0 | 4,096 |
| Terminal ordering-policy changes | 0 | 0 |
| Allocations on record hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR09_JOB_RECORD_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at least 30%
below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bv_job_record_hash_index_preserves_terminal_order` covers dependency
  lookup and unchanged terminal publication sequence.
- `optimization_batch_20260826bv_job_record_hash_index_keeps_ordered_eviction_sets` locks hash
  record ownership and ordered terminal/eviction ownership.
- `optimization_batch_20260826bv_job_record_hash_index_p95` reports paired release P50/P95 samples
  and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor09 still owns cancellation-token authority, deadline policy, dependency outcomes, bounded
results, telemetry, and product integration. This slice only converges internal JobId state
lookup.
