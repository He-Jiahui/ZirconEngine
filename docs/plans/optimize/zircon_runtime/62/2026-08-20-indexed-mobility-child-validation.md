# Runtime62 Indexed Mobility Child Validation

- Date: 2026-08-20
- Session: `optimize-runtime62-mobility-child-index-r1-01a00797-20260820`
- Finding: `RSH-P1-043`
- Performance marker: `PERF-MVP-558`
- Status: implementation complete; managed batch validation and release measurements pending

## Scope

`World::validate_mobility_change` previously scanned every stable entity when changing a node to
`Mobility::Dynamic`, even though `HierarchyMutationIndex` already maintains stable-order direct-child
buckets. The cost therefore grew with total World size rather than the affected node's child count.

The validation path now queries the maintained direct-child bucket and stops at the first Static
child. If the mutation index is not current, it retains the prior stable-order full-World scan as a
correctness fallback. Static-under-Dynamic validation remains the existing direct parent lookup.

## Deterministic Work Reduction

The release workload contains 8,192 entities, one target parent, and eight direct children. Each
sample performs 64 validations with no Static child so both implementations traverse their complete
candidate set. The legacy path visits 524,288 entities per sample; the indexed path visits 512 direct
children, a deterministic 1,024x reduction in candidate visits.

This work-count comparison is deterministic and is not a timing claim. Release latency remains
pending until the managed validator records actual samples.

## Acceptance Contract

- The behavior test places a Static direct child among 4,096 total entities and requires the parent
  transition to Dynamic to retain its typed rejection.
- The source contract test rejects a regression to `stable_entity_ids()` in the indexed hot path.
- The ignored release benchmark runs 21 legacy/indexed sample pairs and alternates which path runs
  first.
- Each sample performs 64 validations over the same 8,192-entity World.
- P50 and P95 use nearest-rank selection.
- Indexed P95 must be at most 25% of legacy P95.
- The managed multi-task validator must parse both raw sample vectors and independently recompute the
  percentile, work-count, and threshold checks before this record can be marked accepted.

## Validation

Scoped formatting and diff checks are required before the candidate snapshot. Cargo tests and
release performance measurements are intentionally deferred to a managed multi-task validation
batch; no passing result or measured latency is claimed in this record yet.
