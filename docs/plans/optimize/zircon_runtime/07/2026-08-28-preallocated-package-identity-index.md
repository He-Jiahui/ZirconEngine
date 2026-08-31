---
title: Runtime07 Preallocated Package Identity Index
category: zircon_runtime
report_id: Runtime07-preallocated-package-identity-index-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Package Identity Index

## Scope

This slice reduces hash-table growth while building the package validation projection. It
contributes package-scale validation evidence for Runtime07, but does not complete the P1-3
cross-backend resolver, trust policy, catalog generation, or lock artifact.

The change affects only the internal `seen` identity index. Duplicate ordinals, manifest-order
projections, borrowed identity storage, membership indexes, and diagnostics remain unchanged.

## Change

- Count every identity row covered by package capabilities, roots, importers, dependencies,
  capability statuses, contributions, embedded features, interfaces, and modules.
- Allocate the `seen` hash set once with that exact row upper bound before indexing.
- Keep the normally empty duplicate-occurrence set allocation-free until a duplicate is found.
- Assert in debug/test builds that the capacity calculation still matches the number of indexed
  rows, so future manifest fields cannot silently leave the estimate stale.

## Deterministic Performance Evidence

The standalone optimized Rust model indexes 65,536 unique identities from a valid package shape
for 17 alternating samples. This isolates the normal accepted-manifest path and includes the
structural capacity-counting pass. Both implementations produced checksum `65536`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Hash-table allocation calls | 16 | 1 | 93.750% |
| Requested allocation bytes | 2,359,516 | 1,179,664 | 50.004% |
| Projection P50 | 7.2964 ms | 5.2938 ms | 27.446% |
| Projection P95 | 10.5983 ms | 8.1564 ms | 23.040% |

Evidence marker: `RUNTIME07_PREALLOCATED_PACKAGE_IDENTITY_INDEX_MODEL_V1`.

A separate malformed workload with 25% duplicate identities reduced allocation calls but did not
show stable latency improvement because the row upper bound intentionally exceeds the number of
unique keys. No latency claim is made for duplicate-heavy manifests; they are rejected validation
inputs rather than the normal publication path.

## Validation

- `python tools/tests/test_runtime07_preallocated_package_identity_index_performance_contract.py`:
  3 passed after the pre-change contract failed all 3 checks.
- Existing Rust regressions cover duplicate ordinals, manifest-order projection, and membership.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in the asynchronous Runtime07 batch.

## Remaining Parent-plan Work

Runtime07 still owns the deterministic resolver, package version/source/digest/trust constraints,
single catalog generation, transactional lifecycle, isolation, execution budgets, and product-scale
acceptance matrix in the canonical review.
