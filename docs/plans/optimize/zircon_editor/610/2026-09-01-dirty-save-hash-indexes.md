---
title: Editor610 Dirty Save Hash Indexes
category: zircon_editor
report_id: Editor610-dirty-save-hash-indexes-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor610 Dirty Save Hash Indexes

Dirty-view save preflight and completion application now use capacity-sized hash indexes for their
temporary toolkit, expected-document, and completion lookups. The previous implementation built a
`BTreeMap`, a `BTreeSet`, and another `BTreeMap` even though none of those indexes supplied output
order. Candidates remain sorted before preflight, failure discovery still follows candidate or
completion input order, and outcomes still follow canonical intent order.

The source regression requires each hash index to reserve from the exact descriptor or intent count
and rejects the former ordered-tree collections. Existing tests continue to cover accumulated
preflight failures, stable partial outcomes, retry order, unknown completions, duplicates, and dirty
state preservation.

The ignored Windows Release benchmark emits `EDITOR610_DIRTY_SAVE_HASH_INDEX_BENCH_V1` over 17
alternating sample pairs and 32,768 documents. It models expected membership, reverse-order
completion insertion, and canonical intent-order removal. The gate requires hash-index P95 to be at
most 60% of the legacy tree-index P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor610 is prepared with Runtime610 under request
`runtime610-editor610-borrowed-scenario-hash-index-performance-20260901ia-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
