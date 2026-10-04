---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
---

# Runtime UI Control Index 哈希目录

`UiSurfaceControlIndex` 已有按引用的增量候选桶，因此稳定 Scrollbar 拖拽不会再次遍历
整棵 `UiTree`。本切片继续收紧该索引本身的热路径：五个只进行按键查询、插入和删除的
目录从 `BTreeMap` 改为 `HashMap`，并在冷建时按节点数预留容量。候选 `UiNodeId` 集合
仍是 `BTreeSet`，所以多重引用和哈希碰撞时的最低 ID 优先顺序不变。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime74 / Runtime UI | control/reference/compiled-slot 目录哈希查找，冷建容量预留，候选顺序保持 | `implemented_pending_validation` | 新增 `control_index_directories_use_hash_lookup_with_stable_candidate_order` 源码回归；本地静态守卫 `8/8` 通过；Runtime UI performance-contract 批次 `213/213`（`0.387s`）通过，Rustfmt 与 scoped diff-check 通过。 |

已有的索引设计仍负责把稳定引用查找从整树扫描收敛到候选桶；本切片只将目录层的
`O(log C)` / `O(log N)` 有序键访问改为期望 `O(1)`，并不把这个结构变化伪称为真实
CPU、分配、RSS 或 input-to-present p50/p95/p99 测量。受管 Windows Cargo 与产品拖拽
profile 仍待异步批量验证。
