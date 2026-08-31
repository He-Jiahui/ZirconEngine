---
title: Runtime07 VM Qualified Family Index
category: zircon_runtime
report_id: Runtime07-vm-qualified-family-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 VM Qualified Family Index

## Scope

This slice removes registry-wide `Arc` cloning and family-name scanning when a VM selector already
names a registered backend family. Known-family error propagation, unknown-family cross-family
fallback, selector ordering, registration replacement, and poisoned-lock recovery remain unchanged.
It advances Runtime07 backend selection without claiming completion of VM sandboxing, ABI,
debugging, package verification, resource budgets, or production language backends.

## Change

- Parse the qualified family prefix before materializing the fallback family snapshot.
- Borrow the prefix for a direct `BTreeMap` lookup and clone only the selected family `Arc`.
- Release the registry lock before invoking the selected backend family.
- Materialize the full family snapshot only for unqualified selectors or unknown prefixes.

## Deterministic Performance Evidence

| Qualified lookup with 4,096 registered families | Before | After |
|---|---:|---:|
| Family `Arc` clones | 4,096 | 1 |
| Linear family-name comparisons | 4,096 | 0 |
| Ordered-map lookups | 0 | 1 |
| Unrelated family `resolve` calls | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME07_VM_QUALIFIED_FAMILY_INDEX_BENCH_V1`. Acceptance requires direct qualified lookup P95 to
be at least 90% below the legacy registry snapshot and scan. Exact Windows timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826bf_qualified_family_lookup_preserves_fallback_semantics` covers
  target-only dispatch and unknown-prefix fallback.
- `optimization_batch_20260826bf_qualified_family_lookup_eliminates_registry_scan` proves zero
  family-name visits on a 4,096-family qualified lookup and locks the borrowed map lookup source
  contract.
- `optimization_batch_20260826bf_qualified_family_lookup_p95` reports paired release P50/P95
  samples and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Runtime07 still owns production backend completeness, package identity and verification, capability
budgets, deterministic scheduling, sandbox isolation, state migration, diagnostics, debugger
integration, and fault/soak evidence. This slice only converges registered-family selection.
