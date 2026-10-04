---
title: Runtime609 Counter Hotspot Borrowed Group Key
category: zircon_runtime
report_id: Runtime609-counter-hotspot-borrowed-group-key-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime609 Counter Hotspot Borrowed Group Key

Counter-hotspot aggregation now groups samples by borrowed `stream` and `name` slices from the
immutable profiling snapshot. The previous key cloned both strings and formatted the final report
path for every accepted sample before the hash lookup. The optimized path owns those three strings
only once per completed group, while filtering, totals, percentile selection, latest-sample rules,
sort order, and report schema remain unchanged.

A focused source regression requires a lifetime-bound group key and rejects cloning or path
formatting in key conversion. The existing behavior test continues to cover grouping, ordering,
frame counts, latest values, and hints.

The ignored Windows Release benchmark emits
`RUNTIME609_COUNTER_HOTSPOT_BORROWED_KEY_BENCH_V1` over 17 alternating sample pairs, 16,384
counters, and 256 groups. Owned group-key text allocations fall from three per accepted sample to
three per completed group. The gate requires borrowed-key grouping P95 to be at most 45% of legacy
owned-key P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime609 is prepared with Editor609 under request
`runtime609-editor609-borrowed-group-prefix-performance-20260901hz-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
