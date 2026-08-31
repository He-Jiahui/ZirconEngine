# RuntimeInterface03 Renderer Parity Batch Mapping

## Status

`implementation_complete; managed_validation_pending`

## Scope

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Parent plan: `03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- Finding: every renderer parity paint row linearly searched all batch source-index lists.

## Change

`UiRendererParitySnapshot::from_paint_elements_batches` now builds one
source-index to batch-index table before projecting paint rows. The table is
borrowed from the existing batch plan, preserves the first matching batch when
duplicate source indices occur, ignores malformed out-of-range indices, and
leaves the public parity DTO and serialized fields unchanged.

## Performance Contract

- Batch lookup work during parity projection: `O(N * B)` source-list scans ->
  `O(N + S)` one-pass mapping, where `S` is total batch source-index entries.
- Per-row lookup allocation: `0 -> 0`; one bounded mapping allocation replaces
  repeated scans.
- Benchmark fixture: 4,096 elements, 512 batches of eight source indices,
  11 alternating samples.
- Acceptance threshold: indexed P95 must be at least 20% lower than linear P95.
- Exact P50/P95: pending managed Windows release benchmark output.

## Verification

- TDD RED: static contract failed before the mapping helper and benchmark
  existed.
- Focused static contract after implementation: `3/3` passed.
- Python compile check and `git diff --check`: passed.
- Managed `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  --jobs 1`: pending asynchronous batch submission.
- Managed ignored release benchmark: pending the same batch.

No commit, push, or WeCom notification is permitted until managed validation is
terminal-successful and the coordinator finalizes the attributed union.
