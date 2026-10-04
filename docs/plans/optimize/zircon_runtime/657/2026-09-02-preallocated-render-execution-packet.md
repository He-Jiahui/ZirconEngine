---
title: Runtime657 Preallocated Render Execution Packet
category: zircon_runtime
report_id: Runtime657-preallocated-render-execution-packet-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: retracted_after_review
validation_status: not_submitted
---

# Runtime657 Preallocated Render Execution Packet

This proposal was retracted during source review: reserving every graph pass for execution batches
over-allocates graphs whose live passes share one queue segment, and the stage-order buffer is too
small to justify a separate benchmark. No source or test remains attached to this plan.

Compiled render packet assembly now reserves execution batches from graph pass count and stage
order from `RenderPassStage::COUNT`. Both are strict upper bounds: every batch owns at least one
live pass and every stage can appear only once. Queue transition handling, culling gaps, batch
validation, and stage ordering remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME657_RENDER_EXECUTION_BATCH_CAPACITY_BENCH_V1`
over 17 alternating sample pairs and 16,384 pass batches. The gate requires reserved P95 to be at
most 80% of unreserved P95 and requires zero reserved capacity growths.

No direct Cargo validation was run. The coordinator owns combined Runtime657/Editor657 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.
