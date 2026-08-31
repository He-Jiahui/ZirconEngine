---
title: Runtime585 Shader Validation ID Borrow
category: zircon_runtime
report_id: Runtime585-shader-validation-id-borrow-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime585 Shader Validation ID Borrow

The per-batch WGPU shader-module validation cache now indexes borrowed content-addressed source IDs
from the immutable prewarm manifest. The former miss path cloned each 64-byte hexadecimal ID into
an owned hash-map key. Success and failure results are still cached by source identity and returned
without repeating validation; all four module-validation entry points now register the manifest
source table when creating the cache.

Regression coverage verifies that the cached key is the manifest-owned ID and that its result slot
is populated. The ignored Windows Release benchmark emits
`RUNTIME585_BORROWED_MODULE_VALIDATION_IDS_BENCH_V1` over 21 alternating sample pairs and 16,384
unique sources per sample. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.60`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime585 is prepared with Editor585 under request
`runtime585-editor585-shader-grid-performance-20260831hd-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
