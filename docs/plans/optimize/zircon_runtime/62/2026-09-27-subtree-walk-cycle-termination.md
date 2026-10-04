---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_derived_state.rs
  - docs/crates/zircon_runtime/scene/world/detached_entity_batch.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-detached-subtree-parent-cycle-rejection.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/subtree_cycle_walk.rs
  - zircon_runtime/src/scene/tests/derived_state/subtree_cycle_profile.rs
---

# Runtime62：损坏父链上的 subtree walker 有限终止

| 里程碑 | 范围 | 状态 | 日期 | 证据 |
|---|---|---|---|---|
| Runtime1013 | `derived_state` 六处 indexed/fallback subtree DFS 起点保护 | `implemented_static_review_passed_managed_validation_pending` | 2026-09-27 | [完成清单](../../../astra/features/runtime/1013-runtime62-subtree-walk-cycle-termination-completion-list.md)、[公共合同](../../../../zircon_runtime/scene/world/detached_entity_batch.md)、R 源快照与后续 managed 回执 |

## 已确认故障和最小修复

Runtime1012 的 `prepare_entity_subtrees` 在校验 requested roots 的父链前应用 deferred 组件通知。合法公开 `query::<Mut<ActiveSelf>>` 留下 pending mutation 后，通知经 `mark_inspection_subtree_fields_dirty` 调用 `subtree_entity_ids`。若 `get_mut::<Hierarchy>` 已造出 raw cycle 并使 hierarchy index 失效，旧 fallback DFS 会重复入环，新的 typed `HierarchyParentChainCycle` guard 尚未执行；当前 indexed 分支以及 `subtree_records`、`subtree_component_count` 的同类 DFS 也会重复入环。

每个 entity 只有一个 `Hierarchy.parent`，现有反向 child index 按稳定 entity 列表为每个 child 建立一条边。因此，从某个起点沿 child 边可达的环必包含该起点；在六处栈扩展时跳过返回起点的 child，能使 raw self/2/3-node cycle 与枝叶有限终止，且每个可达 entity 仍最多展开一次。保留原逆序入栈和合法 forest 的 preorder；没有新增 visited set、全域扫描或图修复。涉及 `collect_subtree_records` 的 indexed/fallback、`subtree_entity_ids` 的 indexed/fallback，以及 `subtree_component_count` 的 indexed/fallback。缺少或重复 parent edge 的底层模型若改变，需重新证明此边界。

本片是 Q Runtime1010 的显式后继：`derived_state.rs` 编辑前 SHA `4c5fcea969cd4e562eb219af437e0cf59517268af4a572d0a7baac12559bfd55`，derived-state 测试绑定 SHA `cbb0316971f8571272ef106618e7d7d041c2ebe4a1870f6a790e6f566903be21`，结构门槛 SHA `42f978aec2a2ceeced4af2ec89e50455ee87cd199f8f0259d761390e484b4b57`。公共合同是 R Runtime1012 的显式后继，编辑前 SHA `674bed9da1c88c50e850fdd87dfb400ddb0f590d3c1fb11b90cb5f370f8e2985`。不修改 Runtime1012 的生产 owner，也不改 Q 的其它冻结输入。

## 回归与性能协议

- 普通测试从真实 `World` 入口验证 dirty/current index 上的 self/2/3 环与枝叶的 preorder 和组件计数；尾节点起步仍只遍历尾节点及其子节点；合法 forest 的次序、计数、prepare 范围和当前索引零重建保持原义。
- 实际 pending `Mut<ActiveSelf>` 通知在 `prepare_entity_subtrees` 中完成有限遍历，随后返回 typed parent-chain error；首轮 flush 保持已暴露的 effective `world_generation`，再次拒绝不产生 rows 或 lifecycle commit。既有 Runtime1012 parent-chain、row/tick、observer 及 Runtime08 batch 回归继续适用。
- ignored Release profile 仅在合法 forest 上，将完整 Q 旧 `subtree_records` 与 `subtree_component_count` public body 对照当前真实 API；4096 深链、100k unrelated、2/128 枝，5 warmup pairs 和 31 alternating sample pairs。构造、核对和 drop 在计时外，输出 raw 纳秒样本与 p50/p95/p99。此 profile 不测分配，也不替代 Runtime1012 完整 prepare profile。
- 结构门槛保留原七个 derived-state child 的 48 个测试声明断言，并单独核对新增五个普通测试、一个 ignored Release 测试、绑定及文件预算。

## 当前证据与开放项

先保存失败场景回归源码及 tests-first 快照，再写六处保护。受本轮约束，未运行 dynamic red、Cargo、Release profile 或 managed 验证；原始延迟和分配数据仍 pending，不能据此宣称性能通过。精确 preimage、lease/maintenance 授权、inverse 和最终源 manifest 使用 `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-r-runtime62-subtree-walk-cycle-*` 作为 R 批次证据前缀。

RSH-G03 要求所有损坏图诊断 walker 有界；本片只覆盖上述六处 subtree DFS，不关闭其它 parent/transform walker、restore 合并图成环或 protected-component authority 的开放门槛。
