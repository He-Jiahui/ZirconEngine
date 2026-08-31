---
title: Editor402 Borrowed Export Report Key Update
category: zircon_editor
report_id: Editor402-borrowed-export-report-key-update-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor402 Borrowed Export Report Key Update

Desktop export report publication now checks the report cache with the summary's borrowed profile
name and replaces an existing value in place. Only a profile that is not yet present clones its
name for the map key. Completed-job polling and pending-job cancellation share the same insertion
policy; status text, invalidation, and dirty-marking order are unchanged.

This removes a temporary profile-name allocation whenever a later run or cancellation replaces the
cached report for an existing export profile. Regression coverage checks first insertion, same-key
replacement, and the borrowed production lookup. The ignored Windows Release benchmark emits
`EDITOR402_BORROWED_EXPORT_REPORT_KEY_BENCH_V1` over 17 alternating paired samples and 8,192
same-profile updates per sample. The pressure case reduces key copies per sample from 8,192 to
zero. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70` (at least 30% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor402 is prepared with Runtime472 under request
`runtime472-editor402-performance-batch-20260831fp-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
