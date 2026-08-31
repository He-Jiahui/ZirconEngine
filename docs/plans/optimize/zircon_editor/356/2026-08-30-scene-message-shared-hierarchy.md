---
title: Editor356 Scene Message Shared Hierarchy
category: zircon_editor
report_id: Editor356-scene-message-shared-hierarchy-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor356 Scene Message Shared Hierarchy

Scene-inspection messages now store added anchors, changed anchors, and removed entity identifiers
in shared immutable slices. Constructors still accept owned vectors, accessors still expose slices,
and serde preserves the existing array representation.

Latest-message coalescing clones the current editor message before changing only its selection
delta. The previous `Vec` fields therefore deep-copied every hierarchy entry; `Arc<[T]>` reduces
those unrelated copies to reference-count increments.

The ignored Windows Release benchmark emits `EDITOR356_SCENE_MESSAGE_SHARED_HIERARCHY_BENCH_V1`
over 17 alternating paired samples, each cloning a 512-anchor payload 4,096 times, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor356 is prepared with Runtime428 under request
`runtime428-editor356-performance-batch-20260830dt-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
