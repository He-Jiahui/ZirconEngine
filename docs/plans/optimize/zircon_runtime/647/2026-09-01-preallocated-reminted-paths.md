---
title: Runtime647 Preallocated Reminted Paths
category: zircon_runtime
report_id: Runtime647-preallocated-reminted-paths-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime647 Preallocated Reminted Paths

Duplicate-GUID normalization now reserves its reminted-path output from the scanned metadata count.
Each scanned document can append at most one root path, so this is a strict upper bound that removes
geometric growth during large repair passes without changing scan order, diagnostics, save behavior,
or returned paths.

The ignored Windows Release benchmark emits `RUNTIME647_REMINTED_PATH_CAPACITY_BENCH_V1` over 17
alternating sample pairs with 65,536 scanned metadata items and a 50% remint rate. The gate requires
reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime647/Editor647 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime647 is prepared with Editor647 under the shared `optimization_batch_jh_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
