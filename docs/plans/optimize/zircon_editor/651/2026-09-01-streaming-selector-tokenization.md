---
title: Editor651 Streaming Selector Tokenization
category: zircon_editor
report_id: Editor651-streaming-selector-tokenization-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor651 Streaming Selector Tokenization

Inspector selector tokenization now locates token boundaries directly through UTF-8 byte offsets
instead of materializing an intermediate `Vec<char>`. Token kind classification, `:host` handling,
empty-token rejection, Unicode boundaries, and downstream owned token values remain unchanged.

The ignored Windows Release benchmark emits `EDITOR651_STREAMING_SELECTOR_TOKENIZATION_BENCH_V1`
over 17 alternating sample pairs, 2,048 parses per sample, and a long mixed selector. The gate
requires streaming P95 to be at most 70% of the buffered parser P95.

No direct Cargo validation was run. The coordinator owns combined Runtime651/Editor651 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor651 is prepared with Runtime651 under the shared `optimization_batch_jl_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
