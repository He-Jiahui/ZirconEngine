---
title: Editor634 Preallocated Default View Instances
category: zircon_editor
report_id: Editor634-preallocated-default-view-instances-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor634 Preallocated Default View Instances

Default workbench view projection now computes the total primary and drawer view count once and
uses it to reserve both unique-ID membership and the resulting instance vector. The previous
collections both grew from zero.

The capacity fold uses saturated addition. Window traversal, primary-before-drawer ordering,
first-instance retention, generated titles, and host assignment remain unchanged.

The ignored Windows Release benchmark emits `EDITOR634_PREALLOCATED_DEFAULT_VIEW_INSTANCE_BENCH_V1`
over 17 alternating sample pairs with 65,536 view identities. The gate requires preallocated P95
to be at most 80% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime634/Editor634 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor634 is prepared with Runtime634 under request
`runtime634-editor634-shader-view-capacity-performance-20260901ix-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
