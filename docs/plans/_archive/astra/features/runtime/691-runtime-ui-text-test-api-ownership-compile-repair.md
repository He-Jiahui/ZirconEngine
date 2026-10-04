---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/82/2026-08-27-focused-bound-text-policy.md
  - docs/plans/optimize/zircon_runtime/82/2026-08-29-number-field-focused-model-refresh-architecture.md
related_code:
  - zircon_runtime/src/ui/text/measure_cache/generation_key_tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/bound_text_model_updates/tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/number_model_updates/tests.rs
  - zircon_runtime/src/ui/surface/render/cache/tests/update.rs
---

# Runtime UI/Text 测试 API 与所有权编译修复

本批次合并历史受管 Windows UI-feature 日志中的四类测试漂移：

- retained plain-document cache 已返回 `Result`，五处 cache identity/source-alias 断言仍把它
  当作直接 parsed document；
- 三处 `update_text_model(&mut surface, ...)` 在同一调用内读取 `surface.tree.tree_id`，
  触发可变/不可变借用重叠；
- text/number input test fixture 仍使用 `toml::map::Map`，而 template metadata 已收敛到
  `BTreeMap`；
- render-cache test 的局部 `extract` 值遮蔽了后续构造器函数。

有效 plain-text fixture 现在在测试边界显式 `expect` retained document；其余调用先快照 tree
ID、使用当前 metadata map 类型，并以 `current_extract` 保留函数可见性。缓存命中、source
alias 拒绝、text/number model、render-cache allocation 与排序语义均未改变。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime11A / Runtime11B / Runtime82 | retained text cache、bound/number model 与 render-cache 测试 API/所有权编译修复 | `implemented_pending_validation` | 历史受管日志定位 retained-document 八项、bound-text 四项、number-model 两项和 render-cache 一项直接诊断；当前结构守卫 `5/5`、无直接 Result 使用守卫通过、Runtime performance-contract 合批 `1146/1146`（`4.333s`）、Runtime Text contract 合批 `143/143`（`0.933s`）、scoped diff-check 通过。受管 Windows Cargo、原始 Rust 回归和 Release p50/p95/p99 仍待异步批量验证。 |

整文件 Rustfmt 对上述测试路径仍报告修改前既有、非本次 diff hunk 的格式差异；本批次未把
编译修复扩大为格式化重写。静态结果不等同于产品性能验收。
