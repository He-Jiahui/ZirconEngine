---
title: Editor13 Unstable Preset Name Dedup
category: zircon_editor
report_id: Editor13-unstable-preset-name-dedup-2026-08-26
date: 2026-08-26
session_id: root-editor13-drawer-tab-dedup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Unstable Preset Name Dedup

## Scope

The editor's global preset list merges project asset names and configuration presets, then sorts
and deduplicates before presentation. Its required result is deterministic ordering with no
duplicate names, so the stable sort was replaced with `sort_unstable` while preserving the merge
and dedup semantics.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | --- |
| 4,096 merged names | stable sort `1` | unstable sort `1` | same deterministic unique list |
| Windows-native release p95 | dynamic evidence pending | <= 95% of legacy p95 | stable sort bookkeeping removed |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`EDITOR13_UNSTABLE_PRESET_NAME_MERGE_DEDUP_BENCH_V1` with both p95 timings, sample/iteration and
asset/config/merged counts, and stable-sort counts. Exact elapsed-time evidence is accepted only
from the coordinator terminal receipt.

## Validation

- Functional coverage retains the existing list ordering and duplicate elimination contract.
- Source contracts prevent the stable global-name sort from returning.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the asset projection task; no
  per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.
