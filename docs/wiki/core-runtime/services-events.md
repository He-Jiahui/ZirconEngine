---
related_code:
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/runtime/config_store.rs
implementation_files:
  - zircon_runtime/src/core/runtime/handle
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/manager/resolver.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/engine-architecture/core-runtime-service-registry.md
tests:
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_runtime/src/core/manager/tests.rs
doc_type: module-detail
---

# 服务解析、事件和配置

## 服务访问模型

Core 将服务分为 `Driver`、`Manager`、`Plugin`。Driver 是生命周期更低的底层提供者；Manager 是可被上层稳定解析的业务入口；Plugin 是可装卸扩展。`RegistryName` 使用规范化片段，避免调用方各自重复字符串。

调用方有两类入口：

```rust
use zircon_runtime::core::CoreRuntime;

// 当调用者知道实现/trait 类型和规范服务名时。
let manager = runtime.resolve_manager::<MyManager>("Example.Manager")?;

// 当需要跨异步边界保存身份而不是缓存 Arc 时。
let handle = runtime.resolve_manager_handle::<MyManager>("Example.Manager")?;
let manager = handle.enter()?;
# let _ = manager;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ServiceHandle::enter` 返回带 in-flight 计数的 `ServiceCallGuard`；guard 离开作用域后才允许对应服务继续排空。`ManagerServiceHandle` 则由 `ManagerResolver` 使用，二者都不能当成永久有效的裸 `Arc`。

以上 `MyManager` 是调用方定义的服务类型占位符；真实模块应使用 owner 暴露的 trait、`*_MANAGER_NAME` 常量及 `ManagerResolver`。例如 resolver 提供 `rendering_manager_handle`、`resource_manager_handle`、`input_manager_handle`、`config_manager_handle`、`level_manager_handle`、`animation_manager_handle`、`navigation_manager_handle` 等领域化入口。

## ManagerServiceHandle

`ManagerServiceHandle<T>` 保存服务槽位、generation、注册名、runtime 身份和类型标记。它的意义是“这次解析得到的服务版本”，而不是永远有效的裸指针。模块重启、服务重建或 runtime 关闭会使 generation 失效；下一次使用必须重新解析并处理 `CoreError`。

跨线程长期工作应保存 handle 或 `CoreWeak`，不要把 `Arc<dyn Manager>` 长期挂在 registry-owned 对象中。这样既避免引用环，也能阻止卸载后的陈旧调用。

## EventBus

事件总线承载低耦合通知。每个 `EngineEvent` 包含 `topic: String` 与 JSON `payload`。订阅策略在背压语义上明确：

| `EngineEventDeliveryPolicy` | 适用场景 | 行为 |
| --- | --- | --- |
| `Lossless` | 低频、不可丢失的控制事件 | 生产者/订阅路径保留每个事件，须审查积压风险 |
| `BoundedDropOldest` | 高频状态或 UI 通知 | 有界队列满时丢弃最旧项 |
| `Latest` | 只关心最新快照的状态同步 | 旧值被新值替换 |

```rust
use serde_json::json;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::core::framework::events::EngineEventDeliveryPolicy;

let subscription = runtime.subscribe_events(
    "scene.changed",
    EngineEventDeliveryPolicy::Latest,
);
runtime.publish_event("scene.changed", json!({"generation": 42}));
let event = subscription.try_recv()?;
# let _ = event;
# Ok::<(), Box<dyn std::error::Error>>(())
```

订阅实现返回 `EngineEventSubscription`，支持 `recv`、`try_recv` 和 `recv_timeout`。应用主循环不应在 UI/渲染线程无限期阻塞 `recv`；使用 poll、bounded timeout 或 wake integration。

## 事件诊断和背压

`event_bus_diagnostics()` 返回每 topic 的发布、投递、丢弃、排队峰值、等待者和抽样延迟指标。选择 `Lossless` 前应先明确消费者速率和关停策略；高频 frame/viewport/state 流一般应使用 `Latest` 或有界 drop policy，而非无上限排队。

## Config store

core 内置的是进程内 config store，不替代项目 manifest、editor settings 或资源数据库：

```rust
use serde_json::json;

runtime.store_config_value("diagnostics.enabled", json!(true));
let enabled: bool = runtime.load_config("diagnostics.enabled")?;
let raw = runtime.load_config_value("diagnostics.enabled");
# let _ = (enabled, raw);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`load_config<T>` 用 serde 做 typed decode，decode 失败返回 `CoreError`。需要持久化时由 foundation/project/settings owner 从外部配置源加载、迁移和原子保存；调用方不能把临时 config store 当作项目文件。

## 常见错误

- 解析错误的服务种类、名称、类型或生命周期状态会返回 `CoreError`。
- EventBus 的 disconnected/empty/timeout 分别通过 `EngineEventReceiveError`、`EngineEventTryReceiveError`、`EngineEventReceiveTimeoutError` 表示。
- 缓存旧 `Arc`、忽略 generation、在事件回调中递归同步发布同一高频 topic 都会破坏关停和背压边界。
