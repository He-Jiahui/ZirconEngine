---
related_code:
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/config_path.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/time.rs
implementation_files:
  - zircon_runtime/src/core/runtime/events
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/time.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_runtime/src/core/runtime/config_store
  - zircon_runtime/src/foundation/runtime/config_manager_tests.rs
  - zircon_runtime/src/tests/time.rs
doc_type: module-detail
status: current
---

# 事件、配置与时间

## 事件总线

`core::framework::events` 定义中性 DTO 和接收协议，`core::runtime::events` 实现具体 `EventBus`。事件由字符串 topic 与 `serde_json::Value` payload 组成。

### 投递策略

| 策略 | 队列语义 | 适用场景 |
| --- | --- | --- |
| `Lossless` | 不主动丢弃；发布者可能承受背压 | 必须完整处理的控制/审计事件 |
| `BoundedDropOldest { capacity }` | 固定容量，满时丢最旧项 | 高频状态流，允许有限历史损失 |
| `Latest` | 只保留最新值 | UI/telemetry 当前状态 |

```rust
use std::num::NonZeroUsize;
use zircon_runtime::core::framework::events::EngineEventDeliveryPolicy;

let subscription = runtime.subscribe_events(
    "scene.changed",
    EngineEventDeliveryPolicy::BoundedDropOldest {
        capacity: NonZeroUsize::new(64).unwrap(),
    },
);

runtime.publish_event(
    "scene.changed",
    serde_json::json!({ "entity": 42 }),
);

let event = subscription.try_recv()?;
```

订阅接口还提供阻塞 `recv` 和带上限的 `recv_timeout`。返回的是 `Arc<EngineEvent>`，同一投递可共享 payload，调用者不应修改事件。

### 诊断

`event_bus_diagnostics()` 返回 topic、subscriber、published、delivered、dropped、queued、等待者、queue age 和 publish/delivery-lock timing 等累计快照。默认诊断模式是每 64 次采样一次日常 timing；可直接构造 `EventBus::new(EventBusDiagnosticsMode)` 改为完整、采样或关闭。

## Core 配置存储

Core 内置 `ConfigStore` 是线程安全的进程内 JSON key/value 存储。公开门面包括：

- `store_config_value(key, Value)`
- `load_config_value(key) -> Option<Value>`
- `load_config<T: DeserializeOwned>(key)`
- `snapshot_config_values() -> HashMap<String, Value>`

值在内部以 `Arc<Value>` 保存，typed load 可直接从共享 JSON 反序列化；快照才深复制所有 JSON 值。

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct GameplayConfig { max_players: u32 }

runtime.store_config_value(
    "gameplay",
    serde_json::to_value(GameplayConfig { max_players: 4 })?,
);
let config: GameplayConfig = runtime.load_config("gameplay")?;
```

注意：`CoreRuntime` 当前公开门面没有泛型 `store<T>`，调用端先用 `serde_json::to_value`，或通过 Foundation `ConfigManager` 写入。

## Foundation 配置持久化

`FoundationModule` 注册 `FoundationModule.Manager.ConfigManager` 为 Immediate manager。`DefaultConfigManager` 在构造时恢复并加载 JSON 文件，变更通过后台 worker debounce 后原子写入，并提供显式 `flush(timeout)` 与 `persistence_report()`。

配置路径优先级：

1. `ZIRCON_CONFIG_PATH`。
2. Windows 的 `LOCALAPPDATA` 或 `APPDATA` 下 `ZirconEngine/config.json`。
3. Unix 的 `XDG_CONFIG_HOME/ZirconEngine/config.json`。
4. `$HOME/.config/ZirconEngine/config.json`。
5. 回退为当前目录 `.zircon-config.json`。

多实例写同一目标由 commit fence 协调；恢复路径会尝试从 backup 恢复缺失目标。持久化失败通过 typed config error/report 暴露，不能假设内存更新意味着磁盘已经落盘。

## 外层帧时钟

`FrameClock` 从 `ClockSource` 获取 monotonic `Instant`。生产默认走系统 monotonic 快路径，测试/replay 可注入 `ManualClockSource`。

`tick_time(max_fixed_steps)` 读取 clock source；`advance_time_by(delta, max_fixed_steps)` 使用调用者提供的 delta。二者生成 `FrameTimeSnapshot`：

- `outer_frame_index`
- `raw_real_delta`
- `real_elapsed`
- `fixed_step_budget`
- 可选 `FrameTimeDiscontinuity`
- monotonic real `ClockDomainStamp`

Core 不拥有 World 的 virtual/fixed 时间。World 的 pause、time scale、fixed-step debt 和 commit 由 `LevelSystem` 派生并管理。

## 时钟不连续与 Rebase

应用前后台切换、暂停/恢复、窗口遮挡变化或 surface 重建可通过 `submit_clock_discontinuity` 触发 rebase。返回的 `FrameClockRebaseReceipt` 包含 generation、first-tick policy 和原因，并在下一次 frame snapshot 中出现一次。

```rust
use zircon_runtime::core::{
    ClockDiscontinuity, ClockLifecycleTransition,
};

let receipt = runtime.submit_clock_discontinuity(
    ClockDiscontinuity::ApplicationLifecycle(
        ClockLifecycleTransition::Resumed,
    ),
);
assert!(receipt.generation() > 0);
```

## 时间策略

`apply_time_policy(TimePolicyTransaction)` 原子验证并应用“之后创建或同步的 World 默认策略”，返回 previous、applied、generation 和 changed receipt。无变化不会推进 generation。

## 限制与实现状态

- EventBus 是进程内 topic bus，不是持久化日志或网络消息代理。
- JSON 配置 key 没有在 Core 层建立领域 namespace 强制规则，模块应使用稳定前缀或契约常量。
- `Lossless` 应谨慎用于高频生产路径，避免慢 subscriber 把背压传回 publisher。
- 事件、内存配置、Foundation 持久化、帧时钟和默认 World 时间策略均已实现并有专项测试。
