---
title: Runtime658 Batched Event Drain Diagnostics
category: zircon_runtime
report_id: Runtime658-batched-event-drain-diagnostics-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: admission_rejected_external_worktree_dirty
---

# Runtime658 Batched Event Drain Diagnostics

Subscriber deactivation drains its detached queue through one diagnostics operation. The operation
uses one queue-depth atomic update for the batch, captures one monotonic timestamp, aggregates age
samples locally, and publishes the aggregates once. This replaces one atomic compare/update and
one clock read per drained event while preserving saturating queue depth, the number of age
samples, total age, maximum age, and disconnect accounting.

The deterministic regression covers mixed timestamp availability and verifies the resulting queue
depth and age counters. A source guard binds the regression to the subscriber deactivation path.
The ignored Windows Release benchmark emits
`RUNTIME658_BATCHED_EVENT_DRAIN_DIAGNOSTICS_BENCH_V1` over 17 alternating sample pairs, 2,048
batches per sample, and 1,024 drained events per batch. Its gate requires batched P95 to be at most
20% of per-event P95.

Aggregate coordinator admission was rejected before immutable ticket creation because an external
Git worktree is dirty. No Cargo validation or performance measurement ran. The coordinator must
validate this Runtime candidate with other ready Runtime/Editor work after admission is available;
no accepted optimization record, commit, push, or WeCom publication exists.
