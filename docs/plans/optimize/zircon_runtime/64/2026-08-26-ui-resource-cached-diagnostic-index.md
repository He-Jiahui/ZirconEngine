---
title: Runtime64 UI Resource Cached Diagnostic Index
category: zircon_runtime
report_id: Runtime64-ui-resource-cached-diagnostic-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime64 UI Resource Cached Diagnostic Index

## Scope

This slice removes repeated full diagnostic scans when `resolve_dependencies` encounters cached UI
resource placeholders. New diagnostic ranges, cached primary/fallback association, global diagnostic
index order, duplicate suppression, fallback diagnostic-index recovery, resolved handles, report
resources, and final diagnostics remain unchanged. It advances Runtime64 cache reporting without
claiming completion of resource authority, version leases, reload, cancellation, or scale gates.

## Change

- Build one URI-to-global-diagnostic-index map at the start of a dependency report.
- Increment that index when an uncached resolution appends diagnostics.
- Resolve cached placeholder indices through primary/fallback hash lookups.
- Sort and deduplicate merged primary/fallback indices to preserve prior global order.

## Deterministic Performance Evidence

| 4,096 diagnostics, 512 cached lookups, four scans per sample | Before | After |
|---|---:|---:|
| Diagnostic/reference comparisons per sample | 8,388,608 | 0 |
| Diagnostic index-build visits per sample | 0 | 16,384 |
| Primary/fallback hash lookups per sample | 0 | 4,096 |
| Global diagnostic-index order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME64_UI_RESOURCE_CACHED_DIAGNOSTIC_INDEX_BENCH_V1`. Acceptance requires cached diagnostic
index P95 to be at least 80% below repeated full scans. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bc_cached_diagnostic_index_preserves_global_order` covers mixed
  primary/fallback order, same-URI deduplication, and fallback diagnostic-index recovery.
- `optimization_batch_20260826bc_cached_diagnostic_resolution_uses_uri_index` requires the URI
  index, incremental update, and sorted merge and rejects the cached full scan.
- `optimization_batch_20260826bc_cached_diagnostic_index_p95` reports paired release P50/P95
  samples and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Runtime64 still owns exact schema admission, asynchronous load tickets, version leases, cache
budgets, dependency SCC validation, reload/cancellation, project lifecycle, and large-scale fault
and concurrency evidence. This slice only converges cached UI resource diagnostic projection.
