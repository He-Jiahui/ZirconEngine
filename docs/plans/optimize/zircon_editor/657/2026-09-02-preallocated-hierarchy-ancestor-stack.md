---
title: Editor657 Preallocated Hierarchy Ancestor Stack
category: zircon_editor
report_id: Editor657-preallocated-hierarchy-ancestor-stack-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: retracted_after_review
validation_status: not_submitted
---

# Editor657 Preallocated Hierarchy Ancestor Stack

This proposal was retracted during source review: reserving the full row count over-allocates the
usual shallow hierarchy and does not establish a useful product-level win. No source or test
remains attached to this plan.

Hierarchy parent projection now reserves its ancestor stack from the source row count. The stack
can contain each row at most once, so the bound is strict. Parent selection, depth popping,
reverse ancestor propagation, selection overlay preservation, and telemetry remain unchanged.

The ignored Windows Release benchmark emits `EDITOR657_HIERARCHY_ANCESTOR_STACK_CAPACITY_BENCH_V1`
over 17 alternating sample pairs and a 16,384-row deep hierarchy. The gate requires reserved P95
to be at most 80% of unreserved P95 and requires zero reserved capacity growths.

No direct Cargo validation was run. The coordinator owns combined Runtime657/Editor657 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.
