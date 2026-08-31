---
title: Runtime413 Plugin Feature Borrowed Admission
category: zircon_runtime
report_id: Runtime413-plugin-feature-borrowed-admission-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime413 Plugin Feature Borrowed Admission

Plugin render-feature admission now borrows the effective feature name and capability requirements
directly from `RendererFeatureAsset`. The previous path called `feature_name()` and `descriptor()`;
for descriptor-backed plugins that cloned the complete descriptor twice per admission, including
extract-section, history, pass, and graph-mutation storage. The new plugin branch performs zero
descriptor clones while preserving descriptor-override name precedence, disabled-feature lookup,
feature-local requirements, and descriptor requirements. Builtin admission behavior is unchanged.

The ignored Windows Release benchmark emits
`RUNTIME413_PLUGIN_FEATURE_BORROWED_ADMISSION_BENCH_V1` over 17 alternating paired samples, each
performing 512 admission checks against a descriptor with 128 extract sections, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime413 is prepared with Editor343 under request
`runtime413-editor343-performance-batch-20260830df-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
