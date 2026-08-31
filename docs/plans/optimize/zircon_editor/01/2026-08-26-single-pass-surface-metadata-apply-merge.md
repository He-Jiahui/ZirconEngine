---
title: Editor01 Single-pass Surface Metadata Apply Merge
category: zircon_editor
report_id: Editor01-single-pass-surface-metadata-apply-merge-2026-08-26
date: 2026-08-26
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Single-pass Surface Metadata Apply Merge

## Scope

This slice removes temporary metadata-map clones when a retained surface applies authored metadata
to a dynamic node. Attribute/style ownership and override precedence remain unchanged.

## Implementation

`RetainedUiProjectionSurfaceMetadataIndex::apply_to` now extends the target maps from cloned
key/value pairs in the immutable index. The old clone-then-extend sequence allocated one temporary
attribute map and one temporary style map per patch; the direct iterator copies entries into the
caller-owned maps exactly once.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Temporary attribute maps per patch | 1 | 0 | direct entry merge |
| Temporary style maps per patch | 1 | 0 | direct entry merge |
| Windows-native release p95 | dynamic evidence pending | <= 85% of legacy p95 | coordinator gate |

The ignored release benchmark alternates 17 samples over 256 iterations and prints
`EDITOR01_SINGLE_PASS_SURFACE_METADATA_APPLY_MERGE_BENCH_V1` with both p95 timings, attribute/style
counts, and temporary-map counts. Exact elapsed-time evidence is accepted only from the coordinator
terminal receipt.

## Validation

- Functional coverage checks full metadata application and authored last-write precedence.
- A source contract prevents reintroduction of clone-then-extend in either map.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the index-merge task; no
  per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.

## Remaining Parent-plan Work

The parent Editor01 plan still requires generation-bound retained snapshots, delta invalidation,
virtualization, input/paint budgets, and current-source product profiling evidence.
