---
title: Editor355 Theme Promotion Owned Style Imports
category: zircon_editor
report_id: Editor355-theme-promotion-owned-style-imports-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor355 Theme Promotion Owned Style Imports

Local-theme promotion now transfers the source document's style-import vector into the promoted
style asset and installs the new external reference in a fresh source vector. Imported style order,
theme contents, and the source document's resulting reference remain unchanged.

The previous path cloned every imported style string into the promoted document and then cleared
the original vector. `mem::take` removes the full-vector clone and reuses its allocation in the
promoted asset.

The ignored Windows Release benchmark emits
`EDITOR355_THEME_PROMOTION_OWNED_STYLE_IMPORTS_BENCH_V1` over 17 alternating paired samples, each
promoting 2,048 vectors containing 256 style references, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor355 is prepared with Runtime427 under request
`runtime427-editor355-performance-batch-20260830ds-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
