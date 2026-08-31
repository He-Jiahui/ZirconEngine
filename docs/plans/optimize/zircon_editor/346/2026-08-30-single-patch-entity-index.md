---
title: Editor346 Single Patch Entity Index
category: zircon_editor
report_id: Editor346-single-patch-entity-index-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor346 Single Patch Entity Index

Scene hierarchy patch validation now stores a `seen` bit beside each row reference in the entity
index. A single mutable lookup verifies the row anchor and marks the entity consumed; a repeated
anchor observes the existing bit and returns the same mismatch identity.

The previous path allocated both a row `HashMap` and an anchor `HashSet`, then hashed every entity
into both collections. The new path uses one map allocation and one lookup per anchor while
retaining row-count, duplicate-row, duplicate-anchor, missing-row, and field-mismatch checks.

The ignored Windows Release benchmark emits
`EDITOR346_SINGLE_PATCH_ENTITY_INDEX_BENCH_V1` over 17 alternating paired samples, each validating
16,384 one-to-one hierarchy rows and anchors, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor346 is prepared with Runtime418 under request
`runtime418-editor346-performance-batch-20260830dj-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
