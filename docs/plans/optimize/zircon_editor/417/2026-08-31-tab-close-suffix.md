---
title: Editor417 Tab Close Suffix
category: zircon_editor
report_id: Editor417-tab-close-suffix-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor417 Tab Close Suffix

Editor icon-button context now checks the common `TabClose` suffix first, making the normal custom
tab-close path a single string comparison. Legacy Dock/Page/Document prefix forms with trailing
extensions remain accepted through a first-byte-gated compatibility fallback; button context and
style selection are unchanged.

Regression coverage verifies the standard suffix, all legacy prefix forms, extended legacy IDs,
and non-matches. The ignored Windows Release benchmark emits `EDITOR417_TAB_CLOSE_SUFFIX_BENCH_V1`
over 17 alternating paired samples and 1,048,576 custom-tab-close lookups per sample. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.50` (at least 50% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor417 is prepared with Runtime487 under request
`runtime487-editor417-performance-batch-20260831ge-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
