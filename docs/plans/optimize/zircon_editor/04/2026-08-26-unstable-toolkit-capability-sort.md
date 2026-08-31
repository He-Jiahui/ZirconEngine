---
title: Editor04 Unstable Toolkit Capability Sort
category: zircon_editor
report_id: Editor04-unstable-toolkit-capability-sort-2026-08-26
date: 2026-08-26
session_id: root-editor04-folder-child-hash-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Unstable Toolkit Capability Sort

## Scope

Asset toolkit descriptors canonicalize required capabilities for capability-aware asset opening.
The final list is sorted and deduplicated, so equal-key insertion order has no meaning.

## Implementation

The builder now reserves the iterator lower bound, extends directly into the reserved vector,
and uses `sort_unstable` before deduplication. The resulting capability list is unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable sort calls | 1 | 0 |
| Initial capacity growth | geometric | lower bound reserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR04_TYPE_REGISTRY_TOOLKIT_CAPABILITY_BENCH_V1` with both p95
durations, sample/iteration/capability counts, and allocation/sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and descriptor equivalence tests are prepared.
The release benchmark is batched with context command and creation template in one Editor crate
command; commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
