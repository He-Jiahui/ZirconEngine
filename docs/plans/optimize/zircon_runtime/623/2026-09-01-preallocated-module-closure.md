---
title: Runtime623 Preallocated Module Closure
category: zircon_runtime
report_id: Runtime623-preallocated-module-closure-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime623 Preallocated Module Closure

Frozen runtime module graph activation and dependent closure queries now reserve their borrowed
membership sets from the corresponding graph-map node counts. Both queries previously grew an
empty hash set during transitive traversal even though the graph already knew its total node count.

Stack traversal, cycle suppression, missing-module diagnostics, and final stable activation-order
projection remain unchanged. Existing graph behavior coverage remains authoritative; focused
source coverage locks both capacities and borrowed insertion paths.

The ignored Windows Release benchmark emits `RUNTIME623_PREALLOCATED_MODULE_CLOSURE_BENCH_V1`
over 17 alternating sample pairs with 32,768 unique long module names. The gate requires
preallocated membership P95 to be at most 85% of unreserved membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime623 is prepared with Editor623 under request
`runtime623-editor623-module-template-import-performance-20260901im-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
