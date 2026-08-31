---
title: Runtime414 Texture Descriptor Single Projection
category: zircon_runtime
report_id: Runtime414-texture-descriptor-single-projection-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime414 Texture Descriptor Single Projection

RGBA8 texture upload admission now accepts a caller-owned `RenderImageDescriptor` inside the
crate. The public standalone readiness entry still derives its own descriptor, but uses that one
projection for both shape validation and mip/format validation. Resource-streamer texture
resolution reuses the descriptor it already derives for material-dimension validation when it
checks upload readiness.

The previous resource-streamer path materialized the complete descriptor three times per ready
RGBA8 reference: once for material dimension, once for upload-shape validation, and once for
upload mip/format validation. Each projection clones descriptor `String` and `Vec` storage. The
new path materializes it once. Container upload behavior and public API behavior are unchanged.

The ignored Windows Release benchmark emits
`RUNTIME414_TEXTURE_DESCRIPTOR_SINGLE_PROJECTION_BENCH_V1` over 17 alternating paired samples,
each performing 1,024 admissions with 256 usage and 256 asset-usage entries, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime414 is prepared with Runtime415 under request
`runtime414-runtime415-performance-batch-20260830dg-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
