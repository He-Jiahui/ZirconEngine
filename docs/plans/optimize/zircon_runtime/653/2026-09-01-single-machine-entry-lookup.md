---
title: Runtime653 Single Machine Entry Lookup
category: zircon_runtime
report_id: Runtime653-single-machine-entry-lookup-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime653 Single Machine Entry Lookup

State registry insertion now downcasts the mutable `Entry` value returned by `or_insert_with`
directly, eliminating a second HashMap lookup on every state-machine creation or registration.
TypeId ownership, wrong-type protection, and all state-machine behavior remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME653_SINGLE_MACHINE_LOOKUP_BENCH_V1` over 17
alternating sample pairs and 4,194,304 lookups per sample across 4,096 machine keys. The gate
requires the single-entry path P95 to be at most 75% of the old double-lookup path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime653/Editor653 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime653 is prepared with Editor653 under the shared `optimization_batch_jn_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
