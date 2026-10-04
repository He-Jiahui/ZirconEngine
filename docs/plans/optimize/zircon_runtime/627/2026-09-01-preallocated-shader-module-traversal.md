---
title: Runtime627 Preallocated Shader Module Traversal
category: zircon_runtime
report_id: Runtime627-preallocated-shader-module-traversal-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime627 Preallocated Shader Module Traversal

Shader module root closure construction now reserves visited and registry indexes from the source
module plus root counts. Registry resolution also reserves its visited set and ordered output from
the immutable module table upper bound. These structures previously grew from zero during every
module closure traversal.

Dependency DFS order, cycle diagnostics, unknown-module errors, generated-module filtering, and
content hashing remain unchanged. The capacity changes do not alter shader module identity or the
resolved source sequence.

The ignored Windows Release benchmark emits
`RUNTIME627_PREALLOCATED_SHADER_MODULE_TRAVERSAL_BENCH_V1` over 17 alternating sample pairs with
32,768 unique long module tokens. The gate requires preallocated traversal P95 to be at most 85% of
the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime627/Editor627 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime627 is prepared with Editor627 under request
`runtime627-editor627-shader-faulted-index-performance-20260901iq-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
