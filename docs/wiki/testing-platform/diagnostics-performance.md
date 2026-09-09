---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/core/runtime/tasks
  - zircon_app/src/entry/runtime_entry_app/runtime_product_diagnostics.rs
  - tools/cargo-zircon/src/build/product_build/capture.rs
  - tools/cargo-zircon/src/build/receipt/file_digest.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/core/runtime/tasks
  - zircon_app/src/entry/runtime_entry_app
  - tools/cargo-zircon/src/build/product_build
plan_sources:
  - docs/plans/milestone-validation-policy.md
  - docs/plans/performance
tests:
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/core/runtime/tasks
  - tools/cargo-zircon/src/build
doc_type: diagnostics-reference
---

# 诊断与性能

## Runtime 诊断

`DiagnosticsStore` 保存结构化事件、计数器和批次游标；`CoreRuntime` 提供读取、清空和订阅入口。应用层的 product diagnostics 会把 profile、module composition、frame、surface 和 shutdown 结果附加到同一条证据链。诊断 payload 应包含稳定 code、generation、时间戳和可操作 hint，避免只记录自然语言。

## 任务与指标

`EngineTaskGraph`、`JobScheduler` 和 bounded IO API 公开排队、运行、取消、失败和 drain 指标。性能页关注三个约束：任务域不能越过 owner scope、队列/字节预算必须有上限、shutdown 必须在 deadline 内完成。`profiling` Cargo profile 保留符号；`diagnostic` storage mode 保留更完整的 Cargo/rustc 默认信息。

## 构建证据

product build capture 记录 stdout/stderr、退出状态和日志片段；receipt 的 file digest 使用 canonical bytes，不能以文件名或 mtime 代替。性能比较必须固定源码 digest、toolchain、profile、feature 和 target pool，否则数据不可比。

## 基准写法

- 先运行 warm pool，再采集 cold/warm 两类数据。
- 报告 p50/p95、失败率、峰值内存和 artifact 大小；不要只报平均耗时。
- 将 `performance_tests.rs` 与行为测试分开，性能回归不改变产品退出条件。
- 发现资源耗尽时先检查 coordinator lane、磁盘 reserve 和 scratch 清理，再判断为编译器或 Runtime 性能问题。
