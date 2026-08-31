---
title: Runtime11C Icon Request Hash Dedup
category: zircon_runtime
report_id: Runtime11C-icon-request-hash-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C Icon Request Hash Dedup

## Scope

This slice replaces the icon-atlas request deduplication tree with `HashMap`. Duplicate icon IDs
still keep the first request. Atlas planning still explicitly sorts pending slots by icon ID and
semantic ID, so atlas dimensions, slot order, UV placement, and deterministic plan output are
unchanged.

The removed tree ordering was redundant: its values were collected and then sorted again before
layout. SVG parsing, pixel-size calculation, padding, page limits, and raster asset semantics are
unchanged.

## Performance Workload

The release workload admits 8,192 requests over 4,096 long shared-prefix icon IDs, with the second
half duplicating the first in reverse order.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered dedup admissions | 8,192 | 0 |
| Hash dedup admissions | 0 | 8,192 |
| Final explicit slot sorts | 1 | 1 |
| Duplicate winner changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME11C_ICON_REQUEST_HASH_DEDUP_BENCH_V1`. Acceptance requires hash dedup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826cb_icon_request_hash_dedup_preserves_first_and_slot_order` covers
  first-request ownership and deterministic slot order.
- `optimization_batch_20260826cb_icon_request_hash_dedup_keeps_explicit_slot_sort` locks the hash
  admission plus explicit output-order contract.
- `optimization_batch_20260826cb_icon_request_hash_dedup_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime11C still owns production icon rasterization, GPU page upload, device/session lifetime,
generation invalidation, eviction, and the renderer consumer. This slice only removes redundant
ordered request deduplication.
