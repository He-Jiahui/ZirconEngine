---
title: Editor56 Dependency Reverse-edge Delta
category: zircon_editor
report_id: Editor56-dependency-reverse-edge-delta-2026-08-25
date: 2026-08-25
session_id: root-editor56-dependency-delta-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor56 Dependency Reverse-edge Delta

## Scope

This slice reduces reverse-index churn in Editor56's open UI-document dependency generation,
aligned with the plan's incremental index and project-scale performance direction. It does not
claim the parent plan's query runtime, provider registry, paged results, navigation receipts, or
cross-domain Find Usage work is complete.

## Implementation

`replace_dependencies` still normalizes and orders the incoming dependency set and preserves the
existing generation semantics. When the set changes, it now removes only `previous - next` reverse
edges and inserts only `next - previous` reverse edges before publishing the new per-instance set.

Unchanged reverse edges remain resident. This avoids deleting and recreating every edge, cloning
the entire next dependency set, and cloning the instance ID once per unchanged dependency. Impact
ordering and direct-over-import precedence remain unchanged.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 10K dependencies, replace one item | 20,000 reverse-edge mutations | 2 reverse-edge mutations | 99.99% mutation reduction |
| Unchanged dependency edges | 9,999 removals plus 9,999 inserts | 0 mutations | 100.00% unchanged-edge churn removed |
| Stored next set | full `BTreeSet` clone before iteration | ownership moved after delta apply | one 10K-set clone removed |
| Focused release wall-clock target | unbounded | <= 500 ms | pending terminal evidence |

The operation still performs normalization, next-set construction, and ordered set-difference
scans; this record does not claim constant-time replacement. The ignored Windows-native release
evidence prints `EDITOR56_DEPENDENCY_DELTA_BENCH_V1` with dependency count, changed-edge count,
legacy and optimized reverse-edge mutations, reduction percentage, elapsed milliseconds, and the
target. Exact wall-clock evidence is accepted only from the coordinator's terminal result.

## Validation

- RED proved the delta behavior tests referenced helpers absent from the full-rebuild path.
- Removed/added delta classification, shared-edge preservation, old-edge retirement, new-edge
  publication, and the ignored 10K release gate are prepared for a multi-task coordinator batch.
- Scoped `rustfmt --check`, `git diff --check`, and the reverse-edge delta contract pass locally.
- No local Cargo lane is launched and no coordinator compile is monitored in real time.
- Final validation ticket, terminal marker values, integration commit, and WeCom delivery remain
  pending.

## Documentation Decision

The public Editor search documentation does not promise the internal open-document reverse-index
update algorithm. Dependency impact and ordering semantics are unchanged, so this scoped
optimization record is the only documentation change.

## Remaining Parent-plan Work

Capability-truth fixes, typed queries, provider and operation lifecycles, indexed project search,
paged results, hierarchy integration, cross-domain Find Usage, navigation receipts, and full
million-item qualification remain open under Editor56.
