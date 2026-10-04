---
title: Editor655 Preallocated Renderer Diagnostic Indexes
category: zircon_editor
report_id: Editor655-preallocated-renderer-diagnostic-indexes-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: retracted_after_review
validation_status: not_submitted
---

# Editor655 Preallocated Renderer Diagnostic Indexes

The proposed RendererData projection reservation was withdrawn. Diagnostic rows are not a strict
bound on the number of useful buckets: the feature domain is intentionally low-cardinality, and
material or shader references may repeat or be absent. Reserving from row/reference counts could
therefore exchange a small rehash cost for persistent over-allocation on common editor views.

Runtime655 and Editor655 are both retracted, and their candidate source/test changes were removed.
The earlier aggregate request was rejected before Cargo admission. No test, performance, commit,
push, or WeCom success is claimed.
