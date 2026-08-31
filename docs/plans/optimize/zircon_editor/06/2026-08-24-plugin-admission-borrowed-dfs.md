---
title: Editor06 Plugin Admission Borrowed DFS Optimization
category: zircon_editor
report_id: Editor06-plugin-admission-borrowed-dfs-2026-08-24
date: 2026-08-24
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Plugin Admission Borrowed DFS Optimization

## Scope

This slice removes package-ID string cloning from the ordinary dependency-cycle admission path. It
advances Editor06 catalog admission without changing deterministic traversal, external-dependency
filtering, duplicate-package handling, or the owned cycle diagnostic returned to callers.

## Implementation

`find_dependency_cycle` now stores `&str` package IDs in its completed set, visiting set, and DFS
path. Those references remain valid for the lifetime of the immutable dependency map. An acyclic
package therefore enters and leaves the traversal without creating an owned copy of its ID.

When a cycle is found, only the reported cycle slice is converted to owned strings. This retains
the existing public error payload and exact path ordering while keeping the common success path
borrowed. The tree sets still allocate their own nodes; this change only eliminates package-ID
string clones and their owned bytes.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 4,096 acyclic packages, 512-byte IDs | 12,288 package-ID string clones | 0 package-ID string clones | 100.0000% clone reduction |
| Cycle diagnostic ownership | owned path | owned path | unchanged |
| Traversal ordering | sorted package/dependency order | sorted package/dependency order | unchanged |
| Release p95 | dynamic evidence pending | <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 21 legacy/optimized sample pairs and prints
`EDITOR06_PLUGIN_ADMISSION_BORROWED_DFS_BENCH_V1` with exact p95 nanoseconds, package count, ID
width, and package-ID clone reduction. Dynamic elapsed time is accepted only from coordinator
terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, cycle-path regression, and the production
  borrowed-ID source contract are performed before coordinator submission.
- The focused regression and ignored release performance evidence are queued with a Runtime task in
  one shared Runtime/Editor coordinator batch; no per-task Cargo lane is launched.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

Editor06 still owns the broader plugin-management authority, durable lifecycle operations,
dependency planning, settings contributions, authoring workflows, UI scalability, reload recovery,
and product qualification described by its parent plan. Those milestones remain separate and are
not claimed complete by this admission hot-path optimization.
