---
title: Editor06 Unstable Plugin Capability Projection
category: zircon_editor
report_id: Editor06-unstable-plugin-capability-projection-2026-08-26
date: 2026-08-26
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Unstable Plugin Capability Projection

## Scope

Each projected editor-plugin row owns a sorted, duplicate-free capability list. The projection now
uses `sort_unstable` before `dedup`, preserving serialized capability order and removing stable-sort
bookkeeping from every catalog refresh.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | --- |
| 2,048 row capabilities, 1,024 unique | stable sort `1` | unstable sort `1` | same serialized unique list |
| Windows-native release p95 | dynamic evidence pending | <= 95% of legacy p95 | stable sort bookkeeping removed |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`EDITOR06_UNSTABLE_PLUGIN_CAPABILITY_PROJECTION_BENCH_V1` with both p95 timings, sample/iteration/
entry/unique counts, and stable-sort counts. Exact elapsed-time evidence is accepted only from the
coordinator terminal receipt.

## Validation

- Functional coverage compares stable and unstable ordered-unique capability results.
- Source contracts prevent the stable row capability sort from returning.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the capability index task; no
  per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.
