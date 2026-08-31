---
title: Editor02 Save Preflight Adjacent Dedup Optimization
category: zircon_editor
report_id: Editor02-save-preflight-adjacent-dedup-2026-08-24
date: 2026-08-24
session_id: root-editor02-save-preflight-dedup-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor02 Save Preflight Adjacent Dedup Optimization

## Scope

This slice removes the ordered-set allocation from dirty-document batch-save preflight. It advances
Editor02's Save All and close/save batch scalability without changing toolkit validation, failure
ordering, estimated-byte accounting, completion application, dirty generations, or save semantics.

## Implementation

`SaveDirtyViewsRequest::prepare` already sorts candidates by `DocumentId`. Duplicate detection now
compares each sorted document with its immediate predecessor instead of inserting every document
into a `BTreeSet`. The first candidate for each document remains accepted for duplicate checking,
and every subsequent equal candidate emits the same `DuplicateDocument` failure in the same sorted
position as before.

The separate completion-set validation still uses its ordered set because completions are not
pre-sorted and unknown completions must fail before dirty state changes.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Duplicate-check complexity after candidate sort | O(n log n) | O(n) | one predecessor comparison per candidate |
| Ordered-set entries for 65,536 candidates / 32,768 unique documents | 32,768 | 0 | 100.0000% entry reduction |
| Auxiliary duplicate-check collection | `BTreeSet<DocumentId>` | `Option<DocumentId>` | zero heap allocation for duplicate tracking |
| 65,536-candidate release p95 | dynamic evidence pending | <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 21 legacy/optimized sample pairs and prints
`EDITOR02_SAVE_PREFLIGHT_DEDUP_BENCH_V1` with exact p95 nanoseconds, candidate and unique-document
counts, ordered-set entry counts, and the deterministic entry reduction. Dynamic elapsed time is accepted
only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, sorted duplicate regression, and the production
  adjacency source contract are performed before coordinator submission.
- The focused regression and ignored release performance evidence are queued only in a shared
  Runtime/Editor coordinator batch; no per-task Cargo lane is launched.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

Editor02 still owns the product Save All/close prompt path, atomic file replacement, autosave and
recovery lifecycle, document-scoped history cleanup, and large-project qualification. Those parent
milestones remain separate and are not claimed complete by this preflight optimization.
