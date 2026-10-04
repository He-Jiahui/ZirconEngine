---
title: Editor640 Preallocated Viewport Screen Lines
category: zircon_editor
report_id: Editor640-preallocated-viewport-screen-lines-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor640 Preallocated Viewport Screen Lines

Viewport transform-handle projection now computes a conservative line upper bound from each overlay
element before appending projected lines. Axis rings reserve 48 segment slots, axis lines and scales
reserve their three-line maximum, and center anchors reserve two; projection failures are still
filtered exactly as before.

The regression checks the source-level capacity contract. The ignored Windows Release benchmark emits
`EDITOR640_PREALLOCATED_VIEWPORT_SCREEN_LINES_BENCH_V1` over 17 alternating sample pairs and 65,536
line projections. The gate requires preallocated P95 to be at most 85% of unreserved P95.

No direct Cargo validation was run. The coordinator owns aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
