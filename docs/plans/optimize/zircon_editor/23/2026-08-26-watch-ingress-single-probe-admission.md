---
title: Editor23 Watch Ingress Single-Probe Admission
category: zircon_editor
report_id: Editor23-watch-ingress-single-probe-admission-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Watch Ingress Single-Probe Admission

## Scope

This slice removes the duplicate hash lookup used to admit UI asset watcher paths. FIFO order,
duplicate coalescing, bounded capacity, overflow reconciliation, drained-path restoration, oldest
age, and ingress counters remain unchanged. It advances the Editor23 UI asset watcher hot path
without claiming completion of revision-qualified save, cross-asset transactions, external-change
conflict resolution, or product-scale soak gates.

## Change

- Replace the parallel `HashSet<PathBuf>` membership store with a `HashMap<PathBuf, ()>` so the
  stable entry API can classify occupied and vacant paths with one hash probe.
- Move the incoming path into the entry and clone it only when a vacant path is actually queued;
  duplicate events do not acquire a new path-buffer copy.
- Apply the same entry admission to callback-restored paths while retaining front order and
  overflow behavior.
- Add test-only probe accounting plus split behavior, source-contract, and ignored P95 tests.

## Deterministic Performance Evidence

| 8,192 unique watcher paths | Before | After |
|---|---:|---:|
| Admission hash probes | 16,384 | 8,192 |
| Duplicate-event admission hash probes per event | 1 | 1 |
| Full-queue unique admission hash probes per event | 1 | 1 |
| Queue order or capacity changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_WATCH_INGRESS_SINGLE_PROBE_BENCH_V1`. Acceptance requires single-probe admission P95 to
be at least 25% below the legacy contains-then-insert kernel. Exact Windows timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826bd_watch_ingress_single_probe_preserves_queue_semantics` covers
  unique admission, duplicate coalescing, overflow, counters, and FIFO drain order.
- `optimization_batch_20260826bd_watch_ingress_single_probe_eliminates_duplicate_lookup` requires
  one probe per unique or duplicate event and rejects `contains` in the production admission loop.
- `optimization_batch_20260826bd_watch_ingress_single_probe_p95` reports paired release P50/P95
  samples and enforces the 25% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns lossless V2 documents, revision CAS, transactional multi-file mutations,
watcher conflict policy, preview/device/input/accessibility authoring, and product-scale fault and
soak evidence. This slice only converges UI asset watcher path admission.
