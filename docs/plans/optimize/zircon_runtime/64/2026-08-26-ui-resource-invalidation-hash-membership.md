---
title: Runtime64 UI Resource Invalidation Hash Membership
category: zircon_runtime
report_id: Runtime64-ui-resource-invalidation-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime64 UI Resource Invalidation Hash Membership

## Scope

This slice removes nested URI scans from UI resource resolver cache invalidation. Trimmed input,
empty URI rejection, first-occurrence report order, primary and fallback matching, scheme mapping,
cache retention, removed counts, and diagnostic retention remain unchanged. It advances the
Runtime64 cache/dependency hot path without claiming completion of resource authority, version
leases, reload generation, cancellation, or product-scale qualification.

## Change

- Deduplicate trimmed URIs with an owned hash map carrying each first-seen index.
- Reconstruct the public requested URI report in original first-occurrence order.
- Borrow those owned strings into one hash set for cache membership checks.
- Map each primary/fallback reference URI at most once instead of once per requested URI.

## Deterministic Performance Evidence

| 512 references, 512 invalidations, four scans per sample | Before | After |
|---|---:|---:|
| Reference/invalidation pair checks per sample | 1,048,576 | 0 |
| Hash membership probes per sample | 0 | 4,096 |
| Runtime locator mapping attempts per sample | 1,048,576 | 2,048 |
| Requested URI report-order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME64_UI_RESOURCE_INVALIDATION_HASH_BENCH_V1`. Acceptance requires hash invalidation P95 to
be at least 90% below nested URI scans. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bb_resource_invalidation_preserves_order_and_matches` covers trim,
  empty/duplicate admission, first-seen order, primary/fallback matches, and legacy equivalence.
- `optimization_batch_20260826bb_resource_invalidation_uses_hash_membership` requires the ordered
  hash index and borrowed membership set and rejects the prior linear scans.
- `optimization_batch_20260826bb_resource_invalidation_hash_membership_p95` reports paired release
  P50/P95 samples and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Runtime64 still owns exact schema admission, asynchronous load tickets, version leases, cache
budgets, dependency SCC validation, reload/cancellation, project lifecycle, and large-scale fault
and concurrency evidence. This slice only converges UI resolver invalidation membership.
