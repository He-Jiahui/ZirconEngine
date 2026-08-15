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
  - zircon_runtime/src/dynamic_api/session/registry/wake_registration.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/queue.rs
tests:
  - reactive_cadence_coalesces_requests_and_suppresses_idle_frames
  - continuous_cadence_never_suppresses_frame_pumps
  - headless_cadence_uses_fixed_wait_deadlines
  - headless_early_wake_does_not_pump_or_move_fixed_deadline
  - redraw_delivery_does_not_schedule_another_reactive_frame
  - wake_registration_channel_callback_invokes_the_runtime_sink
  - pending_asset_reload_work_keeps_the_reactive_loop_alive_until_completion
  - create_session_connects_the_runtime_wake_sink_to_the_asset_completion_producer
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
- reactive 帧请求由启动/恢复、非 redraw window event、device event 与 `ApplicationHandler::proxy_wake_up` 消费入口触发；重复请求合并为一次 tick。`RedrawRequested` 不会反向请求下一帧，避免反馈环重新变成持续空转。
- `pump_frame_loop` 在 reactive idle 时于 `tick_frame`、host-request drain 和 `request_redraw` 之前返回；真正泵帧仍保持 tick → host request → redraw 顺序。退出时 `runtime_frame_cadence_summary` 输出 requests/pumps/idle-suppressed/redraw 计数，为 WPR trace 提供同源核对。
- TDD 静态红态先确认 `frame_cadence.rs` 缺失；实现后 cadence anchors green，scoped rustfmt 与 `git diff --check` 通过。

2026-08-10 current-source 前向修复已接通 runtime-origin producer，并吸收二次审查问题：

- `RuntimeWakeRegistration::channel_wake` 将 session 级 V3 wake sink 适配为资源 channel callback；callback clone 共享同一 lifecycle。当前线程若正在执行同一 session 的 wake callback，同步 `destroy_session` 会在进入 close 前返回 `InvalidArgument`，避免等待自身 in-flight guard；其他线程仍由 disable 与 drain barrier 收敛。
- `DynamicSceneAssetReloadQueue` 使用 `ProjectAssetManager::subscribe_project_generation_wake` 接收容量 1 的 coalesced generation token。token publication 归属每一个成功提交的 project generation，而不是非空 `AssetChange` batch；因此 overflow/dirty reconciliation 即使产生空 change 列表，也会在 resource/typed events 提交后唤醒。open/watch/import/reimport/close 均经同一个 fenced publication owner，callback 只负责把 empty 转为 pending，不再为纯唤醒复制完整 `AssetChange`。
- queue 每帧最多 drain 一个 token；`has_pending_work` 统一覆盖 typed receiver backlog、carried event、reconciliation、pending task 与 deferred/ready 结果。即使单帧预算耗尽但尚未生成 task，`RuntimeDynamicSession::frame_demand` 也保持 `Immediate`，直至所有 queue-owned 工作收敛后才恢复 animation/Idle demand。
- 新增行为/源码合同覆盖 callback reentry 拒绝、容量 1 token 合并、receiver backlog 与 budget-exhaustion demand。scoped `rustfmt --edition 2021`、`git diff --check` 和 current-source 静态合同已通过；未执行 Cargo、WPR 或产品运行验收。

Open state: `source_repair_complete_pending_managed_cargo_wpr_and_product_wake_regression`。当前仍不得写 fixed return：需要受管 runtime/app focused/source-guard Cargo、Desktop 30 秒 WPR CPU/wakeup 证据，以及真实项目资源变更在 reactive idle 下完成 reload 的产品回归；不能用静态合同代替这些验收。
