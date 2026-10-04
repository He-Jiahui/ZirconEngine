---
title: Editor640 Preallocated Build Export Target Indexes
category: zircon_editor
report_id: Editor640-preallocated-build-export-target-indexes-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor640 Preallocated Build Export Target Indexes

Build-export target projection now reserves platform and target-ID count maps from the materialized
target count. The duplicate occurrence map remains unreserved because it is normally sparse and only
grows for repeated target identities. Row order, stable ID generation, duplicate suffixing, and node
layout remain unchanged.

The ignored Windows Release benchmark emits
`EDITOR640_PREALLOCATED_BUILD_EXPORT_TARGET_INDEX_BENCH_V1` over 17 alternating sample pairs with
65,536 targets. The gate requires preallocated P95 to be at most 80% of the unreserved two-index
build P95.

No direct Cargo validation was run. The coordinator owns combined Runtime640/Editor640 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor640 is prepared with Runtime640 under the shared `optimization_batch_ja_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
