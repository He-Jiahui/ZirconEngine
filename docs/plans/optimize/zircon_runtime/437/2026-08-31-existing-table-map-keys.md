---
title: Runtime437 Existing Table Map Key Updates
category: zircon_runtime
report_id: Runtime437-existing-table-map-keys-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime437 Existing Table Map Key Updates

Table column-width and sort-direction mutation now probes owned metadata maps with borrowed keys and
overwrites existing values in place. Repeated resize and sort interactions therefore avoid
allocating replacement key strings for the dynamic field key and the fixed `width` and
`sortDirection` keys.

A shared leaf helper preserves the prior fallback for missing keys by allocating and inserting an
owned key only on first use. The surrounding metadata conversion, matching, value construction,
mutation request, binding report, and accepted/rejected result remain unchanged. Regression tests
cover both existing-key replacement and absent-key insertion.

The ignored Windows Release benchmark emits `RUNTIME437_EXISTING_TABLE_MAP_KEY_BENCH_V1` over 17
alternating paired samples. Each sample performs 128 update passes over 256 existing column keys,
for 32,768 replacements: the legacy path allocates 32,768 key strings and the optimized path
allocates none. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime437 is prepared with Editor365 under request
`runtime437-editor365-performance-batch-20260831ec-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
