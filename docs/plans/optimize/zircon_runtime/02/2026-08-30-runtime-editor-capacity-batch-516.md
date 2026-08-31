---
title: Runtime Editor Capacity Batch 516
category: zircon_runtime
report_id: RuntimeEditor516-capacity-batch-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Editor Capacity Batch 516

Runtime GPU-instancing candidate projection now reserves the visible-batch count as a safe upper
bound before retaining only dynamic batches with multiple stable instance keys. Candidate order and
the existing filter remain unchanged. Editor retained text binding now computes the exact one-main
plus optional-numeric mutation count before allocation while preserving numeric parsing and the
main-text-first mutation order.

The ignored Windows Release evidence models 32,768 Runtime batches with 64 candidates and 32,768
Editor batches with two mutations. `RUNTIME516_GPU_INSTANCING_CANDIDATE_CAPACITY_BENCH_V1` and
`EDITOR516_TEXT_MUTATION_CAPACITY_BENCH_V1` both require zero optimized vector growth versus a
positive legacy count.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, focused tests, ignored performance evidence, manifest-only commit/push, and one-shot
WeCom publication after the declared gates pass.

## Current batched validation handoff (2026-08-30)

The combined request is `runtime516-gpu-candidate-editor516-text-mutation-20260830dd-v1`. Receipt,
ticket, source manifest, and terminal evidence are recorded after coordinator acceptance.
