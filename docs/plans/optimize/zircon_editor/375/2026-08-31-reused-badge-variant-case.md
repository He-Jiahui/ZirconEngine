---
title: Editor375 Reused Badge Variant Case
category: zircon_editor
report_id: Editor375-reused-badge-variant-case-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor375 Reused Badge Variant Case

Editor retained pane badge projection now computes PascalCase forms of overlap, vertical origin, and
horizontal origin once per node, then reuses them across the overlap and two anchor-origin tokens.
The former path converted each value again for the extended anchor token, performing six conversions
where three are sufficient.

Variant token content and order, badge visibility, color, overlap defaults, anchor defaults, and
authored attribute precedence remain unchanged. Regression coverage compares the optimized token
stream with the former implementation and requires one conversion per derived value.

The ignored Windows Release benchmark emits `EDITOR375_REUSED_BADGE_VARIANT_CASE_BENCH_V1` over 17
alternating paired samples. Each sample projects 4,096 long geometry variant inputs into a
preallocated output string. The legacy model performs six PascalCase conversions per projection and
the optimized model performs three. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor375 is prepared with Runtime447 under request
`runtime447-editor375-performance-batch-20260831em-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
