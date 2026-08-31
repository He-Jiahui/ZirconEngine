---
title: Editor04 Folder Construction Hotpath Optimization
category: zircon_editor
report_id: Editor04-folder-child-hash-admission-2026-08-24
date: 2026-08-24
session_id: root-editor04-folder-child-hash-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Folder Construction Hotpath Optimization

## Scope

This slice removes growing-vector duplicate scans and repeated ancestor-path construction from
asset-catalog folder generation. It advances Editor04's large-catalog projection path without
changing folder identity, parent links, recursive asset counts, direct-asset ordering, or the
published child-folder ordering.

## Implementation

`FolderBuilder` now admits child folder IDs into a request-local `HashSet`. Repeated assets below
the same folder therefore reuse a constant-time membership path, while projects with many sibling
folders avoid scanning every previously discovered sibling for every new child.

Before the public DTO is built, the set is converted to a vector and sorted by display name with
the folder ID as the deterministic tie-breaker. The externally visible ordering is therefore the
same as before; the hash set exists only during catalog generation.

When a terminal folder has already been created by an earlier asset, subsequent assets now update
that folder directly. Only the first asset under a folder constructs every ancestor and publishes
the parent-child hierarchy. Repeated assets still increment the terminal folder's direct and
recursive counts, while avoiding redundant path-segment formatting and tree lookups.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 4,096 unique sibling folders, 256-byte IDs | 8,386,560 linear comparisons | 4,096 hash admissions | 99.9512% fewer admission operations |
| 4,096 assets in one 8-level folder | 32,768 segment path builds | 4,104 terminal/segment path builds | 87.4756% fewer path builds |
| Duplicate child IDs | scanned then rejected | rejected by set insertion | same output |
| Published ordering | display name, then ID | display name, then ID | unchanged |
| Release p95 | dynamic evidence pending | <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 21 legacy/optimized sample pairs for each
task. It prints `EDITOR04_FOLDER_CHILD_HASH_ADMISSION_BENCH_V1` and
`EDITOR04_FOLDER_TERMINAL_CACHE_BENCH_V1` with exact p95 nanoseconds and their structural counts.
Dynamic elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, deterministic order/path regressions, and the
  production hash-admission/terminal-hit source contracts are performed before submission.
- Six focused regression/performance tests for both folder tasks are queued in one coordinator
  batch; no per-task Cargo lane is launched.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

Editor04 still owns the broader runtime/editor asset authority convergence, background catalog
preparation, paged queries, import coordination, preview scheduling, reference projection, and
large-project qualification described by its parent plan. Those milestones remain separate and are
not claimed complete by these folder-construction optimizations.
