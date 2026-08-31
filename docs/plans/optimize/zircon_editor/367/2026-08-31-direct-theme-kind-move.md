---
title: Editor367 Direct Theme Kind Move
category: zircon_editor
report_id: Editor367-direct-theme-kind-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor367 Direct Theme Kind Move

Theme-pane projection now evaluates whether the selected theme is locally editable before building
the output model, then moves `summary.selected_kind` directly into `selected_source_kind`. The old
field initializer cloned the owned string so it could be compared later, allocating a second string
for every theme-pane projection.

The projected kind text, local-theme comparison, promotion availability, draft projection, and all
other theme-pane fields remain unchanged. Regression coverage requires the comparison to remain
before the direct move and rejects restoration of the clone.

The ignored Windows Release benchmark emits `EDITOR367_DIRECT_THEME_KIND_MOVE_BENCH_V1` over 17
alternating paired samples. Each sample models 16,384 projections using the real `Local` kind text.
The legacy path performs 16,384 extra kind-string clones per sample and the optimized path performs
zero. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor367 is prepared with Runtime439 under request
`runtime439-editor367-performance-batch-20260831ee-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
