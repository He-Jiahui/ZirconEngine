---
title: Editor08 Keyword Normalization Hotpath
category: zircon_editor
report_id: Editor08-keyword-normalization-hotpath-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor08 Keyword Normalization Hotpath

## Scope

`EditorCommandDescriptor::with_keywords` canonicalizes command palette keywords during command
admission. The previous path collected into a growing vector and used the stable sorter even
though keyword order is canonicalized and equal entries are removed.

## Implementation

The builder now reserves the iterator lower bound, extends directly into the reserved vector,
and uses `sort_unstable` before deduplication. The resulting sorted unique keyword list is
unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Initial keyword vector capacity | geometric growth | iterator lower bound |
| Stable sort calls | 1 | 0 |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR08_COMMAND_DESCRIPTOR_KEYWORD_NORMALIZATION_BENCH_V1` with
both p95 durations, sample/iteration/keyword counts, and the capacity/sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and legacy-equivalence tests are prepared. The
release benchmark is submitted with missing-capability diagnostics in one Editor crate command;
commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
