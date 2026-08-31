---
title: Runtime07 Hash Package Resolution Sets
category: zircon_runtime
report_id: Runtime07-hash-package-resolution-sets-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Hash Package Resolution Sets

## Scope

This slice reduces membership-query cost in native package inventory construction for export
materialization. It contributes local package-resolution scale evidence for Runtime07 P1-3, but
does not complete the unified version/source/trust solver, catalog generation, or lock artifact.

The selected package ids remain a `BTreeSet`, package outputs remain a `BTreeMap`, and filesystem
entries remain lexically sorted. Only internal state that is queried for membership and never
iterated into observable output changes to hash sets.

## Change

- Build `unresolved_package_ids` as a `HashSet<&str>` after deterministic selection deduplication.
- Track resolved package directories in a `HashSet<PathBuf>` while preserving the sorted traversal
  and lexical first-match policy.
- Preserve the direct-child preference, early completion, symlink rejection, and immutable
  inventory behavior.
- Cover lexical nested fallback and duplicate selected ids with Rust regressions plus a source
  performance contract.

## Deterministic Performance Evidence

The standalone Rust model uses 4,096 selected packages and applies the same deterministic
`BTreeSet` selection step to both paths. The optimized path changes only unresolved-id and
resolved-directory membership state. The table records the conservative complete run; both paths
produced checksum `259983360`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Inventory P50 | 1.5213 ms | 1.1868 ms | 21.988% |
| Inventory P95 | 1.8714 ms | 1.4256 ms | 23.820% |

Evidence marker: `RUNTIME07_HASH_PACKAGE_RESOLUTION_SETS_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_hash_package_resolution_sets_performance_contract.py`: 3
  passed after the pre-change contract failed 3 of 3 checks.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Two Rust regressions cover lexical fallback order and duplicate selection deduplication.
- Managed Rust compilation and focused tests remain pending in an asynchronous coordinator batch;
  this candidate will be batched with another Runtime07 optimization rather than validated alone.

## Remaining Parent-plan Work

Runtime07 still owns the deterministic cross-backend resolver, package version/source/digest/trust
constraints, lock artifact, single catalog generation, transactional lifecycle, isolation,
execution budgets, and product-scale acceptance matrix in the canonical review.
