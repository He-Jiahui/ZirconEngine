---
title: Editor359 Borrowed Asset Change Dedup
category: zircon_editor
report_id: Editor359-borrowed-asset-change-dedup-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor359 Borrowed Asset Change Dedup

UI asset refresh normalization now probes the ordered unique set with the borrowed normalized
asset ID before allocating its owned key. Repeated watcher events and multiple fragment-qualified
references for the same asset therefore allocate once per unique asset instead of once per event.

The function retains its generic input contract and still returns the same sorted
`BTreeSet<String>`. Regression tests cover fragment removal, stable uniqueness, and the
allocate-after-probe source contract.

The ignored Windows Release benchmark emits `EDITOR359_BORROWED_UI_ASSET_CHANGE_DEDUP_BENCH_V1`
over 17 alternating paired samples, each normalizing 2,048 events with 32 unique long asset IDs 256
times, requiring `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor359 is prepared with Runtime431 under request
`runtime431-editor359-performance-batch-20260831dw-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
