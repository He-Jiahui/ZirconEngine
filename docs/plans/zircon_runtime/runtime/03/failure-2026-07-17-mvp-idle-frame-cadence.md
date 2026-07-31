---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: mvp-idle-frame-cadence
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/03
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/lifecycle.rs
tests:
  - reactive_cadence_coalesces_requests_and_suppresses_idle_frames
  - continuous_cadence_never_suppresses_frame_pumps
  - headless_cadence_uses_fixed_wait_deadlines
  - headless_early_wake_does_not_pump_or_move_fixed_deadline
  - redraw_delivery_does_not_schedule_another_reactive_frame
  - WPR 30-second idle CPU and wakeup trace for Desktop profile
  - continuous Game profile frame-cadence regression
---

# Runtime03：Desktop Wait 与无条件 redraw 的空闲 cadence 待验证

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F0 frame-loop 静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md`
- 交接原因：profile-aware cadence 属于 Runtime03 帧循环契约，不能由单个 Desktop/Editor caller 临时限帧。

## 失败现象与复现证据

Desktop 路径设置 `ControlFlow::Wait`，但 `about_to_wait` 仍无条件 tick、drain host requests 并 `request_redraw()`。静态形态可能把本应事件驱动的桌面/编辑器空闲期变成持续唤醒；是否构成瓶颈必须由当前源码 WPR 线程/唤醒 trace 证明。

## 最低共享层根因

根因候选位于 Runtime03 的产品 profile cadence 契约：Game 连续帧、Desktop/Editor reactive 帧和 Headless cadence 尚未通过同一可测状态机明确区分。

## 架构修复验收

- 当前源码 Desktop/Editor 空闲 30 秒记录 CPU、wakeups、redraw/tick/host-drain 次数。
- 分离 Game continuous 与 Desktop/Editor reactive/WaitUntil 策略，保留输入、动画、定时器和后台完成事件的正确唤醒。
- 设定 idle CPU/唤醒预算，并验证持续游戏帧率不退化。

## 禁止临时方案

- 不得全局限帧导致 Game profile 降速。
- 不得跳过 host request 或后台完成事件来制造低 idle CPU。

## 修复结果与回传

2026-07-18 当前实现已落 profile-aware cadence owner：

- `RuntimeFrameCadence` 将 `Game` / `Continuous` / `Mobile` 保持为 `ControlFlow::Poll` 连续帧；`DesktopApp` 改为 `ControlFlow::Wait` reactive 帧；`Headless` 使用私有命名常量 `HEADLESS_FRAME_INTERVAL` 与持久 `next_deadline` 驱动 `WaitUntil`。提前的 OS/proxy wake 不会额外泵帧，也不会把固定周期向后漂移。
- reactive 帧请求由启动/恢复、非 redraw window event、device event 与 `ApplicationHandler::proxy_wake_up` 消费入口触发；重复请求合并为一次 tick。`RedrawRequested` 不会反向请求下一帧，避免反馈环重新变成持续空转。当前只接通了 proxy consumer，运行时内部尚无已证明的 producer。
- `pump_frame_loop` 在 reactive idle 时于 `tick_frame`、host-request drain 和 `request_redraw` 之前返回；真正泵帧仍保持 tick → host request → redraw 顺序。退出时 `runtime_frame_cadence_summary` 输出 requests/pumps/idle-suppressed/redraw 计数，为 WPR trace 提供同源核对。
- TDD 静态红态先确认 `frame_cadence.rs` 缺失；实现后 cadence anchors green，scoped rustfmt 与 `git diff --check` 通过。

Open state: `code_complete_pending_managed_cargo_wpr_and_runtime_origin_wake_producer`。当前仍不得写 fixed return：需要受管 `zircon_app` focused/source-guard Cargo，Desktop 30 秒 WPR CPU/wakeup 证据，以及 runtime 内部长动画/定时器/后台完成任务通过 `EventLoopProxy::wake_up()` 形成真实 producer 的产品回归；不能用静态状态机代替这三项验收。
