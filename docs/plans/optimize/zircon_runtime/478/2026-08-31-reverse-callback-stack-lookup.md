---
title: Runtime478 Reverse Callback Stack Lookup
category: zircon_runtime
report_id: Runtime478-reverse-callback-stack-lookup-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime478 Reverse Callback Stack Lookup

Runtime wake registration now searches the thread-local active callback stack from newest to
oldest. Nested callbacks push their registration immediately before invoking the ABI callback, so
the current registration is normally the last entry. Reversing the equality scan preserves
membership semantics for every stack position while reducing the common top-of-stack hit from the
full callback depth to one comparison.

Regression coverage verifies top, interior, absent, and empty-stack queries. The ignored Windows
Release benchmark emits `RUNTIME478_REVERSE_CALLBACK_STACK_LOOKUP_BENCH_V1` over 17 alternating
paired samples, a 512-entry callback stack, and 16,384 top-entry lookups per sample. The modeled
common hit falls from 512 comparisons to one. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.25` (at least 75% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime478 is prepared with Editor408 under request
`runtime478-editor408-performance-batch-20260831fv-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
