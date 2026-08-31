---
title: Editor13 Unstable Preset Asset Dedup
category: zircon_editor
report_id: Editor13-unstable-preset-asset-dedup-2026-08-26
date: 2026-08-26
session_id: root-editor13-drawer-tab-dedup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Unstable Preset Asset Dedup

## Scope

The project preset asset projection filters locator-backed names, then sorts and deduplicates the
result before it reaches the editor menu. The output is an ordered unique list, so the stable sort
was replaced with `sort_unstable` without changing the public projection.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | --- |
| 2,048 asset locators, 1,024 unique names | stable sort `1` | unstable sort `1` | same ordered unique projection |
| Windows-native release p95 | dynamic evidence pending | <= 95% of legacy p95 | stable sort bookkeeping removed |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`EDITOR13_UNSTABLE_PRESET_ASSET_NAME_DEDUP_BENCH_V1` with both p95 timings, sample/iteration/
entry/unique counts, and stable-sort counts. Exact elapsed-time evidence is accepted only from the
coordinator terminal receipt.

## Validation

- Functional coverage checks the sorted unique preset projection and endpoint names.
- Source contracts prevent the stable asset-name sort from returning.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the global preset merge task;
  no per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.
