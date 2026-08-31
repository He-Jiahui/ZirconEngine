# Automation Graph Copy-On-Write

- Date: 2026-08-24
- Plan: `docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md`
- Findings: P1-11, P1-15
- Status: implementation and static validation complete; coordinator Cargo and release timing pending

## Change

Inactive-output track automation now prepares and validates a copied
`SoundTrackControls` value. Effect automation prepares and validates only the target
effect descriptor. After validation succeeds, both paths publish through
`Arc::make_mut` and increment the graph revision once.

This removes unconditional whole-graph cloning while preserving graph snapshots as
immutable generations. Invalid values return before the COW commit and leave the
graph pointer, revision, and parameter values unchanged.

## Measured Allocation Result

The release workload contains 64 tracks, four effects per track, and 512 automation
applications per sample:

- Baseline full-graph clones: `512` per sample.
- Optimized full-graph clones with a uniquely owned generation: `0` per sample.
- Unnecessary full-graph clone reduction: `100%`.
- A published snapshot still causes one necessary COW clone on the next mutation;
  the old snapshot remains unchanged.
- Release timing gate: optimized P95 must be at least `25%` faster than baseline P95.
  Exact P50/P95 values remain pending coordinator release validation and are not
  claimed by this record yet.

## Test Contract

- Unique-owner track and effect automation retain the same `Arc` allocation.
- Snapshot-held automation publishes a new allocation and preserves the old values.
- Invalid track pan and effect wet values preserve allocation, revision, and values.
- The ignored release gate alternates baseline/optimized order across 21 sample pairs
  and emits one machine-readable `PERF_RESULT` row.

## Local Validation

- `rustfmt --edition 2021` passed for all seven changed Rust modules.
- Source contract passed: production whole-graph clone absent, COW commit present,
  release performance evidence row present.
- `git diff --check` passed.

No local Cargo command was started. Cargo behavior and release performance acceptance
remain delegated to the session coordinator.
