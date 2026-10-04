---
title: Runtime650 Preallocated Catalog Input Index
category: zircon_runtime
report_id: Runtime650-preallocated-catalog-input-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime650 Preallocated Catalog Input Index

Full project generation now reserves the catalog-input hash map from the collected source count.
Each source inserts at most one root catalog input on either the restored or imported path, so the
source count is a strict upper bound. Asset import order, replacement behavior, dependency
projection, and failure handling remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME650_CATALOG_INPUT_INDEX_CAPACITY_BENCH_V1` over
21 alternating sample pairs, 48 batches per sample, and 4,096 source entries per batch. The gate
requires reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime650/Editor650 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime650 is prepared with Editor650 under the shared `optimization_batch_jk_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
