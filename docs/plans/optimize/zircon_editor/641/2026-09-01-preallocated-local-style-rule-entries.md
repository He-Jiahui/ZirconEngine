---
title: Editor641 Preallocated Local Style Rule Entries
category: zircon_editor
report_id: Editor641-preallocated-local-style-rule-entries-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor641 Preallocated Local Style Rule Entries

Local style-rule projection now sums the rule counts already stored on each stylesheet and reserves
the exact entry-vector capacity before cloning rule metadata. The capacity pass visits only the
stylesheet headers rather than the rules themselves. Stylesheet order, rule order, indices, IDs,
and selectors remain unchanged.

The ignored Windows Release benchmark emits
`EDITOR641_PREALLOCATED_LOCAL_STYLE_RULE_ENTRY_BENCH_V1` over 17 alternating sample pairs with 16
stylesheets and 65,536 rules per stylesheet. The gate requires preallocated P95 to be at most 80%
of the unreserved entry-build P95.

No direct Cargo validation was run. The coordinator owns combined Runtime641/Editor641 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor641 is prepared with Runtime641 under the shared `optimization_batch_jb_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
