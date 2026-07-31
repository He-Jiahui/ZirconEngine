---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: app-entry-input-and-gamepad-storm-budget
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/12
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_entry_app/pointer_input
  - zircon_app/src/entry/runtime_entry_app/keyboard_input
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling/drain_budget.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
tests:
  - 125/500/1000 Hz input counter contract
  - bounded gamepad drain and rumble contract
  - reactive gamepad wake latency contract
---

# Runtime12：app entry输入与gamepad风暴预算

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-426 input/gamepad storm budget
- 修复责任计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 交接原因：pointer/device/gamepad producer 的 coalescing、bounded drain 与 reactive wake 合同由 Runtime12 所有。

## 失败现象与复现证据

pointer/raw device/gamepad事件仍逐项同步跨ABI，frame tick虽可合并但event forwarding没有motion/axis accumulator。当前普通物理键fallback已无中间String，gamepad已有256 events/2ms每帧drain预算，rumble已有32 effects/gamepad硬上限；这些只完成静态止损。Desktop reactive idle仍没有gilrs→winit wake路径，且queue peak/age/drop/coalesce不可见。

## 最低共享层根因

pointer、raw device 与 gamepad 仍逐事件跨 ABI forwarding，缺少 motion/axis accumulator；gilrs producer 也没有唤醒 Desktop reactive loop 的 owner。

## 架构修复验收

- motion/wheel/axis采用帧内latest或累加，button/touch/IME/lifecycle边沿保持lossless和有序；稳定key code转换不分配。
- gamepad producer能唤醒reactive loop；单帧drain有count/time预算并报告queue age/peak。
- rumble按gamepad/effect identity合并或设硬上限，完成/断连清理近活跃effects而非历史总量。
- 125/500/1000Hz和1k/10k burst记录ABI calls、alloc、queue、drop/coalesce、input-to-frame p95；结果回传PERF-MVP-426。

## 禁止临时方案

不得合并按钮按下/释放边沿，不得在主线程加无界retry，不得依赖连续`Poll`掩盖gamepad无wake。

## 修复结果与回传

2026-07-22 bounded-drain leaf：

- TDD 静态红态确认 `poll_gamepads` 仍以无上限 `while let` 耗尽 gilrs 队列，且没有 count/time budget 或 continuation。
- `gamepad/polling/drain_budget.rs` 现在单一拥有每帧最多 256 events / 2ms 的具名预算；即使 deadline 已到也允许首个事件，随后触达任一上限即停止。
- `polling.rs` 保持 gilrs 原事件顺序、error exit、disconnect/rumble cleanup 与 `gamepads.inc()` 语义；预算耗尽后只在 gilrs 可变借用结束后调用既有 `request_runtime_frame()`，未消费事件继续留在 gilrs queue。
- focused tests 固定 256/2ms 数值、count/time 边界、空队列非 continuation，以及 budget check -> next event -> record -> update -> gilrs increment -> cleanup -> exit/continuation 的源码顺序。
- source-bound snapshot `919` 冻结 exact2；`rustfmt +1.94.1 --check`、scoped static/source-order gate 与 `git diff --check` 通过，独立 review 为 `Critical 0 / Important 0 / Minor 0`。
- 当前没有 managed Cargo、真实 gilrs burst、queue age/peak 或 125/500/1000Hz 压力结果，因此本叶子不声明 test pass、performance accepted、fixed return 或 commit。

2026-07-22 rumble hard-cap leaf：

- TDD 静态红态确认 `RunningRumbleEffects` 的每 gamepad `Vec` 在 Add 成功后直接 push，没有 entry 上限或明确拒绝错误。
- `RUMBLE_EFFECTS_MAX_PER_GAMEPAD = 32` 现在限定每个 gamepad 的活跃 effect；每次请求先复用既有过期清理，零时长或零 motor 继续 no-op，真实 Add 在 `EffectBuilder::finish/play` 前按当前 bucket 计数并返回 `runtime_gamepad_rumble_effect_limit_reached`。
- 达限不会创建、播放或静默驱逐 effect；Stop、disconnect、expiry 与 shutdown cleanup 路径保持原 owner 和顺序。
- focused tests 固定 32 的 admission 边界，并以 fail-closed production-only source guard 锁定 clear expired -> admission -> finish -> play -> publish 以及既有 cleanup anchors。
- source-bound snapshot `925` 冻结 exact1；`rustfmt +1.94.1 --check`、scoped static/source-order gate 与 `git diff --check` 通过，独立 review 为 `Critical 0 / Important 0 / Minor 0`。
- 当前没有 managed Cargo、真实 force-feedback backend 或 rumble storm 压力结果，因此本叶子不声明 test pass、performance accepted、fixed return 或 commit。

2026-07-22 keyboard fallback leaf：

- TDD 静态红态确认普通 `KeyCode` fallback 以 `format!("{code:?}")` 为每个事件分配临时 `String` 后再执行 FNV-1a。
- `StableKeyCodeHasher` 现在直接实现 `fmt::Write`，让 `format_args!("{code:?}")` 的 UTF-8 bytes 按原 FNV offset/prime 写入，无中间 owned buffer；零 hash 仍按原合同归一到 1。
- Shift/Control/Alt/WASD 显式数值与全部 `NativeKeyCode` 映射保持不变；Escape/F12/ArrowUp/Numpad9 的旧 Debug-byte hash 用固定数值 parity 测试锁定。
- fail-closed production-only source guard 禁止 `format!` / `to_string` 回流，并要求 fallback 继续使用 `fmt::Write` sink。
- source-bound snapshot `930` 冻结 exact1；`rustfmt +1.94.1 --check`、scoped static gate 与 `git diff --check` 通过，独立 review 为 `Critical 0 / Important 0 / Minor 0`。
- 当前没有 managed Cargo、真实 keyboard storm allocation trace 或 125/500/1000Hz 压力结果，因此本叶子不声明 test pass、performance accepted、fixed return 或 commit。

Open state: `gamepad bounded drain、rumble hard cap 与 keyboard nonalloc fallback 已静态实现；pointer/axis coalescing、gilrs producer wake、managed Cargo 与压力证据仍待 Runtime12 完成`。

2026-07-23 current-source总复核：上述三项止损在`runtime_entry_app/**`74/74、3,673行、41 tests、组合指纹`bda129...b2c`中保持；PERF-MVP-426主表已删除“无预算/无界/分配”的过时现象描述。没有真实gilrs/force-feedback、125/500/1000Hz、queue age/peak或current-source Cargo证据，因此failure保持open且不得fixed return。
