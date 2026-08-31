---
title: Editor13 Default View Hash Admission
category: zircon_editor
report_id: Editor13-default-view-hash-admission-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Default View Hash Admission

## Scope

This slice reduces membership and duplicate-allocation cost while building default workbench view
instances. It preserves window traversal and first-seen `Vec` order, but does not change layout
restore transactions, registry authority, persistence schema, or native-window projection.

## Change

- Track admitted `ViewInstanceId` values in `HashSet<ViewInstanceId>` instead of `BTreeSet`.
- Check borrowed ID membership before cloning an ID into the set.
- Publish the original `ViewInstance` only on first admission.
- Preserve primary/drawer traversal and all host, title, payload, and dirty-state fields.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique view IDs | Before | After | Reduction |
|---|---:|---:|---:|
| Owned ID clones | 65,536 | 8,192 | 87.5% removed |
| Membership class | ordered O(log n) | average O(1) hash | tree traversal removed |
| Published instances | 8,192 | 8,192 | unchanged |
| Published order | first-seen | first-seen | unchanged |

The ignored release gate alternates 17 ordered-admission and borrowed-hash-admission samples. It
emits `EDITOR13_DEFAULT_VIEW_HASH_ADMISSION_BENCH_V1`; acceptance requires hash-admission P95 to
be at most 60% of ordered-admission P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826s_editor13_hash_admission_preserves_first_seen_order` covers
  duplicate suppression and first-seen order through the product helper.
- `optimization_batch_20260826s_editor13_default_views_use_hash_admission` requires borrowed
  lookup-before-clone and rejects the production tree-set path.
- `optimization_batch_20260826s_editor13_default_view_hash_admission_performance_evidence` emits
  workload/allocation counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor13 still needs transactional candidate restore, dirty-document close barriers, bounded
schema migration, last-known-good/quarantine, stable placeholder identity, monitor/DPI-aware native
placement, and full startup/restore scale qualification.
