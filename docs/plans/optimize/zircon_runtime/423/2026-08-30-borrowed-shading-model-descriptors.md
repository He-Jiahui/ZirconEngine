---
title: Runtime423 Borrowed Shading Model Descriptors
category: zircon_runtime
report_id: Runtime423-borrowed-shading-model-descriptors-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime423 Borrowed Shading Model Descriptors

Resource-streamer shader include source construction now passes the shading-model registry's
descriptor iterator directly. The existing slice-based `from_project_asset_manager` API remains
available for callers that already own a descriptor slice, while the iterator entry point avoids
cloning every descriptor into a temporary `Vec` on each plugin pipeline source request.

Descriptor filtering, include resolution, runtime-owned include handling, and error behavior are
unchanged. The ignored Windows Release benchmark emits
`RUNTIME423_BORROWED_SHADING_MODEL_DESCRIPTORS_BENCH_V1` over 17 alternating paired samples of 48
descriptors and 8,192 snapshots per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime423 is prepared with Editor351 under request
`runtime423-editor351-performance-batch-20260830do-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
