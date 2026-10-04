---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-19-accessibility-root-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/root_capacity_tests.rs
tests:
  - tools/tests/test_runtime_accessibility_root_capacity_performance_contract.py
---

# Runtime838 · accessibility root projection capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A accessibility snapshot roots | Reserve the authored root-list bound before filtering and publishing admitted roots, preserving order, hidden/missing-root filtering, budget checks, and empty-path capacity. | TDD source/model contract `2/2`; lower Rust order/source regression and ignored `RUNTIME838_ACCESSIBILITY_ROOT_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-root model removes `11→0` geometric growth events; focused thirteen-contract batch `49/49` and refreshed broad non-tooling batch `3489/3489` across `861` modules pass. Managed Cargo/Release and accessibility product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only the temporary published-root vector capacity in the
Runtime accessibility extractor. It does not change node inclusion, relation
resolution, diagnostics, focus fallback, budget accounting, ordering, or
snapshot publication ownership.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/accessibility/extract.rs` | `F87AA27D67184EE005C45B9DB130D92E8FAAD44DCA701ECD7D6404ECA03F23B0` |
| `zircon_runtime/src/ui/accessibility/extract/root_capacity_tests.rs` | `D45B712C84C78DA199709CE7BC3C30015DB929858F6C85B51D7F26F0DDE86FCA` |
| `tools/tests/test_runtime_accessibility_root_capacity_performance_contract.py` | `1E5F85B6CD9276E339459DA2CD8BFA203D9FD4658BD14D948E7AA1A883173C43` |

## Managed gate

No Cargo process is started locally and the coordinator is not polled. Keep
this entry `implemented_pending_validation` until the owner-attributed batched
Windows Release lane proves current-source compilation, accessibility output
parity, allocation behavior, and Runtime accessibility product p50/p95/p99
evidence.
