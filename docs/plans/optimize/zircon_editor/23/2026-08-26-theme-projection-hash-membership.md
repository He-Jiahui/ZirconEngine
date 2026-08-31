---
title: Editor23 Theme Projection Hash Membership
category: zircon_editor
report_id: Editor23-theme-projection-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Theme Projection Hash Membership

## Scope

This slice reduces membership and allocation cost in theme clone/compare/action projection. It
preserves document/import traversal, first-match and last-cascade semantics, and ordered action
projection from the remaining `BTreeMap` outputs. It does not change theme identity or transaction
architecture.

## Change

- Build borrowed local-style import membership in `HashSet<&str>`.
- Use a borrowed tuple-key `HashMap` for local stylesheet rule lookup.
- Use `HashSet<String>` for imported duplicate-rule signatures.
- Borrow the preferred stylesheet ID and allocate its variant prefix once before candidate scan.

## Deterministic Performance Evidence

| Representative theme projection work | Before | After |
|---|---:|---:|
| Imported rule membership | ordered O(log n) | average O(1) hash |
| Local rule tuple membership | ordered O(log n) | average O(1) hash |
| Nested import membership | linear `Vec::contains` | average O(1) borrowed hash |
| Preferred stylesheet ID clones | 1 | 0 |
| Variant-prefix allocations across `n` candidates | up to `n` | 1 |
| Release workload | 8,192 imported rules / 65,536 lookups | same rules and lookups |

The ignored release gate alternates 17 ordered-membership and hash-membership samples and emits
`EDITOR23_THEME_PROJECTION_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash-membership P95 to
be at most 60% of ordered-membership P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826t_editor23_hash_rule_membership_matches_ordered_membership` proves
  the two index strategies produce the same duplicate result over the release workload.
- `optimization_batch_20260826t_editor23_theme_projection_uses_hash_membership` requires all
  membership-only hash indexes and the precomputed variant prefix while rejecting the tree set and
  repeated prefix allocation.
- `optimization_batch_20260826t_editor23_theme_rule_hash_membership_performance_evidence` emits
  both P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor23 still needs stable token/rule identity, typed variants and aliases, cross-asset usage and
transactions, lossless V2 roundtrip, atomic revisioned save, async bounded imports, and 1k/10k/100k
node and large-theme product qualification.
