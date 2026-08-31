---
title: Editor412 Borrowed Empty Source Window ID
category: zircon_editor
report_id: Editor412-borrowed-empty-source-window-id-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor412 Borrowed Empty Source Window ID

Editor callback-source resolution now borrows the native floating-window ID through blank-input
validation and creates an owned string only for a usable ID. Root-shell behavior and the returned
`MainPageId` remain unchanged, while rejected blank IDs no longer allocate or copy.

Regression coverage verifies native floating mode with a whitespace-only ID remains unresolved and
a valid ID retains its exact value. The ignored Windows Release benchmark emits
`EDITOR412_BORROWED_EMPTY_SOURCE_WINDOW_ID_BENCH_V1` over 17 alternating paired samples, a
256-byte blank ID, and 32,768 checks per sample. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.65` (at least 35% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor412 is prepared with Runtime482 under request
`runtime482-editor412-performance-batch-20260831fz-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
