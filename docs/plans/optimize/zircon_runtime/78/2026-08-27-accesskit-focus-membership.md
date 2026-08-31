---
title: Runtime78 AccessKit Focus Membership
category: zircon_runtime
report_id: Runtime78-accesskit-focus-membership-2026-08-27
date: 2026-08-31
session_id: root-runtime78-accesskit-focus-release-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime78 AccessKit Focus Membership

## Scope

`snapshot_to_accesskit_tree_update` previously converted the complete accessibility snapshot into
the final AccessKit node vector and then copied every node id into a temporary `BTreeSet`. That
index existed only to validate one optional focused node before selecting the already-resolved root
as fallback.

The converter now checks the focused id directly against the final node vector with `any`. The
output nodes, their order, the multi-root synthetic node, valid focused ids, and invalid-focus root
fallback remain unchanged. Because the membership scan runs after the synthetic node is appended,
it preserves the old index's exact membership set, including the reserved synthetic id.

Release-r2 extracts that membership policy into `accesskit_focus_node_id` so the production call
site and the ignored release benchmark execute the same optimized helper. The helper remains
private to the AccessKit adapter and does not add a second retained index or public API.

This slice removes one full-snapshot transient index. It does not claim to implement incremental
publication, a retained semantic-tree index, action generation binding, or a product AccessKit
window adapter.

## Behavior Evidence

- `accesskit_tree_update_maps_roles_actions_bounds_children_and_focus` covers a valid focused node
  while preserving node order and role/action conversion.
- `accesskit_tree_update_preserves_text_values_slider_numeric_state_and_relations` covers an
  unknown focused id falling back to the resolved root.
- `test_runtime78_accesskit_focus_membership_performance_contract.py` rejects the temporary
  `BTreeSet`, requires direct membership over final nodes, and freezes focus mapping, root fallback,
  output-node construction, and synthetic-root insertion.
- The deterministic model asserts exact equivalence for a valid final node, a missing id, and the
  synthetic-root id before collecting timing or allocation evidence.

## Deterministic Performance Model

The historical optimized release model uses 16,384 snapshot nodes plus the final synthetic root.
It validates a focused node at the end of the real snapshot, so both implementations inspect the
complete id set. Each sample batches four selections after five warmups; 31 indexed/linear sample
pairs alternate execution order.

| Metric | Temporary BTreeSet | Final-node scan | Reduction |
|---|---:|---:|---:|
| allocations per selection | 1,493 | 0 | 100.000% |
| allocated bytes per selection | 429,224 | 0 | 100.000% |

| Run | Indexed P50 ns | Linear P50 ns | Reduction | Indexed P95 ns | Linear P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1,914,100 | 42,500 | 97.780% | 2,860,800 | 61,700 | 97.840% |
| 2 | 1,664,500 | 42,100 | 97.470% | 2,430,900 | 47,500 | 98.050% |
| 3 | 1,878,800 | 43,500 | 97.680% | 2,290,400 | 60,300 | 97.370% |
| 4 | 2,023,800 | 44,400 | 97.810% | 3,390,800 | 69,200 | 97.960% |

The four-run worst case reduces P50 by 97.470% and P95 by 97.370%. The managed gate requires zero
linear-scan allocations/bytes, at least 90% lower P50, at least 85% lower P95, exact result checksum
`16384`, and nonzero timing checksum `4063604`.

This isolated model measures focused-id membership only. It excludes AccessKit node conversion,
string/property cloning, tree extraction, OS publication, screen-reader latency, frame time, power,
and external-engine performance.

Release-r2 adds the same workload as an in-crate Rust benchmark over real `accesskit::Node` values:
16,384 nodes, four membership selections per sample, four paired warmups, and 21 alternating sample
pairs. It emits both raw arrays with nearest-rank P50/P95. The managed gates require result parity,
nonzero checksums, `65,536 -> 0` temporary index entries per sample, at least 90% lower P50, and at
least 85% lower P95.

## Validation

Passed locally without Cargo on baseline HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784`,
coordinator epoch `575`:

- release-r2 TDD RED: the helper-sharing guard failed and the missing Rust benchmark module raised
  one expected error, while the other three source contracts passed;
- release-r2 GREEN: 5/5 Python source/performance contracts;
- `rustfmt +1.94.1 --check` and scoped diff checks;
- four independent optimized release-model runs with exact result parity and every gate met.

Release-r2 validation request `b7c0594b13be4762ba534ed748273e60` batches the behavior-parity
test and ignored production-helper performance gate under one `runtime78_accesskit_focus_` filter.
The managed command is `cargo +1.94.1 test -p zircon_runtime --locked --release --jobs 1 --
runtime78_accesskit_focus_ --include-ignored --nocapture --test-threads=1`, with two expected tests.
Cargo validation is not claimed until that asynchronous ticket reaches a passing terminal state
and returns both raw arrays plus exact P50/P95.

## Remaining Parent-Plan Work

Runtime78 still owns explicit semantic authoring, qualified publication generations, incremental
changed/removed sets, retained indexes, complete relation/state/text/range/live-region mapping,
per-window platform adapters, atomic action receipts, fault handling, and real assistive-technology
qualification. This slice only removes a redundant per-snapshot focus-membership index.
