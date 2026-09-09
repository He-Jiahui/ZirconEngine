---
related_code:
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/state_machine/mod.rs
  - zircon_runtime/src/core/runtime/random/service.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/events.rs
implementation_files:
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/state_machine
  - zircon_runtime/src/core/runtime/random
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md
tests:
  - zircon_runtime/src/core/runtime/tests
  - zircon_runtime/src/core/runtime/random/tests
doc_type: module-detail
---

# 时间、状态和随机数

## Core 时间与 World 时间

`CoreRuntime` 管理进程外层的单调真实时间、帧序号和默认 time policy；`scene::LevelSystem` 管理每个 World 的 virtual/fixed 时间、暂停和实际 simulation step。两者不能混用：Core 的一帧 tick 不等于每个 World 都推进同样的固定步。

```rust
use std::time::Duration;

let frame = runtime.advance_time_by(Duration::from_millis(16), 4);
// 或由 FrameClock 读取宿主单调时钟。
let frame = runtime.tick_time(4);
# let _ = frame;
```

`FrameTimeSnapshot` 包含 outer frame、real delta、累计时间、固定步预算和不连续证据。`max_fixed_steps` 是防止长时间暂停/卡顿后“追帧风暴”的显式预算，不能任意设置成无限大。

## 时钟不连续与策略

窗口前后台切换、休眠恢复、宿主 surface 重建或手动时钟变化应通过 `ClockDiscontinuity` 提交：

```rust
use zircon_runtime::core::{
    ClockDiscontinuity, ClockLifecycleTransition, ProductTimePolicies, ProductTimeProfile,
};

let receipt = runtime.submit_clock_discontinuity(ClockDiscontinuity::ApplicationLifecycle(
    ClockLifecycleTransition::Resumed,
));
let policies = ProductTimePolicies::for_profile(ProductTimeProfile::Client);
# let _ = (receipt, policies);
```

实际 discontinuity 构造应使用 owner 定义的原因/时间数据。返回的 `FrameClockRebaseReceipt` 表示 generation 变化，异步延迟帧或缓存时间戳在消费前应检查它。

`time_policy`、`time_policy_generation` 与 `apply_time_policy(TimePolicyTransaction)` 管理产品级策略变更。变更返回 `TimePolicyReceipt`，用于确认当前 generation；不要写共享全局静态变量来改变慢动作、暂停或 fixed step。

## Typed State

State machine 提供 `StateSpec`、`State<T>`、`NextState<T>`、`OnEnter<T>`、`OnExit<T>`、`OnTransition<T>`。一个类型是一个 state domain；切换需要显式排队/应用：

```rust
use zircon_runtime::core::{CoreRuntime, StateSpec};

#[derive(Clone, Default)]
struct RunMode;
impl StateSpec for RunMode {}

runtime.init_state::<RunMode>();
runtime.set_next_state(RunMode);
let transition = runtime.apply_state_transition::<RunMode>();
# let _ = transition;
```

hooks 通过 `register_on_enter`、`register_on_exit`、`register_on_transition` 安装。回调应短小、无阻塞并将副作用交给任务/模块，不要在 state hook 中重入 lifecycle activation。

## RandomService

`RandomService` 把 master seed 与命名 stream registry 集中到 runtime 内核，以支持可重现模拟、重放和状态检查点：

```rust
use zircon_runtime::core::CoreRuntime;

let runtime = CoreRuntime::with_random_seed(0x5EED);
let checkpoint = runtime.random_service().checkpoint();
# let _ = checkpoint;
```

具体 stream 注册/采样 API 由 random domain 定义。`RandomServiceState` 只恢复 seed authority，`RandomServiceCheckpoint` 还包含已注册 stream 的进度；后者用于确定性重演。不要在 gameplay/渲染路径使用隐式全局随机数生成器，否则无法与保存、网络或 replay 语义对齐。

## 限制

- `CoreRuntime::new` 和部分便利构造器在初始化失败时 panic；库/宿主需要可恢复错误时使用 `try_new` 或 `try_with_task_graph_options`。
- 真实时间不提供稳定 wall-clock 时间戳；需要日期/时区的功能应由专门服务提供。
- State 类型、time policy 和 random stream 不是自动序列化的项目资产；需要保存时使用所属 scene/project 的明确 DTO。
