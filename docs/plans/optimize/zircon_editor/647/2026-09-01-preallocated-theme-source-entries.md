---
title: Editor647 Preallocated Theme Source Entries
category: zircon_editor
report_id: Editor647-preallocated-theme-source-entries-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor647 Preallocated Theme Source Entries

Theme source projection now reserves one possible local entry plus every imported style reference.
Every import yields exactly one output entry and the local source yields at most one, so the upper
bound removes vector growth while preserving local-first and import-order presentation.

The ignored Windows Release benchmark emits `EDITOR647_THEME_SOURCE_ENTRY_CAPACITY_BENCH_V1` over
17 alternating sample pairs with 65,536 imported sources. The gate requires reserved P95 to be at
most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime647/Editor647 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor647 is prepared with Runtime647 under the shared `optimization_batch_jh_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
