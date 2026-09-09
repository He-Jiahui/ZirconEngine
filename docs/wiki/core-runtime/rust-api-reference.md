---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/operation/mod.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
implementation_files:
  - zircon_runtime/src/core
  - zircon_runtime/src/operation
  - zircon_runtime/src/diagnostic_log
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
tests:
  - zircon_runtime/src/core/runtime/tests
  - zircon_runtime/src/core/manager/tests.rs
doc_type: module-detail
---

# Core Rust API 参考

本页列出面向调用者的核心入口。完整类型字段、trait 约束和 feature gate 以源码 rustdoc 为准。

## Runtime 与生命周期

| API | 用途 | 关键错误/限制 |
| --- | --- | --- |
| `CoreRuntime::try_new` | 创建带默认任务图的 runtime | 返回 `EngineTaskGraphInitError` |
| `try_with_task_graph_options` | 指定 worker 预算 | 多 runtime 宿主应优先使用 |
| `handle` / `weak` | 获取强/弱 core 访问入口 | registry-owned 实例应保存 weak |
| `register_module` | 注册 `ModuleDescriptor` | 名称/依赖/descriptor 验证 |
| `activate_module` / `activate_registered_modules` | 执行 lifecycle | 可能 ready timeout 或 factory error |
| `deactivate_module` / `shutdown_registered_modules_with_drain_timeout` | 反向停机 | service drain/dependency 可阻止卸载 |

## 服务与事件

| API | 用途 |
| --- | --- |
| `resolve_driver` / `resolve_manager` | 立即取得具体服务 `Arc<T>` |
| `resolve_*_handle` | 取得带 generation 的 `ServiceHandle<T>` |
| `ManagerResolver` | 取得领域化 manager handle 并由 weak core 解析 |
| `publish_event` / `subscribe_events` | topic + JSON payload 的事件流 |
| `event_bus_diagnostics` | 读取背压、投递与排队指标 |
| `store_config_value` / `load_config` | 进程内 JSON config store |

## 时间、状态、任务

| API | 用途 |
| --- | --- |
| `tick_time` / `advance_time_by` | 生成 `FrameTimeSnapshot` |
| `submit_clock_discontinuity` | 时钟跳变/rebase receipt |
| `apply_time_policy` | 原子应用 `TimePolicyTransaction` |
| `init_state` / `set_next_state` / `apply_state_transition` | typed state 转换 |
| `create_task_graph_scope` | 建立受限异步任务域 |
| `task_graph_worker_inventory` | 查询 runtime-owned worker 预算 |
| `shutdown_task_graph` | 停止 scoped admission 并返回 shutdown report |

## Operation 与诊断日志

| API | 用途 | 关键限制 |
| --- | --- | --- |
| `RuntimeOperationService::register_handler` | 注册 `operation_id` 对应的三阶段 handler | 同一服务不能重复注册；ID 不能为空 |
| `RuntimeOperationService::submit` / `submit_json` | 有界提交 JSON 操作并取得 handle | 先做 ABI、task 和 retained-byte admission |
| `RuntimeOperationService::tick` | owner 阶段推进 snapshot、prepare completion 和 apply | 不会自动在后台修改 World |
| `RuntimeOperationService::poll` / `harvest` / `cancel` | 查询阶段、取走终态结果或取消未 apply 操作 | 受 phase、deadline、TTL 和 session owner 约束 |
| `initialize_process_log_with_settings` | 初始化进程级控制台/文件 sink | 需要 `diagnostic-log` feature；返回路径可能为 `None` |
| `write_*` / `write_*_lazy` | 按等级写诊断消息 | 过滤前不执行 lazy 闭包，队列有界 |
| `flush_process_log` / `shutdown_process_log` | 有界 flush、同步文件并关闭 sink | 返回 `false` 必须作为输出故障处理 |

## 共享基础

| 模块 | 主要类型 |
| --- | --- |
| `core::framework` | 各领域 trait、descriptor、DTO、events、time、render、scene、UI 合同 |
| `core::manager` | `ManagerServiceHandle`、`ManagerResolver`、规范 manager 名称 |
| `core::math` | `Transform`、`Vec*`、`Mat4`、`CoordinateSchema`、validated/narrowing API |
| `core::resource` | `AssetReference`、`ResourceLocator`、`ResourceHandle<T>`、`ResourceRegistry`、readiness/event 类型 |

## 导入建议

```rust
use zircon_runtime::core::{
    CoreRuntime, EngineTaskGraphOptions, InitLevel, ModuleDescriptor,
    TaskGraphScopeDescriptor,
};
```

需要领域 trait 时从 `core::framework::<domain>` 取；需要稳定 manager 时从 `core::manager` 取。避免深层导入 `core::runtime::state::*` 或私有 slot/lock 模块，因为这些是可重构实现细节。
