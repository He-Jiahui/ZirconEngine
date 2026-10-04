---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/64/2026-08-26-ui-resource-cached-diagnostic-index.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
related_code:
  - zircon_runtime/src/ui/tests/event_routing/component_events/missing_policy.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/render_domains.rs
  - zircon_runtime/src/ui/tests/icon_atlas.rs
---

# Runtime UI 测试 DTO 与断言编译修复

历史受管 Windows UI-feature 编译日志定位到三项低风险测试漂移：

- `UiTemplateActionInvocation` 已隐藏 route storage，回归仍读取旧 `action.route` 字段；
- render-domain composition clauses 仍把空数组作为旧 slice 返回；
- icon-atlas fixture 的 tuple closure 在当前 Rust 推断下无法确定 `icon_id` 类型。

测试现在使用公开的 `target_id()` accessor、具名 `Vec<toml::Value>` 空值和显式 `(&str,
&str)` tuple 参数。组件事件 route、dirty-domain 状态、icon slot 排序/UV 与生产路径均未改变。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime11A / Runtime64 / Runtime78 | action DTO、render-domain clause 与 icon fixture 测试编译修复 | `implemented_pending_validation` | 历史受管日志定位三类直接诊断；当前源码守卫 `3/3`、Runtime performance-contract `1146/1146`（`5.074s`）、Editor performance-contract `581/581`（`0.954s`）、Runtime Text contracts `143/143`（`0.971s`），scoped diff-check 通过。受管 Windows Cargo、原始 Rust 回归和 Release p50/p95/p99 仍待异步批量验证。 |

整文件 Rustfmt 对 render-domain 测试报告的既有未触及格式差异未被扩大修写；静态结果不替代 Cargo 或产品性能门槛。
