---
title: Editor656 Preallocated Catalog Update Index
category: zircon_editor
report_id: Editor656-preallocated-catalog-update-index-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: admission_rejected_external_worktree_dirty
---

# Editor656 Preallocated Catalog Update Index

Batch catalog publication now materializes the update iterator once and reserves its UUID-indexed
update map from the iterator lower bound. The lower bound is exact for the production
`BTreeMap::values().cloned()` watch batch and remains conservative for other iterators. Unknown UUID
filtering, last-write behavior, sorted index application, and one-publication COW semantics remain
unchanged.

The ignored Windows Release benchmark emits `EDITOR656_CATALOG_UPDATE_INDEX_CAPACITY_BENCH_V1`
over 17 alternating sample pairs and 16,384 updates. The gate requires reserved P95 to be at most
80% of unreserved P95 and requires zero reserved capacity growths.

Runtime656 was withdrawn after source review because its proposed descriptor root-count bound was
not strict for dependency closures. Editor656 is retained as an Editor-only candidate. No direct
Cargo validation, performance measurement, commit, push, or WeCom publication has succeeded; the
coordinator must validate this Editor slice after the current external-worktree admission blocker
is resolved.
