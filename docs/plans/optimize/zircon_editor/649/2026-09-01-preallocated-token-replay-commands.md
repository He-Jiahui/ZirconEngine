---
title: Editor649 Preallocated Token Replay Commands
category: zircon_editor
report_id: Editor649-preallocated-token-replay-commands-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor649 Preallocated Token Replay Commands

Theme token replay now reserves its command vector from the sum of current and target token counts.
Each current token can produce at most one removal and each target token at most one upsert, making
the sum a strict upper bound while preserving reverse-removal order, target insertion order, value
cloning, and no-op filtering.

The ignored Windows Release benchmark emits `EDITOR649_TOKEN_REPLAY_CAPACITY_BENCH_V1` over 21
alternating sample pairs, 64 batches per sample, and 4,096 replay commands per batch. The gate
requires reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime649/Editor649 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor649 is prepared with Runtime649 under the shared `optimization_batch_jj_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
