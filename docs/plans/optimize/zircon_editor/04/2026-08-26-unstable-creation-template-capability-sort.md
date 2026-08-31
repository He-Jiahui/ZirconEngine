---
title: Editor04 Unstable Creation Template Capability Sort
category: zircon_editor
report_id: Editor04-unstable-creation-template-capability-sort-2026-08-26
date: 2026-08-26
session_id: root-editor04-folder-child-hash-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Unstable Creation Template Capability Sort

## Scope

Asset creation template descriptors canonicalize required capabilities at registration. Equal
capabilities are removed, leaving no stable-order contract.

## Implementation

The builder now reserves the iterator lower bound, extends into the existing vector, and uses
`sort_unstable` before deduplication without changing the public descriptor projection.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable sort calls | 1 | 0 |
| Initial capacity growth | geometric | lower bound reserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR04_TYPE_REGISTRY_CREATION_TEMPLATE_CAPABILITY_BENCH_V1`
with both p95 durations, sample/iteration/capability counts, and allocation/sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and descriptor equivalence tests are prepared.
The release benchmark is submitted in the same managed Editor command as the other type registry
descriptor builders; commit integration, terminal p95 values, and WeCom delivery remain
coordinator-owned.
