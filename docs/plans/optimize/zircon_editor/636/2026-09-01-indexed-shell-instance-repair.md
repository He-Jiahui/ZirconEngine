---
title: Editor636 Indexed Shell Instance Repair
category: zircon_editor
report_id: Editor636-indexed-shell-instance-repair-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor636 Indexed Shell Instance Repair

Built-in shell layout repair now builds borrowed indexes for open instance and descriptor IDs once
per repair. Baseline main-page and drawer restoration previously repeated linear scans of every open
instance for each exact lookup and each descriptor fallback.

Both indexes reserve from the open-instance count. First insertion wins, preserving the previous
linear scan's first-descriptor-match behavior, while exact instance IDs still take priority over
descriptor fallback. Layout ordering, duplicate admission, active-tab repair, and drawer visibility
semantics remain unchanged.

The ignored Windows Release benchmark emits
`EDITOR636_INDEXED_SHELL_INSTANCE_LOOKUP_BENCH_V1` over 17 alternating sample pairs with 2,048 open
instances and 4,096 mixed exact/fallback queries. The gate requires indexed P95 to be at most 25%
of repeated-linear-scan P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime636/Editor636 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor636 is prepared for a combined Runtime636/Editor636 request. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
