---
title: Editor385 Direct Module Plugin Action ID
category: zircon_editor
report_id: Editor385-direct-module-plugin-action-id-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor385 Direct Module Plugin Action ID

Module/plugin pane projection now joins each action prefix and plugin ID in one exact-capacity
string. A single plugin row requests enable/disable plus packaging, target-mode, unload, and hot
reload action IDs; the former helper invoked generic formatting for every one.

The prefix, dot separator, plugin identity, and empty-input behavior remain byte-for-byte
unchanged. Regression coverage compares all representative forms with the former format
expression.

The ignored Windows Release benchmark emits `EDITOR385_DIRECT_MODULE_PLUGIN_ACTION_ID_BENCH_V1`
over 17 alternating paired samples, each constructing 262,144 representative action IDs. The
optimized path performs three direct writes into one exact-capacity allocation. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.65` (at least 35% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor385 is prepared with Runtime455 under request
`runtime455-editor385-performance-batch-20260831ew-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
