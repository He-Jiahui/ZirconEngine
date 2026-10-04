---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-19-accesskit-tree-projection-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/706-accessibility-visibility-detached-scratch-reuse.md
implementation_files:
  - zircon_runtime/src/ui/accessibility/accesskit.rs
  - zircon_runtime/src/ui/accessibility/accesskit/performance_tests.rs
tests:
  - tools/tests/test_runtime_accesskit_tree_projection_capacity_performance_contract.py
  - tools/tests/test_runtime78_accesskit_focus_membership_performance_contract.py
---

# Runtime809 · AccessKit tree projection capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A/Runtime78 AccessKit tree projection | Reserve exact snapshot, synthetic-root, and direct child bounds before projection, preserving node order, root/focus semantics, and AccessKit properties. | TDD source/model contract `4/4`; lower Rust source regression and ignored `RUNTIME809_ACCESSKIT_TREE_PROJECTION_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-node model removes twelve modeled geometric growth events. The twelve-slice focused batch passes `48/48`; the strict non-tooling performance/pressure batch passes `2643/2643` across `682` files in `51.429s` with zero failures, errors, or skips. Managed Windows/Cargo/Release and AccessKit product percentile evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only transient vectors in the AccessKit adapter. It does not
change accessibility extraction, node identity, relation/state/action mapping,
focus fallback, platform adapter ownership, or public DTOs.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/accessibility/accesskit.rs` | `FD3194C2D730D8672F23BCC36BF8C7AD6D72E1BC3B65D541B618677F36185BFE` |
| `zircon_runtime/src/ui/accessibility/accesskit/performance_tests.rs` | `F22ACCC674947F3A1071C66ADA4F582E6AB02C3AE165CF2E825C7996B415A11C` |
| `tools/tests/test_runtime_accesskit_tree_projection_capacity_performance_contract.py` | `EE51DD1D7D7A2EE00B5B33506B7F684A5424552A31CEA20DF1419B6D5E072C80` |
| `tools/tests/test_runtime78_accesskit_focus_membership_performance_contract.py` | `9505CDF1C46C49B03EAD8293BEAF420BD4D4452823D3CCD8B1326BC2CD6BF29E` |

## Managed gate

No Cargo process is started locally and the external coordinator is not
polled. Local source/model evidence is retained for the merged validation
batch; tooling production remains deferred for the later Rust migration.
