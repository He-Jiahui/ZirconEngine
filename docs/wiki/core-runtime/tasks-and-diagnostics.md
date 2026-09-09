---
related_code:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io
  - zircon_runtime/src/core/runtime/tasks/bounded_stream_io
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/diagnostics
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
tests:
  - zircon_runtime/tests/runtime_task_domain_isolation_performance.rs
  - zircon_runtime/src/tests/runtime_diagnostics
doc_type: module-detail
---

# 任务、I/O 预算与诊断

## EngineTaskGraph

`CoreRuntime` 创建时拥有一个 `EngineTaskGraph`，而不是依赖进程全局线程池。多 runtime 宿主可通过 `EngineTaskGraphOptions::with_worker_threads` 显式控制工作线程预算：

```rust
use zircon_runtime::core::{CoreRuntime, EngineTaskGraphOptions, TaskGraphScopeDescriptor};

let runtime = CoreRuntime::try_with_task_graph_options(
    EngineTaskGraphOptions::with_worker_threads(4),
)?;
let scope = runtime.create_task_graph_scope(
    TaskGraphScopeDescriptor::new("asset-import"),
)?;
# let _ = scope;
# Ok::<(), Box<dyn std::error::Error>>(())
```

scope 把任务 admission、取消、容量和终端 census 绑定到明确 owner。长任务（导入、烘焙、插件扫描、网络/文件流）不应绕过 scope 直接创建无主线程。

## 调度与取消

`TaskDescriptor` 由 `TaskId`、`TaskPoolKind`、label 和 `TaskCancellationPolicy` 描述。`JobScheduler`/`TaskPool` 提供任务运行、状态和报告；`TaskHandle` 与 `JobHandle` 用于观察终端结果。取消不是强杀线程：`TaskCancellationToken` 是协作式信号，任务实现必须在合理边界检查并释放资源。

`parallel_for`、`parallel_map_indices`、`parallel_map_ordered` 适合纯 CPU 批处理；它们不适合持有 World 写锁、GPU encoder 或 UI 线程对象。需要线程命名时使用 `spawn_named_thread`，其错误会转换为 `CoreError::ThreadSpawn`。

## 有界 I/O

两套机制避免 I/O 任务抢占内存/线程：

- `BoundedKeyedIoLane`：按 key 公平 admission、ticket、fence、deadline、取消权限和终端报告，适合资源导入、下载或按项目/资产隔离的工作。
- `BoundedStreamIoLane`：限制并发 reader、单行字节、队列条目和队列总字节，适合子进程 stdout/stderr、日志和流式解析。
- `RetainedByteBudget`：用 lease 限制在内存中保留的 payload/缓存字节；失败要报告 budget，而不是 silently truncate。

调用者应把 ticket/lease 放在 operation 或 session 生命周期中。若项目切换、插件卸载或 host 取消，先关闭 admission，再等待 fence/census，而不是让旧 worker 将结果写入新 generation。

## 任务图关停

```rust
use std::time::Duration;

let report = runtime.shutdown_task_graph(Duration::from_secs(2))?;
# let _ = report;
# Ok::<(), Box<dyn std::error::Error>>(())
```

该调用关闭新的 scoped task admission，并在 deadline 内等待 scope census 与 worker join。返回 `TaskGraphShutdownReport`/`TaskGraphShutdownError`，其中包含未停任务、取消和 worker 状态。它不是完整产品 shutdown；先停止提交新 frame/operation、再执行模块反向 cleanup，最后关闭 task graph。

## DiagnosticsStore

`DiagnosticStore` 是读写分离的时序观测库。模块通过 `CoreRuntime::record_diagnostic` 写入路径、帧号、值、单位和 subsystem tags，编辑器/工具通过 `diagnostic_store_snapshot` 获取只读快照。

```rust
runtime.record_diagnostic(
    "runtime.frame.cpu_ms",
    128,
    4.2,
    Some("ms"),
    ["runtime", "frame"],
);
let snapshot = runtime.diagnostic_store_snapshot();
# let _ = snapshot;
```

`runtime::diagnostics` 还聚合 frame、render、animation、physics、devtools 和 profiling 快照。`ProfileRecorder`、`ProfileCaptureConfig`、`start_capture`、`stop_capture` 适用于受限采集；不要在每帧无界扩充 trace 或把 debug logging 作为性能统计替代品。

## 任务诊断

任务域公开排队、运行、完成、取消、panic、依赖等待和 queue/execution wait 的指标常量，以及 `TaskDiagnosticCursor`/`TaskDiagnosticBatch` 增量读取模型。消费方应使用 cursor 而不是每帧复制完整历史；retention capacity 有上限，慢消费者必须处理 gap/截断语义。

## 错误与限制

- scope 容量、worker 数和 retained byte budget 必须在创建时有效；admission 被拒绝应返回 `TaskGraphAdmissionError` 或对应 lane error。
- 不能假设取消一定立即终止；外部 I/O、GPU/OS API 和用户任务必须配合安全点。
- task panic 会记为终端状态/诊断，但它不是应用继续运行的许可；owner 应决定是否升级为产品失败。
- 性能数据应与具体 profile、帧/工作负载和采集预算一起解释。
