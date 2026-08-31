---
title: Editor378 Allocation-Free Output Finish
category: zircon_editor
report_id: Editor378-allocation-free-output-finish-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor378 Allocation-Free Output Finish

Cargo output tail finalization now formats the truncation diagnostic directly into its destination
`String` and exposes the retained `VecDeque<u8>` as a contiguous slice for UTF-8 decoding. The
former path allocated a temporary formatted string and collected the byte deque into a temporary
`Vec<u8>` before copying both into the final output.

The truncation diagnostic, byte accounting, exact retained tail, valid UTF-8 behavior, and lossy
replacement for invalid UTF-8 remain unchanged. Regression coverage compares both implementations
for valid and invalid byte sequences and forbids the two former temporary buffers.

The ignored Windows Release benchmark emits `EDITOR378_ALLOCATION_FREE_OUTPUT_FINISH_BENCH_V1`
over 17 alternating paired samples. Each sample finishes 32 prepared 64 KiB output tails; tail
construction is outside the timed section. The legacy path performs two temporary allocations per
finish and the optimized valid-UTF-8 path performs none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor378 is prepared with Editor377 under request
`editor377-editor378-performance-batch-20260831ep-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
