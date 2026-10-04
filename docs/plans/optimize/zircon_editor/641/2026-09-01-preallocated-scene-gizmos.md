---
title: Editor641 Preallocated Scene Gizmos
category: zircon_editor
report_id: Editor641-preallocated-scene-gizmos-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor641 Preallocated Scene Gizmos

Scene viewport gizmo extraction now reserves from the exact node-kind admission bound: camera and
directional-light nodes can each contribute at most one scene gizmo. Inactive nodes and construction
failures remain skipped, and additional gizmos are still appended by the interaction extractor.

The regression checks the source contract. The ignored Windows Release benchmark emits
`EDITOR641_PREALLOCATED_SCENE_GIZMOS_BENCH_V1` over 101 alternating sample pairs and 65,536 gizmo
projections. The marker includes both raw sorted series and nearest-rank P50/P95 values. The gate
requires preallocated P95 to be at most 85% of unreserved P95.

### 2026-09-18 evidence refresh

The Release probe was strengthened from 17 to 101 alternating pairs and now reports the complete
sample series plus P50/P95. The source allocation bound and scene-gizmo semantics are unchanged;
the source contract is GREEN at `2/2`, and managed Cargo/Release measurements remain asynchronous
acceptance gates.

The current batched local Runtime/Editor source-contract lane includes Editor641 and Runtime170
plus four adjacent suites: `45/45` tests pass in `94.665s`; exact-file Rustfmt, Python compilation,
scoped diff checks, and wiki validation are green. These receipts are local source/model evidence,
not managed Cargo/Release or product percentile acceptance.

No direct Cargo validation was run. The coordinator owns aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
