---
title: Editor341 Builtin Descriptor Retain
category: zircon_editor
report_id: Editor341-builtin-descriptor-retain-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor341 Builtin Descriptor Retain

Builtin view registration now applies required capabilities in place and retains allowed entries in
the existing candidate vector. Descriptor order, capability assignment, disabled-subsystem
filtering, and the public compatibility helper remain unchanged while startup avoids allocating and
populating a second descriptor vector.

The ignored Windows Release benchmark emits `EDITOR341_BUILTIN_DESCRIPTOR_RETAIN_BENCH_V1` over 17
alternating paired samples using 64 batches of 4,096 descriptors, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor341 is submitted with Editor340 under request
`editor340-editor341-performance-batch-20260830cx-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
