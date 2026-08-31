---
title: Editor392 Direct RGBA Resource Key
category: zircon_editor
report_id: Editor392-direct-rgba-resource-key-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor392 Direct RGBA Resource Key

Retained image recording now builds generated `rgba:<width>x<height>:<hash>` resource keys with one
preallocated string, direct decimal dimensions, and fixed-width lowercase hexadecimal output. The
recording path no longer invokes generic formatting for generated image identities.

Hash inputs, decimal width boundaries, fixed 16-digit hash casing, and the key grammar remain
unchanged. Regression coverage compares empty, small, large, and maximum dimensions with the former
formatter.

The ignored Windows Release benchmark emits `EDITOR392_DIRECT_RGBA_RESOURCE_KEY_BENCH_V1` over 17
alternating paired samples, each building 262,144 representative keys. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor392 is prepared with Runtime462 under request
`runtime462-editor392-performance-batch-20260831fd-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
