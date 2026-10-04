---
title: Runtime642 Hashed Shader Resource Deduplication
category: zircon_runtime
report_id: Runtime642-hashed-shader-resource-deduplication-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime642 Hashed Shader Resource Deduplication

Shader resource-record deduplication now uses two input-sized hash indexes for resource IDs and
locators instead of ordered trees. Duplicate detection still follows input order and reports the
same conflicting pair. The accepted records are still explicitly sorted by locator and ID before
return, so exported order remains deterministic while the identity checks move from `O(n log n)`
to expected `O(n)`.

The ignored Windows Release benchmark emits `RUNTIME642_HASHED_SHADER_RESOURCE_DEDUP_BENCH_V1`
over 17 alternating sample pairs with 32,768 unique shader records. It compares the former two-tree
deduplication with the two-hash-index path while retaining the same final sort. The gate requires
hashed P95 to be at most 80% of ordered-index P95.

No direct Cargo validation was run. The coordinator owns combined Runtime642/Editor642 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime642 is prepared with Editor642 under the shared `optimization_batch_jc_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
