---
title: Editor393 Direct World Space UI And Variant Strings
category: zircon_editor
report_id: Editor393-direct-world-space-ui-and-variant-strings-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor393 Direct World Space UI And Variant Strings

World-space UI pointer statuses now append their fixed message prefix and control ID into one
preallocated string. Table layout context variants likewise append the trimmed existing variant and
layout token directly, preserving duplicate-token and empty-variant behavior.

Regression coverage keeps all pointer event message bytes, `Move` suppression, table/non-table
selection, whitespace trimming, and duplicate-token behavior unchanged. The ignored Windows Release
benchmarks emit `EDITOR393_DIRECT_WORLD_SPACE_STATUS_BENCH_V1` and
`EDITOR393_DIRECT_VARIANT_APPEND_BENCH_V1`, each over 17 alternating paired samples and 262,144
iterations. Each gate requires at least 20% lower optimized P95 than the former formatter.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor393 is prepared with Runtime463 under request
`runtime463-editor393-performance-batch-20260831fg-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
