---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime581-visible-spatial-hash-index.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
tests:
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
---

# Runtime888 Runtime581 Visible-Spatial Hash Index

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Visible spatial query | Immutable visible entries use a capacity-sized `HashMap<u64, VisibleSpatialEntry>` for private candidate lookup; entity publication still sorts and deduplicates, and bounds/ray statistics remain unchanged. | Runtime581 behavior/source contracts cover sorted hits, fallback behavior, and the production hash owner. |
| 性能门禁 | 32,768 candidate keys replace ordered-map probes with expected constant-time lookup. | ignored marker `RUNTIME581_SPATIAL_QUERY_HASH_LOOKUP_BENCH_V1` requires optimized P95 ≤70% of the legacy ordered-map path; managed Runtime Release receipt remains pending. |

- `spatial_query.rs` contains the Runtime581 behavior/source contracts and marker.
- The source plan records Rustfmt/scoped diff evidence and keeps managed Windows Release as the acceptance gate.
- No tooling changes; this record registers the existing optimized source without claiming a Cargo or P95 result.

### Grouped validation submission (2026-09-25)

Runtime581 is included in the shared `optimization_batch_gz` wave: Runtime
development PTY `76539`, Editor development PTY `36565`, Runtime02 Release PTY
`51493`, and Editor Release PTY `35753`. The wrappers remain intentionally
unpolled; compiler, functional-test, and P95 receipts are pending.

The exact `optimization_batch_gz` replacement wave was submitted as Runtime
development PTY `81972`, Editor development PTY `21257`, Runtime02 Release PTY
`20043`, and Editor Release PTY `50976`. The earlier broad-prefix wave did not
select this historical marker; all four replacement wrappers remain unpolled.
