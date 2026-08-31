---
title: Editor397 Direct Command Result Strings
category: zircon_editor
report_id: Editor397-direct-command-result-strings-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor397 Direct Command Result Strings

Runtime UI command dispatch now constructs the `command:<id>` transaction identity and its executed
command status with two exact-capacity direct outputs. Adapter dirty state, mutation source,
validation, and exact user-visible bytes remain unchanged.

Regression coverage compares ordinary, dotted, and slash-bearing command IDs with the former
formatter. The ignored Windows Release benchmark emits
`EDITOR397_DIRECT_COMMAND_RESULT_STRINGS_BENCH_V1` over 17 alternating paired samples, each building
262,144 transaction/status pairs. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at
least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor397 is prepared with Runtime467 under request
`runtime467-editor397-performance-batch-20260831fk-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
