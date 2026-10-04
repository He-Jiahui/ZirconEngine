---
status: implemented_pending_validation
plan_sources:
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/astra/performance/04-hit-grid-cell-batching.md
related_code:
  - zircon_runtime/src/ui/tree/hit_test/cell_membership_patch.rs
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
---

# Runtime09 Hit-grid 批量 patch 编译修复

受管 Windows UI-feature 编译日志曾在
`UiCellMembershipPatches::rejects_missing_cell_before_publishing_valid_replacements` 报出
`E0282`：测试中的 `vec![UiHitTestCell { ... }].into()` 没有可推断的目标容器类型。
该测试用于保护“预检失败时不得发布任一有效 cell replacement”的原子性合同。

测试构造现在显式声明为 `UiPersistentSequence<UiHitTestCell>`。这只固定 Rust 的类型推断，
不改变批量按 cell stage/apply、持久 COW、预检失败原子性、entry 排序或任何 hit 结果。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime09 / Hit-grid | 批量 membership patch 原子性回归的 `E0282` 类型修复 | `implemented_pending_validation` | 历史受管日志定位 `cell_membership_patch.rs:398`；当前源码守卫 `2/2`、Runtime UI performance-contract 批次 `213/213`（`0.387s`）、相关文件 non-recursive Rustfmt 与 scoped diff-check 通过。受管 Windows Cargo、原始 hit-grid 回归和 Release p50/p95/p99 仍待异步批量验证。 |

本记录不宣称历史 139 项编译诊断均已关闭，也不以局部静态通过替代当前源码的包级 Cargo 结果。
