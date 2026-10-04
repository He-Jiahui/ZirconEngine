---
title: Runtime634 Preallocated Shader Replacement Paths
category: zircon_runtime
report_id: Runtime634-preallocated-shader-replacement-paths-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime634 Preallocated Shader Replacement Paths

Targeted shader source replacement now reserves affected import-path membership from the removed
shader count plus the ready-shader iterator bound. The previous hash set grew from zero while both
old include owners and new include providers were collected.

Unknown ready-iterator upper bounds fall back to their lower bounds, and saturated addition keeps
capacity calculation defined. Shader removal, insertion, consumer expansion, and affected-ID
semantics remain unchanged.

The ignored Windows Release benchmark emits
`RUNTIME634_PREALLOCATED_SHADER_REPLACEMENT_PATH_BENCH_V1` over 17 alternating sample pairs with
32,768 removed and 32,768 ready paths. The gate requires preallocated P95 to be at most 80% of the
unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime634/Editor634 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime634 is prepared with Editor634 under request
`runtime634-editor634-shader-view-capacity-performance-20260901ix-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
