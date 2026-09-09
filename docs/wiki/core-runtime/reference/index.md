---
related_code:
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/mod.rs
  - zircon_runtime/src/operation/mod.rs
  - zircon_runtime/src/plugin/bridge/table.rs
implementation_files:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
plan_sources:
  - user: 2026-09-09 完善所有公开接口、机制案例、最佳实践、教程与功能介绍
tests:
  - zircon_runtime/src/core/runtime/tests
  - zircon_runtime/src/operation/tests
  - zircon_runtime/src/plugin/bridge/table.rs
doc_type: category-index
---

# CoreRuntime 公开接口参考

本目录是 `zircon_runtime::core` 的源码级 API 参考。它和上层“快速版”页面的区别是：这里按公开类型和函数组织内容，明确参数、返回值、错误、生命周期、线程边界和 feature gate。文中出现“示意调用形状”时，表示调用顺序真实、但示例省略了项目特有的构造器或业务类型；不得把它当成可直接复制的完整程序。

## 页面地图

| 页面 | 覆盖范围 | 首要源码 |
| --- | --- | --- |
| [Runtime 构造、激活与关闭](runtime-construction.md) | `CoreRuntime` facade、时钟/随机种子、模块启动和 shutdown | `core/runtime/runtime.rs` |
| [CoreHandle、CoreWeak 与服务准入](handles-and-admission.md) | 强弱句柄、`ServiceHandle::enter`、in-flight drain | `core/runtime/handle/*` |
| [ModuleDescriptor 与生命周期排序](module-descriptor-lifecycle.md) | 模块描述符、依赖图、`ModuleLifecycle` 回调 | `core/runtime/descriptors/*` |
| [事件、配置与时间框架](framework-events-config-time.md) | event bus、config store、`FrameClock`、typed state | `core/framework/events.rs`, `core/framework/time/*` |
| [ManagerResolver 与类型化句柄](manager-resolver.md) | manager 名称、`ManagerServiceHandle`、feature gate | `core/manager/*` |
| [任务图、JobScheduler 与取消](task-graph-scheduler.md) | runtime-owned workers、依赖任务、诊断和关闭 | `core/runtime/tasks/*` |
| [RuntimeOperation 服务](runtime-operation-service.md) | bounded submit、snapshot/prepare/apply、harvest | `operation/*` |
| [诊断日志 API](diagnostic-log-api.md) | filter、sink、flush、panic hook 和背压 | `diagnostic_log/*` |
| [平台能力与插件桥边界](platform-plugin-boundaries.md) | capability status、FrozenBridgeTable、generation | `platform/*`, `plugin/bridge/*` |

## 统一阅读约定

### 稳定性标记

- **公开稳定入口**：`pub` 并从 crate 根或公开模块 re-export；可供 app、editor、plugin 调用。
- **公开但 feature-gated**：只有启用相应 Cargo feature 才会编译，例如 `ai-contracts`、`physics-contracts`、`sound-contracts`。
- **crate-private**：`pub(crate)`、`pub(super)` 或未 re-export 的类型；页面只描述其对公开行为的影响，不建议外部依赖。
- **测试专用**：`#[cfg(test)]` 下的 barrier、fixture、source guard 和优化测试；这些不是运行时契约。

### 生命周期方向

```mermaid
flowchart LR
    A[CoreRuntime::try_new] --> B[register_module]
    B --> C[freeze module graph]
    C --> D[activate module]
    D --> E[resolve service]
    E --> F[ServiceHandle::enter]
    F --> G[deactivate with drain]
    G --> H[shutdown task graph]
```

实线表示调用方可直接触发的公开入口；图中的 graph freeze、claim 和 drain 是内部阶段，不能通过手动修改 registry 绕过。

### 错误处理原则

`CoreError`、`RuntimeOperationServiceError`、`TaskGraphAdmissionError` 等错误都携带失败阶段。调用方应按阶段处理：注册/排序失败通常是配置错误，解析失败通常是依赖或类型错误，drain/timeout 则应记录诊断并决定是否终止宿主。不要用 `unwrap` 把可恢复的 `ServiceUnavailable`、`StaleServiceHandle` 或容量错误升级为进程崩溃。

## 参考引擎对照

ZirconEngine 的 runtime-owned task graph 和显式模块描述符更接近 Fyrox 的多 crate、editor/runtime 分层；`dev/Fyrox/fyrox-core` 与 `dev/Fyrox/editor` 的目录形态说明基础设施和编辑器可以独立演进。时间、事件和任务 API 也吸收了 Bevy 的数据导向风格，参考 `dev/bevy/crates/bevy_time`、`bevy_tasks` 与 `bevy_ecs`. 有意的差异是：Zircon 不把系统调度隐藏在全局 world 中，而是要求服务注册、线程准入和关闭都经过 runtime owner，以便插件卸载和多 runtime 并存。

## 版本与验证

本目录的事实以当前源码为准。修改公开签名时，必须同时更新对应页面的 API 表、错误表和示例，并运行 Wiki 严格 metadata 校验；Cargo 构建属于仓库维护任务，不在本参考页中虚构“已通过”。
