---
title: Runtime638 Preallocated Hovered Hit Output
category: zircon_runtime
report_id: Runtime638-preallocated-hovered-hit-output-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime638 Preallocated Hovered Hit Output

The hovered-hit projection now reserves its output vector from the already sorted hit count. Every
hovered result is a subset of that input, so the bound is exact without changing the blocking rule,
input order, or returned hit ownership.

The ignored Windows Release benchmark emits `RUNTIME638_PREALLOCATED_HOVERED_HIT_OUTPUT_BENCH_V1`
over 17 alternating sample pairs with 65,536 hits. The gate requires preallocated P95 to be at most
80% of the unreserved projection P95.

No direct Cargo validation was run. The coordinator owns combined Runtime638/Editor638 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime638 is prepared with Editor638 under the shared `optimization_batch_iz_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
