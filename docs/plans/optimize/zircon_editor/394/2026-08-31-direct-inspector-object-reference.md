---
title: Editor394 Direct Inspector Object Reference
category: zircon_editor
report_id: Editor394-direct-inspector-object-reference-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor394 Direct Inspector Object Reference

Inspector object drag payloads now append their `u64` scene node identity into one preallocated
reference string. Payload kind, locator, display name, source surface, control identity, and exact
`object://scene/node/<id>` bytes remain unchanged while fixed-prefix formatting is removed from drag
startup.

Regression coverage compares zero, ordinary, and maximum node identities with the former formatter.
The ignored Windows Release benchmark emits `EDITOR394_DIRECT_OBJECT_REFERENCE_BENCH_V1` over 17
alternating paired samples, each building 262,144 references. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor394 is prepared with Runtime464 under request
`runtime464-editor394-performance-batch-20260831fh-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
