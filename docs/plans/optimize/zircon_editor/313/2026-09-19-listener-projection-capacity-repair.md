# Editor819 Listener Projection Capacity Repair

- Date: 2026-09-19
- Related plan: `docs/plans/optimize/zircon_editor/313/2026-08-30-listener-projection-capacity-v2.md`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Performance status: `deterministic_target_met`

## Finding

The existing Editor313 lower contract required listener descriptor projection capacity, but the
current `projection.rs` still used iterator `.collect()` for descriptors and deliveries. This left
the recorded optimization unrepresented in the current source and would reintroduce geometric
growth for large listener snapshots and delivery pages.

## Repair

Both projections now allocate their exact input-length bound once and extend the mapped JSON
values into that buffer. Empty inputs remain zero-capacity; JSON field order, value ownership,
delivery order, and public response shape are unchanged.

## Regression contract

The existing lower Rust source/count contract and ignored marker
`EDITOR313_LISTENER_PROJECTION_CAPACITY_BENCH_V1` remain wired. The new Python source/model
contract is `tools/tests/test_editor_listener_projection_capacity_performance_contract.py` and
passes `3/3` after the intentional RED check. A deterministic 4,096-entry model changes `11`
geometric growth events to `0` for both projections.

## Validation boundary

The Editor819 contract is queued into the current Runtime/Editor focused batch. Scoped Rustfmt,
Python compilation, and source checks are local evidence only; managed Windows Cargo/Release,
allocator, and Editor event p50/p95/p99 gates remain pending. Tooling production remains deferred
for the later Rust migration.

The one-process focused Runtime/Editor batch subsequently passes `41/41` with
zero failures, errors, or skips. This receipt does not replace the managed
Cargo/Release or product percentile gates.

Editor821 additionally wires the existing lower Rust test module from
`projection.rs`, so the Editor313 regression and ignored marker are reachable
by Rust test discovery rather than only by source-contract inspection.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/editor_event/listener/projection.rs` | `7D96D20C7CCB304C85E4500CC90B586B136683044E9FB61A5C93E8748AB17C46` |
| `zircon_editor/src/core/editor_event/listener/projection/capacity_tests.rs` | `34BF7D1AD342F642AA7A947618CDEEF4EE133B8D5D9601C58BFCFD991B6CDABB` |
| `tools/tests/test_editor_listener_projection_capacity_performance_contract.py` | `9460665D76A2AE855BD7FC3B2CB85CD8F4B6AB8A6CC2D48F34DB4C99F2AA4CDE` |

The production and contract fingerprints above are the Editor819 repair snapshot. Editor821
adds only the test-module declaration and the strengthened reachability assertion; its current
fingerprints are recorded in
`docs/plans/optimize/zircon_editor/313/2026-09-19-listener-projection-test-wiring-repair.md`.
