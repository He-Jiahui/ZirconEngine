---
title: Editor659 Streamed Source Outline Boundaries
category: zircon_editor
report_id: Editor659-streamed-source-outline-boundaries-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_release_pending
---

# Editor659 Streamed Source Outline Boundaries

Source-outline line segment construction now consumes its ordered event map through a peekable
iterator. The sweep reads the next boundary directly from the iterator, eliminating the copied
boundary vector and the `BTreeMap` lookup previously performed at every boundary. Active-range
priority, same-start specificity, invalid ranges, and final open-range handling remain unchanged.

The deterministic regression compares the complete optimized and legacy segment sequences for
nested and same-start ranges. A source contract rejects reintroduction of the boundary vector or
tree lookup. The ignored Windows Release benchmark emits
`EDITOR659_STREAMED_SOURCE_OUTLINE_BOUNDARIES_BENCH_V1` for 8,192 entries over 17 alternating
samples. Its gate requires streamed-boundary P95 to be at most 80% of legacy P95 and reports one
removed vector allocation plus all eliminated boundary lookups.

This candidate is grouped with Runtime659 for one coordinator validation request. Measured P95,
commit SHA, push state, and WeCom publication must be written only from an accepted coordinator
result.
