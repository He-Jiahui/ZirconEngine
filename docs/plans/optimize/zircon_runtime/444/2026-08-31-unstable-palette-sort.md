---
title: Runtime444 Unstable Palette Sort
category: zircon_runtime
report_id: Runtime444-unstable-palette-sort-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime444 Unstable Palette Sort

Runtime UI component palette projection now uses the allocation-free unstable slice sorter. The
ordering comparator already ends with the registry-unique component id after category, authored sort
key, and display name, so stability cannot affect the projected order or visible values.

Host capability filtering, palette metadata projection, category ordering, authored ordering,
display-name ordering, and component-id tie breaking remain unchanged. Existing projection tests
verify the sorted result; new regression coverage requires the total-key unstable sorter and rejects
restoration of the stable sorter.

The ignored Windows Release benchmark emits `RUNTIME444_UNSTABLE_PALETTE_SORT_BENCH_V1` over 17
alternating paired samples. Each sample sorts eight independently cloned 4,096-row palettes using
the production four-key comparator and unique component ids. Input cloning is excluded from timing.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.90`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime444 is prepared with Editor372 under request
`runtime444-editor372-performance-batch-20260831ej-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
