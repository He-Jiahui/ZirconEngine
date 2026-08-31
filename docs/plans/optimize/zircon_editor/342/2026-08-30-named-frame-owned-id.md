---
title: Editor342 Named Frame Owned ID
category: zircon_editor
report_id: Editor342-named-frame-owned-id-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor342 Named Frame Owned ID

Editor profiling geometry now accepts owned or borrowed named-frame strings through `Into<String>`.
Activity-rail, template-node, and surface-frame control IDs move their formatted string directly
into the profile record, eliminating the second allocation and copy previously caused by borrowing
the formatted ID and immediately calling `to_string()`.

The ignored Windows Release benchmark emits `EDITOR342_NAMED_FRAME_OWNED_ID_BENCH_V1` over 17
alternating paired samples with 16,384 dynamic IDs, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`. The ownership regression test also verifies that the
formatted `String` buffer is retained by the emitted frame.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor342 is submitted with Runtime412 under request
`runtime412-editor342-performance-batch-20260830de-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
