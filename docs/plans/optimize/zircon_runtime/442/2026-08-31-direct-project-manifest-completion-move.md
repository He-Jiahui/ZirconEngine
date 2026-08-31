---
title: Runtime442 Direct Project Manifest Completion Move
category: zircon_runtime
report_id: Runtime442-direct-project-manifest-completion-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime442 Direct Project Manifest Completion Move

Runtime plugin catalog construction now moves its newly assembled project manifest directly into
the completion pipeline. The old path borrowed that temporary manifest and immediately deep-cloned
all project selections before applying selection defaults and feature completion.

The existing borrowed completion entry point still clones caller-owned manifests, preserving its
API and isolation semantics. Registration order, default selection completion, feature selection
completion, and returned manifest contents remain unchanged. Regression coverage requires the new
catalog path to use the owned helper and keeps the borrowed wrapper clone explicit.

The ignored Windows Release benchmark emits
`RUNTIME442_DIRECT_PROJECT_MANIFEST_COMPLETION_MOVE_BENCH_V1` over 17 alternating paired samples.
Each sample completes 64 manifests containing 512 long selection records. Both models pay the
common input construction copy; the legacy model performs another 32,768 selection copies per
sample and the optimized model performs none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime442 is prepared with Editor370 under request
`runtime442-editor370-performance-batch-20260831eh-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
