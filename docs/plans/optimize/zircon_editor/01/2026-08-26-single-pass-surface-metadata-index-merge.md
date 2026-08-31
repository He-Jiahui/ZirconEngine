---
title: Editor01 Single-pass Surface Metadata Index Merge
category: zircon_editor
report_id: Editor01-single-pass-surface-metadata-index-merge-2026-08-26
date: 2026-08-26
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Single-pass Surface Metadata Index Merge

## Scope

This slice removes temporary attribute/style `BTreeMap` clones while building the retained surface
metadata index. It preserves traversal order, duplicate-control precedence, owned index lifetime,
and the existing `last write wins` behavior.

## Implementation

Index construction now extends the destination metadata maps from cloned key/value pairs in the
source node maps. The old `node.attributes.clone()` and `node.style_tokens.clone()` temporary maps
are gone; each source entry is copied directly into the index-owned map before the traversal moves
to the next node.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Temporary attribute maps for 1,024 nodes | 2,048 | 0 | direct entry merge |
| Temporary style maps for 1,024 nodes | 2,048 | 0 | direct entry merge |
| Windows-native release p95 | dynamic evidence pending | <= 85% of legacy p95 | coordinator gate |

The ignored release benchmark alternates 17 samples over 256 iterations and prints
`EDITOR01_SINGLE_PASS_SURFACE_METADATA_INDEX_MERGE_BENCH_V1` with both p95 timings, node and
attribute counts, and temporary-map counts. Exact elapsed-time evidence is accepted only from the
coordinator terminal receipt.

## Validation

- Existing preorder/last-write-wins coverage remains in the production module tests.
- The optimization module adds a populated 1,024-node merge regression and source contract.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the apply-merge task; no
  per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.

## Remaining Parent-plan Work

Editor01 still owns retained UI generation authority, incremental layout/paint invalidation,
virtualized projection, accessibility semantics, and product profiling gates.
