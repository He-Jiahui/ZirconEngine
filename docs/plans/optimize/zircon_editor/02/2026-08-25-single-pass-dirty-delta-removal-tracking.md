---
title: Editor02 Single-pass Dirty-delta Removal Tracking
category: zircon_editor
report_id: Editor02-single-pass-dirty-delta-removal-tracking-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor02 Single-pass Dirty-delta Removal Tracking

## Scope

This slice removes an ordered-set clone from `DirtyRegistry::changes_since` while preserving reset
handling, transaction-delta merging, removed-document ordering, generation retry semantics, and the
published snapshot/removal contract.

## Implementation

The retired path first filtered the external-change set into currently registered documents, then
cloned the complete `BTreeSet<DocumentId>` so the final snapshot loop could distinguish external
removals from transaction-only changes. The cloned tree duplicated every changed key and performed
another full tree traversal even though only removed document IDs were needed afterward.

`partition_external_changes` now makes one ordered pass while the registry generation is captured.
It builds the existing present set and directly appends absent documents to the final removal
vector. `changes_since` moves the original changed set into its existing merge path, so it performs
no full changed-set clone and retains deterministic document order.

## Performance Contract

| Evidence for 4,096 changed documents / 2,048 removals | Retired path | Optimized gate |
| --- | ---: | ---: |
| Changed-set clone entries | 4,096 | 0 |
| Removal-tracking passes over changed set | 2 | 1 |
| Ordered removal output | preserved | preserved |
| Alternating release benchmark | 11 samples x 64 partitions | optimized P95 <= 75% of retired P95 |

The benchmark emits `EDITOR02_SINGLE_PASS_DELTA_PARTITION_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/document counts, cloned-entry counts, and changed-set pass
counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, source-structure gates, and the focused ordering
regression are required before submission. One managed Editor02 Cargo invocation filtered by
`editor02_dirty_registry_` covers this regression and ignored release benchmark together with the
saved-effect clear optimization. Dynamic P95 evidence, integration SHA, and automatic WeCom
performance delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Editor02 still owns the product Save All and close prompt flow, write-time conflict protection,
autosave/recovery lifecycle, document-scoped history cleanup, and large-project qualification. This
micro-optimization does not claim those milestones complete.
