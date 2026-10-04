---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/64/2026-08-26-ui-resource-cached-diagnostic-index.md
  - docs/plans/optimize/zircon_runtime/267/2026-08-28-owned-ui-binding-arguments.md
  - docs/plans/optimize/zircon_runtime/269/2026-08-28-owned-analog-route-policy-event.md
  - docs/plans/optimize/zircon_runtime/358/2026-08-30-style-sheet-capacity.md
related_code:
  - zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver/capacity_tests.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolution_report/cached_diagnostic_index_tests.rs
  - zircon_runtime/src/ui/event_ui/manager/invocation/owned_binding_arguments_tests.rs
  - zircon_runtime/src/ui/surface/input/analog/owned_event_route_policy_tests.rs
---

# Runtime 性能基准回归编译修复

历史受管 Windows UI-feature 编译日志显示四个已有性能基准/回归因测试代码漂移而无法编译：

- style-resolver capacity benchmark 的 `sum()` 对 `usize`、`LengthHint` 和 `BigInt` 失去
  类型推断；
- cached-diagnostic regression 将局部 `reference` 值遮蔽了后续 fixture 工厂函数；
- owned binding arguments 和 owned analog route benchmarks 的闭包会可变借用 benchmark
  输入，但闭包绑定不是 `mut`。

本批次分别固定 `usize` 聚合、采用无阴影的 `resource_reference` 名称、并把两个
`FnMut` benchmark closure 声明为 `mut`。优化算法、样本数、alternating sample 顺序、
P95 门槛、被测生产路径和任何发布态测量均未改变。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime64 / Runtime267 / Runtime269 / Runtime358 | 四个 Runtime 性能基准/回归编译漂移修复 | `implemented_pending_validation` | 历史受管日志定位四类直接诊断；当前源码守卫 `4/4`、Runtime performance-contract 合批 `1146/1146`（`3.510s`）、Runtime Text contract 合批 `143/143`（`0.766s`）、相关文件 non-recursive Rustfmt 和 scoped diff-check 通过。所有 ignored Release benchmark、Windows Cargo 和 p50/p95/p99 产品证据仍待异步批量验证。 |

该记录恢复基准的可编译性，不将静态契约通过等同于 Runtime64/267/269/358 的实际 Release
性能达标。
