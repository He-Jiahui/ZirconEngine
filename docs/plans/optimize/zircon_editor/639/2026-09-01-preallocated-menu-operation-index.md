---
title: Editor639 Preallocated Menu Operation Index
category: zircon_editor
report_id: Editor639-preallocated-menu-operation-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor639 Preallocated Menu Operation Index

The contributed-menu operation index now counts the recursive menu tree and reserves the HashSet
from that item bound before collecting operation paths. Duplicate operations still retain set
semantics, and recursive menu traversal order is unchanged.

The ignored Windows Release benchmark emits `EDITOR639_PREALLOCATED_MENU_OPERATION_INDEX_BENCH_V1`
over 17 alternating sample pairs with 65,536 menu items. The gate requires preallocated P95 to be
at most 80% of the unreserved index-build P95.

No direct Cargo validation was run. The coordinator owns combined Runtime639/Editor639 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor639 is prepared with Runtime639 under the shared `optimization_batch_iz_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
