---
title: Runtime477 Single Dependency Dedupe
category: zircon_runtime
report_id: Runtime477-single-dependency-dedupe-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime477 Single Dependency Dedupe

Asset registry dependency normalization now returns zero- and one-element dependency vectors
directly. The hash set remains for multi-dependency inputs, preserving first-seen ordering and
duplicate removal while avoiding a hash allocation in the common single-dependency case.

Regression coverage verifies one-element capacity and value preservation. The ignored Windows
Release benchmark emits `RUNTIME477_SINGLE_DEPENDENCY_ALLOCATION_BENCH_V1` over 17 alternating
paired samples and 65,536 single-dependency normalizations per sample. The common path removes one
hash-set allocation per dependency normalization. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime477 is prepared with Editor407 under request
`runtime477-editor407-performance-batch-20260831fu-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
