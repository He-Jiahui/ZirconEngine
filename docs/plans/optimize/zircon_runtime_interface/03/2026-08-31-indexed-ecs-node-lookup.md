# RuntimeInterface03 Indexed ECS Projection Node Lookup

## Status

`implementation_complete; managed_validation_pending`

## Scope

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Parent plan: `03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- Finding: ECS projection node lookup repeated a linear scan over a stable ordered sequence.

## Change

`UiEcsProjectionSnapshot::node` now attempts `binary_search_by_key` before the
existing borrowed linear fallback. Runtime projection builds nodes from the
ordered `UiTree.nodes` map, while public constructors and deserialized payloads
may still be unsorted. The fallback preserves those inputs without changing
public fields or serialized shape.

## Performance Contract

- Sorted production lookup: `O(N)` comparisons -> `O(log N)` comparisons.
- Per-lookup allocation count: `0 -> 0`.
- Benchmark fixture: 4,096 sorted nodes, 100,000 near-tail successful probes,
  11 alternating old/new samples.
- Acceptance threshold: indexed P95 must be at least 20% lower than linear P95.
- Exact P50/P95: pending managed Windows release benchmark output.

## Verification

- Pre-change current-source audit recorded the exact linear body
  `self.nodes.iter().find(...)`; no pre-implementation static guard run was
  performed for this second, same-pattern slice.
- Focused static contracts after implementation: ECS `3/3`; combined
  accessibility + ECS `6/6`.
- Python compile check and `git diff --check`: passed.
- Managed `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  --jobs 1`: pending asynchronous batch submission.
- Managed ignored release benchmark: pending the same batch.

No commit, push, or WeCom notification is permitted until managed validation is
terminal-successful and the coordinator finalizes the attributed union.
