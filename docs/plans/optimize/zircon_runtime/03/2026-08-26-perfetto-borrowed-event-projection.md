---
title: Runtime03 Perfetto Borrowed Event Projection
category: zircon_runtime
report_id: Runtime03-perfetto-borrowed-event-projection-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 Perfetto Borrowed Event Projection

## Scope

This slice reduces transient allocation while projecting an already sealed profile snapshot into
the Perfetto JSON artifact. It supports the Runtime03 ownership of CPU trace, snapshot, export, and
per-event allocation qualification. It does not close capture-generation, thread identity,
artifact atomicity, manifest, pagination, or asynchronous export requirements in the parent plan.

## Change

- `PerfettoEvent` borrows event name, category, stream, and optional path from `ProfileSnapshot`.
- Strongly typed untagged argument variants replace per-event `serde_json::Value` construction
  while preserving the existing Perfetto JSON object shape.
- The event vector reserves the exact frame + span + counter count before projection.
- File serialization remains inside the existing export boundary; no retained cache or lifetime
  extension was introduced.

## Deterministic Performance Evidence

| 32,768-span projection | Before | After | Reduction |
|---|---:|---:|---:|
| Owned event text/argument fields | 131,072 | 0 | 100% removed |
| Event text bytes copied | 1,310,720 | 0 | 100% removed |
| Dynamic JSON argument objects | 32,768 | 0 | 100% removed |
| Event-vector capacity | incremental growth from empty | exact 32,768 | exact reservation |

The ignored release gate alternates 17 legacy-owned and borrowed projection samples. It emits
`RUNTIME03_PERFETTO_BORROWED_EVENT_PROJECTION_BENCH_V1`; acceptance requires borrowed P95 to be at
most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826f_runtime03_perfetto_projection_preserves_json_shape` covers frame,
  span, and counter field compatibility.
- `optimization_batch_20260826f_runtime03_perfetto_projection_borrows_event_text` rejects string
  cloning, dynamic JSON construction, and an unreserved event vector in the projection.
- `optimization_batch_20260826f_runtime03_perfetto_projection_performance_evidence` publishes the
  release P95 marker and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and the two source contracts must pass
  before managed validation submission.

## Remaining Parent-plan Work

The recorder remains process-global, snapshot rows still deep-clone under recorder ownership,
Perfetto thread identity is still stream-based, and artifact publication is not yet atomic or
manifested. Those are separate Runtime03 milestones.
