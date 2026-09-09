---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
implementation_files:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/core/manager/resolver.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/engine-architecture/core-runtime-service-registry.md
tests:
  - zircon_runtime/src/core/runtime/tests
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/manager/tests.rs
doc_type: category-index
---

# 核心运行时

`zircon_runtime::core` 是 ZirconEngine 的生命周期和跨模块基础。它不实现每个领域功能，而是提供模块注册、服务解析、生命周期协调、事件、配置、时钟、任务、随机数、状态机和诊断，使 scene、asset、graphics、UI、script、editor 等模块能以同一种方式加入产品。

## 阅读顺序

| 页面 | 回答的问题 |
| --- | --- |
| [运行时与模块生命周期](runtime-and-module-lifecycle.md) | 如何构造 CoreRuntime、排序模块、ready、停机和处理并发？ |
| [服务注册表与依赖](service-registry-and-dependencies.md) | Driver、Manager、Plugin 如何注册、冻结和解析？ |
| [模块、服务与生命周期（快速版）](module-lifecycle.md) | 如何用最小描述符接入模块？ |
| [事件、配置与时间](events-config-and-time.md) | 如何发布事件、保存配置、推进帧时钟和处理 discontinuity？ |
| [服务解析与事件（快速版）](services-events.md) | 如何取得 manager、发布/订阅事件并访问配置？ |
| [时间、状态和随机数](time-state-random.md) | 如何推进时间、管理 typed state、维持可重现随机流？ |
| [任务、状态、随机数与诊断](tasks-state-random-and-diagnostics.md) | 如何管理任务图、状态机、随机流和 profiling？ |
| [任务与诊断（快速版）](tasks-and-diagnostics.md) | 如何创建受限任务域、取消/关停工作并采集观测数据？ |
| [Runtime Operation](operations.md) | 如何注册有界异步操作、分离 worker prepare 与 owner apply，并通过 ABI harvest 结果？ |
| [诊断日志](diagnostic-log.md) | 如何配置级别/前缀过滤、文件 sink、背压、panic flush 和有序关闭？ |
| [Manager 句柄与解析](manager-handles-and-resolution.md) | 如何使用稳定句柄访问跨模块服务？ |
| [Rust API 使用指南](rust-api-guide.md) | 如何从 prelude、EngineModule 和 CoreHandle 编写 Rust？ |
| [Rust API 参考](rust-api-reference.md) | 哪些类型和函数是调用方的稳定入口？ |

## 组成

```text
CoreRuntime / CoreHandle
  +-- module graph + ModuleLifecycle
  +-- driver/manager/plugin service registries
  +-- EventBus + config store
  +-- FrameClock + time policies + typed State
  +-- EngineTaskGraph + bounded I/O + cancellation
  +-- RuntimeOperationService + bounded prepare/apply
  +-- RandomService + DiagnosticsStore
  +-- diagnostic_log sink + process diagnostics
```

`CoreRuntime` 持有一个 `CoreHandle`；`CoreHandle` 是内部模块和宿主可克隆的强访问入口，`CoreWeak` 是 registry-owned 实现反向访问 core 时使用的弱入口。这样可以避免服务实现与 runtime 内核形成强引用环。

## 快速示例

```rust
use zircon_runtime::core::CoreRuntime;

fn start() -> Result<CoreRuntime, Box<dyn std::error::Error>> {
    let runtime = CoreRuntime::try_new()?;
    // 先由 app/builtin/plugin composition 注册 descriptor，再统一激活。
    runtime.activate_registered_modules()?;
    Ok(runtime)
}
```

真正产品通常由 `zircon_app` 收集 builtin 和插件 descriptor。库作者不应绕过组合层，通过临时容器自行创建 scene、graphics 或 manager 实例。

## 当前边界

- **已实现**：模块注册和有向激活、服务句柄、事件总线、config store、帧时间、任务图、Runtime Operation、诊断日志和状态机。
- **受限**：每个领域模块是否编译/激活还取决于 Cargo feature、`RuntimeTargetMode`、profile 和插件清单。
- **内部实现**：`CoreRuntimeInner`、服务槽细节、锁和 lifecycle coordinator 不属于跨 crate API。
- **不保证**：调用 `shutdown_task_graph` 不等于完成所有产品级关停；完整顺序仍由应用 shutdown policy 负责。
