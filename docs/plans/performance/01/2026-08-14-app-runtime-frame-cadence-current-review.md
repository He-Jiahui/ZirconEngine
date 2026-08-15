---
related_code:
  - zircon_app/src/entry/runtime_entry_app/application_handler
  - zircon_app/src/entry/runtime_entry_app/device_events
  - zircon_app/src/entry/runtime_entry_app/event_dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/window_events
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle
  - zircon_app/src/entry/tests/runtime_entry_source_guards
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/03/failure-2026-07-19-app-entry-cadence-and-event-trigger-budget.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
tests:
  - zircon_app/src/entry/tests/runtime_entry_source_guards
  - current-source scoped rustfmt check
  - managed Windows Cargo pending
  - 30-second WPR cadence matrix pending
doc_type: implementation-evidence
status: static_contract_drift_dynamic_pending
---

# App runtime frame cadence 当前源码性能复审（2026-08-14）

## 范围与证据边界

本轮完整复读 cadence 生产 owner 16 个文件、1,172 行，以及直接 source guards 7 个文件、
717 行，共 **23/23 个文件、1,889 行**。相关生产文件已有其他 Session 的未提交修改，本轮仅做
只读复审，不覆盖其实现。23 个文件的 `rustfmt +1.94.1 --edition 2021 --check` 通过；managed
Windows validator 在 Cargo 命令生成前仍被外部 unmanaged artifacts 阻断，因此以下是当前源码
结构证据，不是产品功耗验收。

## 当前修复与合同漂移

当前生产源码已经具备显式 window-event relevance、仅消费 `PointerMotion` 的 raw-device gate、
capacity-one frame request、每次 pump 一次最终 control-flow publication、same-size resize gate，
以及 focus/occlusion cadence 状态。这些前向修复与 Runtime03 failure 的 2026-08-10 记录一致。

但当前 `frame_loop.rs` 使用 `if should_pump { ... }`，干净的
`runtime_entry_source_guards/frame_loop.rs` 仍要求旧的 `if !should_pump { return; }` 源码形状。
静态探针结果为 `SOURCE_IF_SHOULD_PUMP=true`、`SOURCE_EARLY_RETURN=false`、
`GUARD_REQUIRES_EARLY_RETURN=true`、`REDRAW_REQUESTS_IN_PUMP=1`。因此该 source guard 在当前源码上
不可能通过；它只能证明测试与实现漂移，不能证明 cadence 行为回归。修复 owner 继续归 Runtime03，
不得为了让字符串测试变绿而倒退生产控制流。

## 剩余结构性瓶颈

`pump_frame_loop` 仍把 runtime tick/update 与 `window.request_redraw()` 绑定为同一个 pump 行为。
只要 cadence 允许 pump，即使 runtime update 完成后没有可见 presentation damage，也会请求 redraw。
Game profile 的失焦周期当前仍为 60 Hz，和交互前台相同；按静态配置，30 秒失焦空闲约产生
**1,800 次 tick + redraw 请求**。这不是一次常量微调能解决的问题，根因是“服务/模拟 deadline”
和“presentation dirty”尚未成为两个独立信号。

现有 shutdown counters 能看到 request/coalesce/focus transition，但缺少逐 profile 的 tick、update、
redraw、present、suppressed-present、deadline source、main-thread time 与功耗关联，无法判断空闲唤醒
是否来自 runtime demand、timer、输入 continuation 或可见 surface damage。

已有 `target/debug-idle-stability-20260809-002617/idle-evidence.json` 只能作为历史失败线索：进程在
第 1-4 秒均报告 unresponsive，并在第 5 秒前退出，exit code 2173。该记录没有 executable/source
fingerprint、tick/redraw/present counters 或 WPR 栈，不能判断是 cadence、启动失败还是旧二进制故障，
也不能作为当前源码修复前基线。

## 参考引擎结论

- Unreal `LaunchEngineLoop.cpp:5462-5495` 在 game 失焦且 world 条件允许时进入 idle；
  `5928-5934` 的 idle 路径直接阻止 tick/render 并等待 0.1 秒。它给出的原则是失焦无产品工作时
  同时抑制 update 与 render，而不是继续以交互频率 redraw。
- Bevy `winit_config.rs:13-23` 明确区分 focused continuous 与 unfocused reactive-low-power；
  `state.rs:650-724` 根据 update mode 和事件相关性发布 control flow。它支持以 workload/profile
  选择 cadence，但没有要求每次 update 都 present。

参考源码证明的是双信号调度边界，不提供 Zircon 的最终阈值。1 Hz、10 Hz 或 60 Hz 必须由编辑器
timer、音频/网络服务、动画和真实输入延迟矩阵共同决定，不能脱离产品数据硬改。

## 统一优化计划

继续使用 **PERF-MVP-424** 与既有 Runtime03 failure，不新增重复债务。Runtime10 通过新版本 frame
demand 合同分别表达 `next_update_deadline`、`presentation_dirty` 与显式 continuous/debug demand，
禁止原地扩形 V1。App pump 可在 update deadline 到达时 tick，但仅在 dirty generation 前进、
surface lifecycle 要求或显式 capture/present 请求时 redraw；capacity-one wake 只负责唤醒，不隐式
等价于 presentation dirty。

Runtime03 更新过期 source guard 为行为/counter gate，并保留相同尺寸 resize、事件相关性、pump 内
新 request 不丢失等合同。编辑器 timer、gamepad continuation、asset reload 和动画分别声明自己的
deadline/dirty 原因，不在 App 维护互不一致的平台特例。

## 动态验收矩阵

- Desktop/Game/Editor/Headless 各运行 focused、unfocused、occluded 30 秒 idle，以及
  1k/10k 无关事件风暴；另测 animation、timer、gamepad continuation 和显式 continuous。
- 记录 wake、pump、tick、update、redraw request、OS redraw、present、suppressed present、deadline
  source、main-thread p50/p95/p99、CPU、context switch 与功耗；WPR 同时核对 wait 状态和调用栈。
- 无 visible damage 时允许 update 而 redraw/present 为 0；重复无关事件不增加 frame request；
  显式 continuous 保持吞吐，输入边沿与 runtime `Immediate/After` 不丢失。
- 与 Unreal/Bevy 的架构经验只比较行为量级，不伪造跨机器绝对瓦数。当前机器需记录硬件、OS、
  电源计划、窗口状态、采样时长与环境基线，再比较修复前后同场景数据。

在 managed Cargo、30 秒产品 counters/WPR 和独立审查完成前，本模块继续保留在
`pending.md`，不进入 `review.md`。
