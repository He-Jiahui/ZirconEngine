---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/64/2026-08-26-ui-resource-invalidation-hash-membership.md
  - docs/plans/optimize/zircon_runtime/81/2026-08-27-nowrap-clip-width-semantics-review.md
related_code:
  - zircon_runtime/src/ui/surface/input/rich_link.rs
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/measure.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolver/hash_invalidation_tests.rs
---

# Runtime UI 测试导入与 API 编译修复

历史受管 Windows UI-feature 编译日志定位到四个彼此独立的测试模块漂移：

- rich-link 测试两处使用 `UiPointerRoutingPath`，但测试子模块没有导入它；
- scroll-virtualization 测试仍从 `ui::tree` 导入已迁移到 `ui::event_ui` 的
  `UiStateFlags`；
- intrinsic-measurement 测试仍引用已移除的无 session wrapper，而当前实现为
  `measurement::intrinsic_measurement_frame_with_provider`；
- Runtime64 resource-invalidation hash 回归构造 `UiResourceFallbackPolicy`，但缺少
  interface template 导入。

本批次将测试引用收敛到当前所有者路径。measurement 回归创建默认
`SharedTextLayoutSession` 并显式处理 geometry-admission 结果，因此继续覆盖 rich
和 vertical intrinsic frame 的同一语义；不改变任一生产热路径、hash membership 算法、
排序、资源失效结果或 UI 输入行为。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime11A / Runtime64 / Runtime81 | 四个 Runtime UI 测试导入/API 漂移修复 | `implemented_pending_validation` | 历史受管日志定位五处直接诊断（rich-link 两处）；当前源码守卫 `6/6`、Runtime UI performance-contract 批次 `213/213`（`0.409s`）、Runtime Text contract 批次 `143/143`（`0.828s`）、相关文件 non-recursive Rustfmt 与 scoped diff-check 通过。当前源码的受管 Windows Cargo、原始 Rust 回归和 Release p50/p95/p99 仍待异步批量验证。 |

本记录只关闭所列测试路径/API 的静态漂移，不声称历史受管日志中的全部编译诊断已修复，
也不以静态契约结果替代 Runtime64 的 Release P95 或 Runtime81 的产品性能门槛。
