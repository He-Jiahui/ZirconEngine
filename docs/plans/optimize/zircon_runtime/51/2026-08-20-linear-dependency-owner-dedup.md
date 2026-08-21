# Runtime51 Linear Dependency Owner Deduplication

- Date: 2026-08-20
- Session: `optimize-runtime51-dependency-owner-dedup-r1-01a00797-20260820`
- Finding: `ASSETREG-P1-039`
- Performance marker: `PERF-MVP-556`
- Status: implementation complete; managed batch validation and release measurements pending

## Scope

`AssetRegistryIndex::refresh_dependency_owners` previously rejected duplicate resolved UUIDs with `Vec::contains`. For a high-fan-out owner, every dependency path could rescan the growing unique-dependency vector.

The targeted refresh now resolves each path once and admits its UUID through a per-owner `HashSet`. The output vector still preserves the first path that resolves to each UUID, and unresolved-path diagnostics retain their input order.

## Deterministic Work Reduction

The release workload contains 4,096 dependency paths and 256 unique dependency UUIDs in repeated first-seen order. The legacy implementation performs 526,080 linear UUID comparisons across one resolution pass. The optimized implementation performs 4,096 expected-constant-time hash admissions while retaining the same 256 ordered outputs.

This work-count comparison is deterministic and is not a timing claim. Release latency remains pending until the managed validator records actual samples.

## Acceptance Contract

- The behavior test requires first-path-order deduplication and preservation of unresolved dependency diagnostics.
- The ignored release benchmark runs 21 legacy/optimized sample pairs and alternates which implementation runs first.
- Each sample performs 16 full resolution iterations over the 4,096-path workload.
- P50 and P95 use nearest-rank selection.
- Optimized P95 must be at most 75% of legacy P95.
- The managed multi-task validator must parse the raw sample vectors and independently recompute the percentile and threshold checks before this record can be marked accepted.

## Validation

Scoped formatting and diff checks are required before the candidate snapshot. Cargo tests and release performance measurements are intentionally deferred to a managed multi-task validation batch; no passing result or measured latency is claimed in this record yet.
