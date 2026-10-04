---
title: Runtime655 Preallocated Material Control Validation
category: zircon_runtime
report_id: Runtime655-preallocated-material-control-validation-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: retracted_after_review
validation_status: not_submitted
---

# Runtime655 Preallocated Material Control Validation

This proposal was retracted during source review: allocating a heap buffer for every successful
material validation would regress the common no-error path. No source or test remains attached to
this plan.

Material-control validation now allocates its error buffer for the fixed set of 14 validators
before executing them. Validation order, diagnostic variants, property paths, and material values
remain unchanged; the change removes geometric `Vec` growth when several invalid controls are
reported together.

The ignored Windows Release benchmark emits
`RUNTIME655_MATERIAL_CONTROL_VALIDATION_CAPACITY_BENCH_V1` over 17 alternating sample pairs and
4,096 complete validator batches. The gate requires reserved P95 to be at most 80% of unreserved
P95 and requires zero reserved capacity growths.

No direct Cargo validation was run. The coordinator owns combined Runtime655/Editor655 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.
