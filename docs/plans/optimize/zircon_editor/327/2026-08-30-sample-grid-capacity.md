---
title: Editor327 Sample Grid Capacity
category: zircon_editor
report_id: Editor327-sample-grid-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor327 Sample Grid Capacity

Sample-grid point and numeric-array projection now reserve exact TOML array capacity before
filtering and populating output vectors. Invalid values remain filtered, point order and defaults
remain unchanged, and absent arrays still produce empty outputs.

The ignored Windows Release benchmark emits `EDITOR327_SAMPLE_GRID_CAPACITY_BENCH_V1` over 17
paired samples with 256 values per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor327 is submitted with Runtime381 under request
`runtime381-editor327-performance-batch-20260830cc-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
