---
title: Editor06 Unstable Extension Normalization Sort
category: zircon_editor
report_id: Editor06-unstable-extension-normalization-sort-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Unstable Extension Normalization Sort

## Scope

Asset importer extension normalization repairs an unsorted legacy vector before binary insertion.
The repair path deduplicates the vector, so equal extension strings do not carry ordering
semantics and stable sorting is unnecessary.

## Implementation

The repair path now uses `sort_unstable` with the same lexicographic comparator and deduplication.
Normalized extension contents and binary-search insertion behavior are unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable repair sorts | 1 | 0 |
| Extension order/uniqueness | preserved | preserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR06_UNSTABLE_EXTENSION_NORMALIZATION_SORT_BENCH_V1` with both
p95 durations, sample/iteration/extension counts, and stable-sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and extension repair equivalence tests are
prepared. The release benchmark is batched with capability normalization in one Editor crate
command; commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
