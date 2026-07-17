---
related_code:
  - zircon_runtime/src/input
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/script/vm/gameplay_host/input.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
tests:
  - zircon_runtime/src/input/tests
  - gameplay_key_query_reads_the_lightweight_snapshot_for_codes_and_names
doc_type: performance-review-evidence
status: in_progress
---

# Input 静态性能审查证据（2026-07-17）

## 范围与验收状态

已逐文件阅读 `zircon_runtime/src/input` 的全部 22 个 Rust 文件（12 个生产文件、10 个测试文件），并继续读取其直接 framework contract、dynamic API 输入转换、manager resolution 与 gameplay script 调用方。Cargo、长会话内存压力与高频输入产品测试尚未完成，因此该目录继续保留在 `pending.md`。

## P0：事件与录制历史可能无界增长

`DefaultInputManager::submit_event` 在一个 mutex 临界区内对每个输入事件执行：

- sequence 自增；
- `SystemTime::now()` 与 Unix epoch 转换；
- 按事件类型更新帧状态；
- clone 到 `state.events`；
- 原事件与时间戳追加到 `state.records`。

`begin_frame` 清理按钮边沿、滚轮/鼠标累加器、host requests、IME commit、手柄 transition、窗口与拖放事件，但不清理 `events` 或 `records`。生产调用检索没有发现常规帧循环调用 `drain_events()`；`drain_event_records()` 的生产消费者只在显式 recording capture。默认未录制长会话因此可能持续保留两份事件历史。

Bevy 的 `Messages` 文档同样明确说明：若不调用 `update`，buffer 会无限增长；其默认系统每次 update 交换双缓冲并清理旧 buffer。Bevy 的 `AccumulatedMouseMotion` 和 `AccumulatedMouseScroll` 则把高频 motion/scroll 合并成每帧累计值。Zircon 应借鉴这两个不同语义：边沿事件短期保留，高频连续值帧内合并，显式录制按需开启，而不是默认永久保留。

Runtime 12/07/03 的验收要求：

1. 明确 `events` 与 `records` 的 retention/consumer 语义；
2. recording 关闭时不为每个事件支付时间戳与第二份 clone；
3. 队列有上限、drop/coalesce 计数和 diagnostics；
4. 125/500/1000 Hz 输入持续 10 分钟，内存不随事件总量线性增长；
5. button/touch edge 不丢，raw motion 累加、cursor position 取 latest。

## P1：`frame_snapshot` 是全状态深拷贝

一次 `DefaultInputManager::frame_snapshot()` 会克隆或重新收集按钮三态、cursor requests、滚轮事件、触控、连接手柄、axis/button state、rumble、IME、窗口状态与拖放数据。该 API 适合一次帧边界快照，不适合窄查询反复调用。

已完成的低风险切片：`gameplay.key_pressed` 从完整 `InputFrameSnapshot` 改为轻量 `InputSnapshot`，并补数字 key code / logical key 测试。它仍会复制 pressed buttons；Runtime 12 后续应提供直接只读按钮查询或一次 frame snapshot 在脚本调用上下文复用，避免同帧每个脚本调用重复锁与 clone。

## P1：action evaluator 的规模复杂度

当前 evaluator 每次调用都会构建 consumed button/axis/context 的 `BTreeSet`，然后对每个 action 重新线性扫描全部 bindings。每个 axis binding 又分别线性扫描 frame 的 axis states 与 transitions。按 `A=actions`、`B=bindings`、`X=binding axes`、`G=gamepad axes` 粗略看，核心路径接近 `O(A*B + B*X*G)`，同时把 action id clone 到 pressed/value/transition 集合。

小 action map 不必提前复杂化；Runtime 12 需要先增加 16/64/256 actions 与 1/4/8 gamepads 的确定性 benchmark，再决定：

- set_action_map 时构建 action→binding range/index；
- frame snapshot 为 gamepad axis 提供 keyed lookup；
- action id intern/Arc 化或 state 用稳定 action index；
- empty consumed/context slices 走无集合构建 fast path。

## P1：每个 pointer event 同步修改 world

dynamic session 的 pointer-moved 处理在 submit input event 之外，还立即进入 `level.with_world_mut` 更新 camera，并同步 selection orbit target。OS 输入采样率因此直接驱动 world lock/mutation 次数。该项与上述 retention 问题叠加，必须在产品压力测试中同时测主线程 scope、manager resolve、mutex wait、world mutation 和 queue depth。

## 当前简单优化状态

| 切片 | 静态检查 | Cargo | 产品证据 |
|---|---|---|---|
| gameplay key query 改轻量 snapshot | rustfmt/diff 待统一末轮复核 | pending：共享 CPU lane 被占用 | pending |

以上状态不满足 `review.md` 验收条件。
