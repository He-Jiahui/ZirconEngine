---
title: Editor344 Scene Mode Borrowed Owner Boundary
category: zircon_editor
report_id: Editor344-scene-mode-borrowed-owner-boundary-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor344 Scene Mode Borrowed Owner Boundary

Isolated scene-mode callbacks now borrow the mode owner id across the plugin panic boundary.
`run_with_ctx` splits immutable `owner_id` and mutable `inner` field borrows before entering the
callback, preserving checkpoint restoration, overlay invalidation, fault recording, and callback
ordering. Inner-id validation and guarded destruction use the same borrowed owner path.

The previous implementation cloned the complete owner `String` before every input, update, enter,
and exit callback solely to satisfy whole-`self` borrowing. A normal successful callback therefore
allocated and copied an owner id that the synchronous boundary only reads. The new path performs
zero owner-id allocations per callback.

The ignored Windows Release benchmark emits
`EDITOR344_SCENE_MODE_BORROWED_OWNER_BOUNDARY_BENCH_V1` over 17 alternating paired samples, each
executing 65,536 successful plugin boundaries with a representative qualified owner id, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor344 is prepared with Runtime416 under request
`editor344-runtime416-performance-batch-20260830dh-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
