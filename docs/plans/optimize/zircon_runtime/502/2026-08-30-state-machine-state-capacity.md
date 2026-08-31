---
title: Runtime502 State Machine State Capacity
category: zircon_runtime
report_id: Runtime502-state-machine-state-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime502 State Machine State Capacity

Animation state-machine compilation previously grew its validated source-state vector from zero.
The collector now reserves the authored state count as a safe upper bound before retaining valid,
unique states. Authored order, duplicate handling, diagnostic order, and dense slot assignment are
unchanged.

The source regression requires the authored-count reservation. The ignored Windows Release
benchmark emits `RUNTIME502_STATE_MACHINE_STATE_CAPACITY_BENCH_V1` for 32,768 synthetic states and
requires zero optimized vector-growth events versus a positive legacy count, a 100% growth-event
reduction.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, focused tests, ignored performance evidence, manifest-only commit/push, and one-shot
WeCom publication after the declared gates pass.

## Current batched validation handoff (2026-08-30)

Runtime502 is batched with Editor502 under request
`runtime502-state-machine-editor502-journal-entry-capacity-20260830co-v1`. Receipt, ticket, source
manifest, and terminal evidence are recorded after coordinator acceptance.
