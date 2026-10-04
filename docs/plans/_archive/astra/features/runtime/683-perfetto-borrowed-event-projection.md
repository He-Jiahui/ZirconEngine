---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-08-26-perfetto-borrowed-event-projection.md
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
---

# Runtime03 Perfetto 借用事件投影

Perfetto 导出投影现在借用 `ProfileSnapshot` 中的事件文本与路径，并使用强类型参数变体
代替逐事件 `serde_json::Value` 构造；事件向量按 frame/span/counter 总数一次性预留容量。
JSON 字段形状、事件顺序和文件导出边界保持不变。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime03 | 借用事件文本、类型化 args 与精确事件容量 | `implemented_pending_validation` | `optimization_batch_20260826f_runtime03_perfetto_projection_preserves_json_shape`、借用/无动态 JSON 源码守卫已落源；Runtime/Editor 静态合同批次 `1727/1727`（`4.095s`）、Rustfmt 与 scoped diff-check 通过。32,768 span 模型移除逐事件文本/args 拷贝并从空向量增长改为总量预留；受管 Windows Release 配对 p50/p95/p99 仍待异步验证。 |

本记录不宣称 Perfetto 线程身份、快照深拷贝、原子 artifact 发布或产品导出吞吐已完成。
