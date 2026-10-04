---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-16-focus-navigation-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/tree/node/focus.rs
tests:
  - zircon_runtime/src/ui/tree/node/focus/navigation_capacity_tests.rs
  - tools/tests/test_runtime_tree_focus_navigation_capacity_performance_contract.py
---

# Runtime791 · Focus navigation output capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A P1-14 focus navigation | Reserve the tree node-count upper bound before recursive focus-candidate collection, preserving root/child order and all candidate semantics. | TDD RED→GREEN source contract `3/3`; enabled-focusable order/count regression; ignored `RUNTIME791_FOCUS_NAVIGATION_CAPACITY_BENCH_V1`; deterministic 4,096-node model `11→0`; managed Cargo/Release, allocator, and product percentile evidence remain pending. | implemented_pending_validation |

## Implementation evidence

`focusable_nodes_in_navigation_order` now starts with
`Vec::with_capacity(self.nodes.len())`. The bound is deliberately documented as
an upper bound: detached, hidden, or non-focusable nodes can make the reserved
capacity larger than the final output, while no valid traversal can emit more
IDs than `UiTree.nodes` contains. The lower Rust module verifies exact source
order and the capacity invariant on 128 enabled focusable roots.

The ignored Release benchmark alternates legacy and reserved collection on a
4,096-root tree, reports paired p95/raw timings, and models geometric growth
without asserting a fragile local timing ratio. It is held for the combined
owner-attributed Windows lane.

The refreshed single-process Runtime/Editor performance-contract batch loads
553 files and passes `1978/1978` tests in `5.513s`, with zero failures, errors,
or skips. This is local source/model evidence only.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/tree/node/focus.rs` | `8B43174C3B4CF5FF6EEF6949E01F38CEEEE973B5260DCCC3E7AB3EDFF1A67CCB` |
| `zircon_runtime/src/ui/tree/node/focus/navigation_capacity_tests.rs` | `0857C419E571E01864E1B98CC2C3B0D8EC0297F4A9B859901BB77B5B4A8C21D7` |
| `tools/tests/test_runtime_tree_focus_navigation_capacity_performance_contract.py` | `A710C128D5D6A071C6048963BFCDE14A1D7730912D69366C2F9F99678AF041F3` |

## 性能与受管验证边界

The deterministic model removes 11 modeled vector-growth events for the
4,096-node fixture (`11→0`). The local source contract and Rustfmt checks are
green, as does the 553-file/1978-test Runtime/Editor contract batch, but these
receipts do not establish Rust Cargo compilation, Windows Release allocation
counts, or product navigation p50/p95/p99. Keep the status
`implemented_pending_validation` until the asynchronous managed batch supplies
those gates. This session did not query or poll the coordinator, and tooling
remains intentionally deferred.
