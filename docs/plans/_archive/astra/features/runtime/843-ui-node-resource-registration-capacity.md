---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a/2026-09-20-ui-node-resource-registration-capacity.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/template/asset/surface_index/node_resource_registration.rs
tests:
  - zircon_runtime/src/ui/template/asset/surface_index/capacity_tests.rs
  - tools/tests/test_runtime_ui_node_resource_registration_capacity_performance_contract.py
---

# Runtime843 · UI node resource registration output capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11a retained UI asset index | Lazily reserve the resource-free node report on the first empty projection and seed each metadata collector with the saturating authored-map key bound; preserve URI scheme filtering, fallback policy, first-seen order, per-node ownership, and stale-edge removal. | TDD RED→GREEN source/model contract `4/4`; combined Runtime/Editor repair/capacity batch `40/40` in `0.046s`; batched non-tooling performance/pressure loader `2366/2366` across `647` modules in `5.524s`, with zero load errors/failures/errors/skips; lower collector-capacity regression and ignored `RUNTIME843_NODE_RESOURCE_REGISTRATION_CAPACITY_BENCH_V1` marker are wired; deterministic `4,096`-node model changes `11→0` growth events. Managed Cargo/Windows Release, allocator, and UI asset registration product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## 性能边界

The lazy report reservation keeps the all-resource path at zero output allocation while removing
geometric growth when a dense tree emits resource-free nodes. The per-node URI bound is a local
lower bound from already-authored metadata maps; no second metadata walk or new ownership
authority is introduced.

## 当前指纹

| File | SHA-256 |
| --- | --- |
| `node_resource_registration.rs` | `363F6F5F00928FB9DAB8902DBE37B43317C4B115DA44C0917B6D5C113E50CF65` |
| `capacity_tests.rs` | `DCCB9BFA9E9212330A2CBEFA0637110275737ED94063C3849CF3AFCAFA1CA88B` |
| `test_runtime_ui_node_resource_registration_capacity_performance_contract.py` | `684D4CA6F7933DB457D60E4AC20B360738E7DAAB272B37CCAFCE1E74BBF46D03` |

## 受管验证

This slice is handed to the existing batched Runtime/Editor Windows Release lane. No standalone
Cargo process or coordinator status query was started. Keep this record at
`implemented_pending_validation` until current-source compilation, lower Rust behavior tests,
Release allocation evidence, and UI asset registration product percentile gates arrive. Tooling
production work remains deferred for the later Rust migration.
