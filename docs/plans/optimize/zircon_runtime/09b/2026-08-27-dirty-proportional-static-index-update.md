---
title: Runtime09B Dirty-Proportional Static Index Update
category: zircon_runtime
report_id: Runtime09B-dirty-proportional-static-index-update-2026-08-27
date: 2026-08-27
session_id: root-runtime09b-dirty-proportional-static-index-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Dirty-Proportional Static Index Update

## Scope

This slice removes the full-scene ordered lookup map from incremental visibility static-index
updates. It directly advances Runtime09B P1-5 without changing the persistent grid, snapshot COW,
query containers, or the public report schema.

## Change

- Allocate a temporary hash lookup only for the K inserted and updated stable-instance keys.
- Scan the current N-instance slice once and retain only matching rows.
- Preserve plan traversal order, missing-key removal, and the previous last-duplicate-row behavior.
- Skip the current-instance scan entirely when K is zero.

The update remains O(N + K) because the current input is still a flat slice, but temporary index
storage falls from O(N) to O(K). Replacing the flat scan with a persistent render-scene slot table
remains parent-plan work.

## Deterministic Performance Evidence

Independent optimized Rust 1.94.1 model, 65,536 instances, 128 changed keys, 4 repetitions and 21
alternating samples:

| Metric | Full-scene `BTreeMap` | Changed-key `HashMap` | Reduction |
|---|---:|---:|---:|
| allocations | 23,852 | 4 | 99.98% |
| allocated bytes | 13,158,272 | 17,472 | 99.87% |
| P50 | 13,400,500 ns | 4,304,800 ns | 67.9% |
| P95 | 23,701,700 ns | 5,638,900 ns | 76.2% |

The executable gate requires at least 95% fewer temporary allocated bytes and at least 40% lower
P95. The stable checksum is `17596834528452050169`.

## Acceptance

- The Rust regression proves that a duplicate current key still resolves to its last row.
- The Python source contract rejects reconstruction of a full-scene ordered map, requires one
  current-slice scan, and checks the model thresholds.
- The independent model emits `RUNTIME09B_DIRTY_PROPORTIONAL_STATIC_INDEX_MODEL_V1` and enforces
  the allocation-byte and P95 targets.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, source contracts, focused Rust tests, and
  model execution are submitted together through one managed validation request.

## Remaining Parent-Plan Work

The uniform grid still uses tree containers, query paths still materialize candidates, and COW can
copy whole maps while snapshots are live. Persistent dense slots, paged spatial ownership, reused
query scratch, and true dirty-only lookup remain open Runtime09B work.
