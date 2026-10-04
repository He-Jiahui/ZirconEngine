---
title: Runtime651 Preallocated Identity Changes
category: zircon_runtime
report_id: Runtime651-preallocated-identity-changes-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime651 Preallocated Identity Changes

Projected metadata inventory loading now reserves the identity-change vector from the collected
source count. Each source can produce at most one rename change, so the source count is a strict
upper bound while metadata order, URI comparison, and merged watcher-change ordering remain intact.

The ignored Windows Release benchmark emits `RUNTIME651_IDENTITY_CHANGE_CAPACITY_BENCH_V1` over 21
alternating sample pairs, 64 batches per sample, and 4,096 source entries per batch. The gate
requires reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime651/Editor651 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime651 is prepared with Editor651 under the shared `optimization_batch_jl_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
