---
title: Editor06 Unstable Contribution Capability Sort
category: zircon_editor
report_id: Editor06-unstable-contribution-capability-sort-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Unstable Contribution Capability Sort

## Scope

Menu item and asset importer descriptors canonicalize required capabilities at extension
admission. Capability order is sorted and duplicates are removed, so stable sorting is not
observable.

## Implementation

Both builders now reserve the iterator lower bound, extend into the existing vector, and use
`sort_unstable` before deduplication. Existing repeated builder calls retain their contents and
canonical order.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable sort calls per menu/importer pair | 2 | 0 |
| Initial capacity growth | geometric | iterator lower bound reserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR06_UNSTABLE_CONTRIBUTION_CAPABILITY_SORT_BENCH_V1` with both
p95 durations, sample/iteration/capability counts, and capacity/sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and descriptor equivalence tests are prepared.
The release benchmark is submitted with extension normalization in one Editor crate command;
commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
