---
related_code:
  - zircon_app/src/bin
  - zircon_app/src/entry
  - zircon_app/src/plugins
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/minimum-viable-engine-foundation.md
reference_sources:
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
tests:
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
doc_type: performance-review-evidence
status: in_progress
---

# MVP 入口静态审查证据（2026-07-17）

## 范围与状态

- 已逐文件阅读 `zircon_app/src/bin` 的 11 个 Rust 文件。
- 已逐文件阅读 `zircon_app/src/entry` 的全部 138 个 Rust 文件：95 个生产文件与 43 个独立测试文件，包括入口选择、runner、动态运行库宿主和 72 个 `runtime_entry_app` 文件。
- 已逐文件阅读 `zircon_app/src` 的其余 9 个 Rust 文件，包括 plugin-group builder、prelude、softbuffer presenter 与测试；`zircon_app` 基线 158 个 Rust 文件的静态逐文件阅读已完成。
- 产品启动/空闲 trace 与 Cargo gate 尚未完成，因此 `zircon_app` 继续保留在 `pending.md`，不得提前写入 `review.md`。

## 已确认热点

### 1. 空 host-request 批次每帧 JSON 往返

`RuntimeEntryApp::pump_frame_loop` 每帧调用 `RuntimeSession::drain_host_requests`。修改前，runtime 侧对空 `ZrRuntimeHostRequestBatchV1` 仍执行 `serde_json::to_vec`，宿主随后执行 `serde_json::from_slice` 并调用 ABI free。空缓冲在宿主端已被定义为“没有请求”，所以无需序列化。

当前切片已经：

- 在 runtime FFI 对空 `batch.requests` 返回 `ZrOwnedByteBuffer::empty()`；
- 把现有二次 drain 测试改为断言零长度输出；
- 通过 `rustfmt --check` 和 `git diff --check`；
- 因共享 Cargo CPU lane 被其他会话占用，尚未运行编译/测试，不计为验收通过。

责任计划：Runtime 10（dynamic API 契约）与 Runtime 07（热路径）。

### 2. 输入历史存在无界增长风险

`DefaultInputManager::submit_event` 对每个事件执行：

1. 获取全局输入状态互斥锁；
2. 调用 `SystemTime::now()`；
3. 将事件 clone 到 `state.events`；
4. 将事件与时间戳追加到 `state.records`。

生产源码检索没有找到常规帧循环对 `drain_events()` 的调用；`drain_event_records()` 的生产调用只存在于显式 `InputRecordingFrame::capture_from_manager`。因此未启用录制的长会话仍可能同时保留两份不断增长的事件历史。高轮询率 pointer/mouse-motion 会放大锁、时钟读取、clone 与内存增长。

这不是安全的小改：`drain_events`/recording 是公开语义，必须由 Runtime 12 设计“帧消费、按需录制或有界历史”，并提供丢弃计数和 1000 Hz 长窗口压力测试。Runtime 07 负责预算，Runtime 03 负责帧生命周期位置。

### 3. 高频 pointer-moved 同步修改 world

每个 winit pointer-moved 事件同步跨 ABI 后，动态 session 不只更新输入快照，还会立即：

- 解析 `InputManager` service；
- 锁定并追加两份事件历史；
- `level.with_world_mut` 更新 camera controller；
- 同步 orbit target 与 selection。

这使 OS 输入频率直接决定主线程 world mutation 频率。后续动态测试必须覆盖 125/500/1000 Hz，并区分可合并的 latest cursor position、可累加的 raw motion，以及不可丢失的按键/触控边沿。

### 4. 桌面 Wait 策略需要验证是否空闲忙转

`DesktopApp` 映射到 `ControlFlow::Wait`，但 `about_to_wait` 无条件执行 tick、host-request drain 和 `request_redraw`。主动重绘会再次唤醒事件循环，因此静态形态仍可能接近 continuous update。

Bevy 的 winit runner 明确区分 `UpdateMode::Continuous` 与带 `WaitUntil` 的 reactive 模式，并跟踪是否真的需要 redraw。Zircon 不能直接照搬，但需要 WPR 记录 editor/runtime desktop profile 的 30 秒 idle CPU、context switch 与 wakeup rate，再决定 cadence、`WaitUntil` 或 dirty redraw 策略。

### 5. bootstrap 报告重复生成 module descriptors

`bootstrap_entry_with_report` 先调用 `entry.module_selection_report()`；该方法遍历模块并生成 descriptors。随后 `entry.bootstrap()` 再生成一次 descriptors，注册时又 clone 每个 descriptor。普通 `bootstrap()` 也通过带报告路径，因此不是仅诊断命令才触发。

该问题是启动期而非每帧热区。修复前应增加 descriptor 调用计数测试，并让报告与注册共用同一批 descriptors，避免破坏 module activation 与报告一致性。

### 6. softbuffer 完整帧存在冗余全 surface 清零

修改前 `SoftbufferRuntimePresenter::present` 对 surface 执行 `buffer.fill(0)`，随后对完整 RGBA frame 覆盖每个像素。1280x720 下，这在真正的颜色转换之外额外写入 3,686,400 bytes/帧；60 Hz 时约为 221 MB/s 的冗余写流量，且 fallback 正是 CPU readback 已经较重的路径。

当前切片把转换提取为纯函数：完整 payload 直接覆盖，只有截断 payload 才预清零以保持未覆盖像素为黑。两个单测分别固定 no-preclear 与 defensive-clear 分支；Cargo 验收仍待共享通道释放。

## 候选但尚未定性

- viewer 在窗口建立时展示一次启动帧，随后无场景 redraw 可能再次生成并提交整张 RGBA gradient；需要当前源码 viewer trace。
- HDRI viewer 先解码 RGB32F 读取曝光/尺寸，随后 importer 可能再次解码相同 bytes；Render 13 已有更高优先级的串行 staging 问题，避免重复派单。
- profiling 宏在 capture 未激活时仍可能构造 scope name 并争用全局 mutex；需先完成 inactive fast-path 微基准。
- `apply_event_loop_policy` 每帧重复设置相同 control flow、非通用按键用 `format!("{code:?}")` 求 hash、gamepad rumble 清理在每个 host request 前执行，均先作为低优先级候选保留。

## 下一验证步

1. Cargo lane 可用后运行 entry source-guard、`zircon_runtime` host-request/profiling 聚焦测试与 package validator。
2. 构建相同配置的 runtime/editor first-frame-exit 产品，分别做 3 次启动测量。
3. 用 WPR/xperf 采集 30 秒 desktop idle 与连续 game frame，对比 wakeup、CPU sample、线程与 I/O。
4. 只有完成这些证据后，才按模块把 `zircon_app/src/entry` 从 `pending.md` 移入 `review.md`。
