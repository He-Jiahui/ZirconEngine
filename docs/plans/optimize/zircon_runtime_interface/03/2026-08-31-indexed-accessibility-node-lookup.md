# RuntimeInterface03 Indexed Accessibility Node Lookup

## Status

`implementation_complete; managed_validation_pending`

## Scope

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Parent plan: `03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- Finding: P2-04, repeated accessibility snapshot node lookup used a linear scan.

## Change

`UiAccessibilityTreeSnapshot::node` now tries `binary_search_by_key` before the
existing borrowed linear lookup. Runtime accessibility extraction already
materializes `nodes` from `BTreeMap::into_values()`, so production snapshots use
the ordered fast path without a second index allocation. Unsorted legacy or
externally deserialized snapshots retain the old lookup behavior through the
fallback.

The public DTO fields and serialized wire shape are unchanged. Lookup remains
borrowed and does not clone nodes, strings, actions, or child lists.

## Performance Contract

- Sorted production lookup: `O(N)` comparisons -> `O(log N)` comparisons.
- Per-lookup allocation count: `0 -> 0`.
- Benchmark fixture: 4,096 sorted nodes, 100,000 near-tail successful probes,
  11 alternating old/new samples.
- Acceptance threshold: indexed P95 must be at least 20% lower than linear P95.
- Exact P50/P95: pending managed Windows release benchmark output.

## Verification

- TDD RED: the focused static contract failed because the prior body did not
  contain `binary_search_by_key`.
- Focused static contract after implementation: `3/3` passed.
- Python compile check: passed.
- `git diff --check` for the implementation/test/guard slice: passed.
- Managed `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  --jobs 1`: pending asynchronous coordinator validation.
- Managed ignored release benchmark: pending the same batch.

No commit, push, or WeCom notification is permitted until the managed batch is
terminal-successful and the coordinator finalizes the attributed union.
