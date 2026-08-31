---
title: Editor352 Dialog Attribute Borrow
category: zircon_editor
report_id: Editor352-dialog-attribute-borrow-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor352 Dialog Attribute Borrow

Dialog and confirm-dialog action projection now reads TOML string attributes by reference and clones
only when converting the selected label or action id into the owned host-contract model. Default
labels and action ids, precedence order, and unsupported-role behavior remain unchanged.

The previous path cloned every candidate string while searching attribute names. The borrowed
lookup avoids those transient string allocations, including the multiple lookups performed for
confirm-dialog actions.

The ignored Windows Release benchmark emits
`EDITOR352_DIALOG_ATTRIBUTE_BORROW_BENCH_V1` over 17 alternating paired samples and 8,192 paired
attribute lookups per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor352 is prepared with Runtime424 under request
`runtime424-editor352-performance-batch-20260830dp-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
