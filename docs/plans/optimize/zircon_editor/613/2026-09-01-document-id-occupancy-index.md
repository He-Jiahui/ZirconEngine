---
title: Editor613 Document ID Occupancy Index
category: zircon_editor
report_id: Editor613-document-id-occupancy-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor613 Document ID Occupancy Index

Document identity allocation now maintains one `HashSet<DocumentId>` alongside the root and scene
identity maps. The previous collision probe scanned both ordered maps' values for every candidate,
so a collision-heavy project paid two linear passes per probe despite only needing membership.

Both identity constructors insert into the shared index, and both retention paths remove the exact
evicted ID. Collision probing is now a single constant-time membership check while the ordered
maps remain unchanged for deterministic retention snapshots and path lookup. Existing collision,
reopen, scene-session, close, and retention semantics are preserved.

The ignored Windows Release benchmark emits `EDITOR613_DOCUMENT_ID_OCCUPANCY_INDEX_BENCH_V1` over
17 alternating sample pairs, 2,048 project roots, 2,048 scene identities, and 1,024 occupancy
queries. The gate requires indexed membership P95 to be at most 10% of the two-map value scans.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regressions, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor613 is prepared with Runtime613 under request
`runtime613-editor613-counted-removal-occupancy-performance-20260901id-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
