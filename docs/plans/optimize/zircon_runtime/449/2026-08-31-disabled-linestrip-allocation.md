---
title: Runtime449 Disabled Line Strip Allocation
category: zircon_runtime
report_id: Runtime449-disabled-linestrip-allocation-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime449 Disabled Line Strip Allocation

Disabled gizmo line strips now consume their input iterator without collecting its points into a
temporary vector. The former path always materialized the complete line strip and then passed the
command to the common disabled-command guard, which immediately discarded it.

Iterator consumption and its observable side effects remain unchanged, while disabled buffers
still retain no command. Enabled buffers continue to collect and store the exact point sequence.
Regression coverage counts visits through a disabled iterator and constrains the allocation to the
enabled branch.

The ignored Windows Release benchmark emits `RUNTIME449_DISABLED_LINESTRIP_ALLOCATION_BENCH_V1`
over 17 alternating paired samples. Each sample submits 256 disabled line strips containing 1,024
points, with identical per-point observation in both paths. The legacy path allocates one point
vector per strip and the optimized path allocates none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime449 is prepared with Editor379 under request
`runtime449-editor379-performance-batch-20260831eq-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
