---
title: Runtime626 Preallocated GPU Scene Sync Indexes
category: zircon_runtime
report_id: Runtime626-preallocated-gpu-scene-sync-indexes-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime626 Preallocated GPU Scene Sync Indexes

GPU Scene pending-draw synchronization now reserves both the live stable-key set and synced-entry
map from the pending draw slice length. These two indexes previously grew independently from zero
on every synchronization pass.

Pending draw order, GPU Scene registration, retained-key publication, per-entry payload generation,
and returned entry identity remain unchanged. Focused source coverage locks the common capacity for
both indexes without touching the GPU ABI or resource lifecycle.

The ignored Windows Release benchmark emits `RUNTIME626_PREALLOCATED_GPU_SCENE_SYNC_BENCH_V1` over
17 alternating sample pairs with 65,536 unique draw keys. The gate requires preallocated index P95
to be at most 85% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime626/Editor626 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime626 is prepared with Editor626 under request
`runtime626-editor626-gpu-plugin-snapshot-performance-20260901ip-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
