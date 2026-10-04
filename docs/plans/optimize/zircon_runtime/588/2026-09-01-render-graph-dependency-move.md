---
title: Runtime588 Render Graph Dependency Move
category: zircon_runtime
report_id: Runtime588-render-graph-dependency-move-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime588 Render Graph Dependency Move

Render-graph compilation now moves each pass's manual dependency vector out of the consumed
`RenderGraphBuilder`. The former path cloned every vector before dependency completion and then
dropped the original vectors with the builder. Resource/version validation and the two existing
execution/provenance dependency graphs remain unchanged.

Error ordering is preserved because all fallible builder validation still runs before extraction.
Existing graph coverage verifies that manual dependencies keep live passes, while a focused source
guard requires the consuming path and rejects restoration of the initial clone.

The ignored Windows Release benchmark emits `RUNTIME588_RENDER_GRAPH_DEPENDENCY_MOVE_BENCH_V1`
over 21 alternating sample pairs, 8,192 passes, and 24 dependencies per pass. The legacy model
clones 8,192 dependency vectors per sample; the optimized model moves them without vector-buffer
allocation. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.55`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime588 is prepared with Editor588 under request
`runtime588-editor588-dependency-pane-performance-20260901hl-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
