---
related_code:
  - zircon_app/src/entry/runtime_entry_app
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
tests:
  - inline unit and source-contract tests: 41
  - current-source managed Windows Cargo pending
  - F0 cadence/input/fallback product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App runtime entry app当前源码复核（2026-07-23）

## 范围与当前基线

`zircon_app/src/entry/runtime_entry_app/**`当前源码 **74/74** 个Rust文件、**3,673** 行、**41** 条内联测试已逐文件阅读；path+Git-blob清单的SHA-256指纹为`bda12961981c5f20e15b7a11ac09610dba30e2a116251e9c328576e9baf37b2c`。其中14个tracked文件有外部未提交修改，`event_loop_policy/frame_cadence.rs`和`gamepad/polling/drain_budget.rs`为未跟踪当前源码；本轮只读保留，不吸收为本计划实现。

## 已成立的当前止损

- DesktopApp cadence已消费完整`Idle/Immediate/After` runtime demand，Immediate只发布一次coalesced proxy wake，idle pump可以被抑制。
- gamepad drain已有每帧最多256 events或2ms的双预算，耗尽后请求续帧；rumble已有每gamepad最多32个active effects及expiry/disconnect/shutdown清理。
- 普通`KeyCode` fallback直接把Debug输出写入FNV sink，没有中间`String`；有效UTF-8 drag/drop路径借用`Cow<str>`。

以上只有源码与局部测试合同，尚无current-source Cargo、真实gilrs/force-feedback或产品counter，不能标为动态验收。

## 剩余热点

- **PERF-MVP-424 / Runtime03**：除Close/Destroyed/Redraw外几乎所有window event都先请求frame，所有device event也在确认是否为PointerMotion前请求frame；每次pump在tick前后各调用一次`set_control_flow`。Game/Continuous/Mobile保持`Poll`，Headless固定16ms；same-size resize仍重发runtime resize、重绑surface并resize fallback presenter。
- **PERF-MVP-425 / Runtime10**：每个tick后一次性取得完整host-request Vec并逐项在主线程应用；没有count/bytes/time/backpressure，cursor/IME latest-state也没有合并。
- **PERF-MVP-426 / Runtime12**：pointer/IME/keyboard/gamepad仍逐事件同步跨ABI；gamepad虽有drain预算，却没有gilrs producer到reactive winit loop的wake路径，也没有queue peak/age/coalesce/drop观测。axis/motion/latest-state没有帧内accumulator，button/touch/IME边沿则必须保序。
- **PERF-MVP-023 / Render17+Runtime10**：native surface失败或强制capture时，fallback仍同步取得完整foreign RGBA frame再交给Softbuffer。

窗口属性只在创建期收集monitor Vec、clone title并匹配video mode，当前没有每帧caller；它不是新的MVP P0根因。surface bind中的环境变量读取也只发生于bind/rebind，不单独编号。

## 参考与验收

Bevy把focused/unfocused mode分开，并让Reactive显式选择是否响应device/user/window event；`reactive_low_power`关闭device-event触发。Unreal在非前台且world ready时进入idle mode。Zircon应采用同类“profile/focus/visibility/event relevance + runtime demand”合同，但阈值由产品counter决定，不照搬参考引擎常数。

动态门覆盖Runtime/Desktop/Editor/Headless focused/unfocused/occluded idle 30秒，以及1k/10k mixed window/device/host-request和125/500/1000Hz input/gamepad burst。记录tick/redraw/wake、control-flow publish、duplicate resize工作、ABI calls、drain count/time、queue peak/age/drop/coalesce、main-thread p95和CPU；真实fallback补RGBA copied bytes、Softbuffer像素与RenderDoc。完成前不进入`review.md`。

## 责任计划

- Runtime03：PERF-MVP-424，继续使用`runtime/03/failure-2026-07-19-app-entry-cadence-and-event-trigger-budget.md`。
- Runtime10：PERF-MVP-425及fallback ABI owner。
- Runtime12：PERF-MVP-426，继续使用`runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md`。
- Render17：PERF-MVP-023正常GPU present与fallback/readback边界。
