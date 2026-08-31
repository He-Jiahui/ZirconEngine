---
title: Editor391 Direct View Operation Path
category: zircon_editor
report_id: Editor391-direct-view-operation-path-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor391 Direct View Operation Path

Editor extension view descriptors now build their `view.<id>.open` operation path in one
exact-capacity string. Extension registration, command contribution, store batching, and workbench
menu projection no longer invoke generic formatting for this fixed path grammar.

View identifier bytes, prefix and suffix placement, parser validation, and error behavior remain
unchanged. Regression coverage compares representative core, nested, and plugin view identities with
the former formatter and guards the production source from regressing to `format!`.

The ignored Windows Release benchmark emits `EDITOR391_DIRECT_VIEW_OPERATION_PATH_BENCH_V1` over 17
alternating paired samples, each building 262,144 representative paths. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70` (at least 30% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor391 is prepared with Runtime461 under request
`runtime461-editor391-performance-batch-20260831fc-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
