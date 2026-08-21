# Runtime63 Allocation-Free Single Resource Reflection Write Optimization Record

- Date: 2026-08-21
- Owner: `optimize-runtime63-single-resource-write-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md`, RSR-P1-045
- Status: implementation complete; combined managed validation pending

## Problem

`WorldReflection::reflect_write` resolves one resource field to its dense slot,
then wrapped the single `(slot, value)` pair in a `Vec` before calling the batch
adapter. Every Inspector, remote, or scripting resource-property write therefore
paid one heap allocation even though the operation was explicitly singular.

## Change

- `ReflectResource` now declares a dedicated `write_field_by_slot` adapter in
  addition to the transaction-oriented batch adapter.
- The ordinary reflection facade calls the single-slot adapter directly after
  slot resolution. It no longer constructs a single-element batch.
- All existing resource adapters implement the single-slot contract explicitly;
  DynamicScene transaction publication retains the existing atomic batch path.
- The facade behavior regression records the selected route and requires exactly
  one single-slot call and zero batch calls.

## Deterministic Performance Evidence

The managed release gate performs 100,000 alternating writes per sample through
the same `ReflectResource` adapter:

| Measure | Legacy | Optimized | Gate |
|---|---:|---:|---:|
| Writes per sample | 100,000 | 100,000 | exact |
| Single-element `Vec` allocations | 100,000 | 0 | eliminated |
| Timing distribution | 21 samples | 21 samples | alternating first-run order |
| Nearest-rank P95 | pending | pending | optimized <= 75% of legacy |

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

The pinned Runtime63 child validator is
`zircon-validation-runtime63-single-resource-write.ps1` at SHA-256
`58B35F52746B0C0AAC8DC011DC0DF226E7F6925DC783A1A06B67A3A71B6EF9FE`.
It is aggregated with four other Rust optimization tasks by
`zircon-validation-runtime-rust-followup-five.ps1` at SHA-256
`BCB55FF500D73C323BA9F3E5B36852873B4819B2F8C46B9FEE20A00D1056DD47`.

## Acceptance

- Resource reads, changed writes, unchanged writes, structured errors, and
  resource change ticks retain their prior behavior.
- `resource_reflection_single_write_release_benchmark` emits raw 21-pair
  distributions, recomputable nearest-rank P50/P95 values, and exact allocation
  counts.
- Exact-file Rustfmt, scoped diff checks, Cargo regressions, and release timing
  are required in one managed multi-task Windows validation copy. No per-task
  Cargo invocation is used.

## Remaining Scope

This slice removes the avoidable container allocation from already-resolved
single resource writes. It does not implement the broader compiled
`PropertyPlan`, catalog generation, or multi-field transaction work required by
RSR-P1-045 and adjacent Runtime63 findings.
