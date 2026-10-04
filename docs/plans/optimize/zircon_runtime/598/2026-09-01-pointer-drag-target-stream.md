---
title: Runtime598 Pointer Drag Target Stream
category: zircon_runtime
report_id: Runtime598-pointer-drag-target-stream-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime598 Pointer Drag Target Stream

Picking drag enter and leave dispatch now iterates the active drag map directly while emitting
events. The previous implementation copied every dragged target into a temporary `Vec` before
emitting the same ordered events. BTreeMap key order, hover bookkeeping, event payloads, and
pointer/button routing are unchanged; only the transient target snapshot allocation is removed.

Focused tests cover multiple dragged targets and preserve their deterministic order for both enter
and leave transitions. A source guard requires both hot paths to stream `dragging.keys()` without a
temporary collection. The ignored Windows Release benchmark emits
`RUNTIME598_DRAG_TARGET_STREAM_BENCH_V1` over 17 alternating sample pairs and 32,768 targets. The
gate requires streamed P95 to be at most 90% of the legacy snapshot path.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime598 is prepared with Editor598 under request
`runtime598-editor598-drag-capability-performance-20260901hq-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
