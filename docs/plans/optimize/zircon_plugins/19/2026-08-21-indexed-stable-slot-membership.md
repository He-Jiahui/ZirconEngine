# Plugins19 Indexed Stable-Slot Membership Optimization Record

- Date: 2026-08-21
- Owner: `optimize-plugins19-stable-slot-index-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/19-first-party-hybrid-gi-source-runtime-editor-dist-catalog-scene-representation-surface-cache-global-sdf-radiance-cache-probe-trace-denoise-product-integration-review.md`, HGI-V-P1-19
- Status: implementation complete; combined managed validation pending

## Problem

Surface-cache stable-slot assignment preserves valid previous slots before it
fills remaining capacity. The second pass previously checked every item against
the growing assigned `Vec` with a linear `any`. A steady 4,096-page frame whose
slots are all already stable therefore performed 8,390,656 item-ID comparisons
inside each atlas assignment, even though the decision is simple membership.

## Change

- Track assigned item IDs in a `BTreeSet` while valid previous slots are kept
  and while free slots are assigned.
- Preserve the existing item iteration, old-slot validation, slot-conflict,
  duplicate-item, free-slot ordering, and output ordering semantics.
- Add a focused equivalence regression containing duplicate item IDs, an
  out-of-capacity old slot, and two items competing for the same old slot.
- Add an ignored release gate with 4,096 stable items and 21 alternating
  legacy/indexed sample pairs.

## Deterministic Performance Evidence

The release workload supplies the same 4,096 item IDs, 4,096-slot capacity,
and complete previous-slot map to both implementations. Their output must be
identical before timing begins.

| Measure | Legacy Vec membership | Indexed membership | Gate |
|---|---:|---:|---:|
| Stable items | 4,096 | 4,096 | exact |
| Membership operations | 8,390,656 linear comparisons | 4,096 indexed lookups | 99.951% fewer operations |
| Timing distribution | 21 samples | 21 samples | alternating first-run order |
| Nearest-rank P95 | pending | pending | indexed <= 25% of legacy |

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

The pinned Plugins19 child validator is
`zircon-validation-plugins19-hybrid-gi-stable-slot.ps1` at SHA-256
`836B0F7680401EEC57F34AF247BFEE0745953F84256D2B7663306E4B0A71396C`.
It is aggregated with the existing nine plugin batches by
`zircon-validation-plugin-super-batch-ten.ps1` at SHA-256
`312AA0F567505293D9D8E8890467FE79F7227E667C0C471CF905FC37EAD41B35`.
Both scripts have zero PowerShell AST parse errors.

## Acceptance

- The focused regression proves duplicate and invalid previous-slot behavior is
  unchanged, including output order.
- Existing Hybrid GI scene-representation tests continue to own page reuse,
  atlas-slot stability, invalidation, capture-slot, and dirty-page behavior.
- `surface_cache_indexed_stable_slot_membership_release_benchmark` emits 21 alternating raw
  sample pairs, recomputable nearest-rank P50/P95 values, and exact operation
  counts.
- Exact-file Rustfmt, scoped diff checks, Cargo regressions, and release timing
  are required in one managed multi-task Windows validation copy. No per-task
  Cargo invocation is used.

## Remaining Scope

This slice removes the quadratic membership scan inside stable-slot assignment.
It does not yet replace the surrounding per-frame Surface Cache snapshots,
temporary maps/sets, page-content rebuilds, GPU resource recreation, readback,
or global collector lock described by HGI-V-P1-19 and HGI-V-P1-38 through
HGI-V-P1-42.
