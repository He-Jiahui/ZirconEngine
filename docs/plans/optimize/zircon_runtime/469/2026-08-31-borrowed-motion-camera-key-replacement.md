---
title: Runtime469 Borrowed Motion Camera Key Replacement
category: zircon_runtime
report_id: Runtime469-borrowed-motion-camera-key-replacement-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime469 Borrowed Motion Camera Key Replacement

Per-frame motion-vector camera history replacement now borrows the viewport camera key while
probing the existing map entry. The common replacement path updates the camera in place and avoids
cloning the key; only the first insertion clones ownership into the map. Keys whose culling and
volume layer sets use shared storage therefore avoid two `Arc` increments and two decrements on
every subsequent frame.

Regression coverage verifies that two updates retain one map entry and expose the latest camera.
The ignored Windows Release benchmark emits `RUNTIME469_BORROWED_MOTION_CAMERA_KEY_BENCH_V1` over
17 alternating paired samples, each replacing one wide-layer camera key 262,144 times. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.85` (at least 15% lower P95) and records the
reduction from 1,048,576 legacy `Arc` reference-count operations per sample to zero on the existing
key path.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime469 is prepared with Editor399 under request
`runtime469-editor399-performance-batch-20260831fm-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
