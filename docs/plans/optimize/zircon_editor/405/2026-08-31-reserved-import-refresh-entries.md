---
title: Editor405 Reserved Import Refresh Entries
category: zircon_editor
report_id: Editor405-reserved-import-refresh-entries-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor405 Reserved Import Refresh Entries

Editor UI asset import refresh now reserves its temporary entry vector from the requested instance
count before reading session references. Missing sessions are still skipped, iteration remains in
the `BTreeSet` order, cloned instance and import-reference ownership is unchanged, and the session
read lock covers the same collection phase.

This removes repeated vector growth while preparing a large refresh batch. Structural regression
coverage requires the capacity reservation and retirement of the `filter_map().collect()` path.
The ignored Windows Release benchmark emits
`EDITOR405_RESERVED_IMPORT_REFRESH_ENTRIES_BENCH_V1` over 17 alternating paired samples, 256
builds per sample, and 1,024 72-byte entries per build. The pressure case reduces vector growth
allocations per build from nine to zero. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor405 is prepared with Runtime475 under request
`runtime475-editor405-performance-batch-20260831fs-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
