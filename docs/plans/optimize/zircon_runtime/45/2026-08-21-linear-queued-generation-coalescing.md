# Runtime45 linear queued-generation coalescing

- Owner: `optimize-runtime45-linear-queue-coalescing-r1-01a00797-20260821`
- Source plan: `45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md`
- Finding: `PREF-P1-42`
- Status: implementation and deterministic evidence complete; combined managed Cargo validation pending

## Problem

`coalesce_queued_generation` removed every superseded queued generation with
`VecDeque::remove(index)`. When a large matching suffix was coalesced, every removal shifted the
remaining suffix and made the pass quadratic in the number of queued generations.

## Change

The queue is now partitioned in one stable pass. Retained entries are restored to the lane queue in
their original order, matching entries are returned in their original order, and the existing
terminal notification, reservation release, and saturation-counter updates run unchanged for every
superseded entry. The active-successor and newer-queued-successor decisions are unchanged.

## Deterministic evidence

The release workload contains 8,192 queued entries and coalesces the final 4,096 entries.

| Metric | Legacy repeated remove | Indexed linear partition | Reduction |
| --- | ---: | ---: | ---: |
| Queue element moves / visits | 8,386,560 | 8,192 | 99.902% |

The operation-count result follows directly from the workload: the legacy suffix removals move
`4,095 + ... + 1 + 0 = 8,386,560` queue elements, while the replacement examines every queued entry
once. Timing evidence is collected as 21 alternating legacy/optimized pairs with nearest-rank P50
and P95. The release gate requires optimized P95 to be at most 25% of legacy P95.

## Acceptance

- `bounded_keyed_io_matching_queue_partition_preserves_retained_and_removed_order` locks stable
  retained and removed ordering.
- `bounded_keyed_io_linear_queued_generation_coalescing_release_benchmark` verifies output equality,
  emits the raw 21-pair samples and percentile fields, and enforces the P95 threshold.
- The managed Runtime Rust follow-up batch runs the existing `bounded_keyed_io` regression filter
  and the ignored release gate together; no per-task Cargo process is launched from this session.

Pinned validation artifacts:

- Runtime45 child: `zircon-validation-runtime45-linear-queue-coalescing.ps1`, SHA-256
  `A4B50B971F30B89189B087323953F7AAC0A59EC1F046AB2AC90B97DCD73A7B9C`.
- Seven-task Runtime batch: `zircon-validation-runtime-rust-followup-seven.ps1`, SHA-256
  `496E330422E00EF3E1D8767B72C68FC92E59C3F77ED1AFD7839DC03106AE251E`.
- Both scripts parse with zero PowerShell AST errors.

## Remaining scope

This change closes the repeated-removal hotspot in queued-generation coalescing. Ordered insertion,
lane-wide scheduling, durability, multi-process coordination, and the remaining `PREF-P1-42`
acceptance items remain owned by their existing plan work.
