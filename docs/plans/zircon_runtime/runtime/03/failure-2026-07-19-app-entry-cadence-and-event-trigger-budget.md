---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: app-entry-cadence-and-event-trigger-budget
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/03
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/window_events
  - zircon_app/src/entry/runtime_entry_app/device_events
  - zircon_app/src/entry/runtime_entry_app/surface_present/resize.rs
tests:
  - focused/unfocused idle cadence counters
  - unhandled event no-frame regression
  - duplicate resize no-work regression
  - frame_pump_publishes_only_the_final_control_flow
  - only_consumed_raw_device_motion_schedules_a_reactive_frame
  - handled_window_events_schedule_frames_but_duplicate_resize_does_not
  - game_cadence_throttles_unfocused_and_occluded_windows
  - mobile_cadence_has_explicit_foreground_and_background_limits
  - explicit_continuous_profile_ignores_visibility_throttling
  - low_power_cadence_consumes_runtime_immediate_and_after_demand
  - reactive_pending_request_survives_runtime_idle_until_next_pump
  - reactive_pending_request_still_polls_when_runtime_immediate_coalesces
---

# Runtime03：app entry cadence与事件触发预算

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-424 app-entry cadence/event relevance budget
- 修复责任计划：`docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md`
- 交接原因：runtime demand、event relevance 与最终 control-flow publish 的 cadence owner 由 Runtime03 协调 App entry 消费。

## 失败现象与复现证据

Desktop reactive cadence当前已消费`Idle/Immediate/After` runtime demand并合并Immediate wake，但window event判定除Close/Destroyed/Redraw外一律请求帧，device hook也在确认`PointerMotion`前请求帧；未处理gesture/axis/device noise仍触发runtime tick。每个pump在tick前后各发布一次control flow。Game/Continuous/Mobile保持`Poll`，Headless固定16ms；相同尺寸resize仍重做runtime resize、surface rebind与fallback resize。

## 最低共享层根因

App entry 缺少统一 event relevance authority，frame pump 重复发布 control flow，且 same-size resize 没有阻止 runtime resize、surface rebind 与 presenter resize 的 no-op gate。

## 架构修复验收

- 以显式event relevance表驱动frame request；未处理window/device事件请求数为0，按钮/IME/lifecycle边沿不得丢失。
- focused/unfocused/occluded和active timer/runtime demand分别决定cadence；连续模式有显示/调试需求时保持吞吐，后台与空闲有明确上限。
- frame pump每次只发布一次最终control flow；duplicate resize的runtime resize/rebind/presenter resize均为0。
- Runtime/Desktop/Editor/Headless各运行30秒idle与1k/10k event storm，记录tick/redraw/wake/CPU、p95和coalesce count；结果回传PERF-MVP-424。

## 禁止临时方案

不得用全局sleep掩盖事件风暴，不得把输入边沿按latest-value丢弃，不得为每种窗口各维护一套互不一致的cadence状态。

## 修复结果与回传

2026-07-23 current-source复核：`runtime_entry_app/**`74/74、3,673行指纹`bda129...b2c`确认demand/coalesced-wake止损已在当前源码，且8个frame-cadence unit tests覆盖reactive/continuous/headless局部状态机；没有current-source managed Cargo、30秒idle、event storm或duplicate resize counter。该静态证据不重开Runtime03已完成的schedule里程碑，只保持本post-completion failure open。

2026-08-10 current-source前向修复：window事件改由显式handled-event表决定frame request，raw device事件仅已消费的`PointerMotion`请求帧；`pump_frame_loop`每次只发布一次最终control flow；同一normalized viewport size在请求层与surface resize owner均提前返回，不再重复runtime resize、surface rebind或presenter resize。TDD源码红态、scoped rustfmt、diff-check与静态合同通过；未执行Cargo、30秒产品idle/event storm或产品计数验收。

2026-08-10 focus/visibility current-source前向修复：沿用同一`RuntimeFrameCadence`而非平台分支，按Bevy focused/unfocused profile与Unreal foreground idle gate收敛为Game前台`Poll`、失焦60 Hz、遮挡1 Hz，Mobile前台60 Hz、后台/遮挡1 Hz，显式Continuous始终`Poll`，Desktop reactive与Headless fixed不变。focus/occlusion先更新cadence，再仅为真实状态边沿请求帧，重复事件不会重置deadline；LowPower消费`Immediate/After/Idle`，control flow取runtime deadline与profile周期的较早者。shutdown summary新增accepted/coalesced/ignored request、focus/occlusion transition与low-power pump/suppression计数。二次审查发现的moved-value编译问题、重复状态风暴与LowPower demand丢失均已前向修复；TDD源码红态、scoped rustfmt、diff-check与静态合同通过；未执行Cargo、30秒产品idle/event storm或产品计数验收。

2026-08-10 第三轮审查前向修复：pump开始时消费旧request后，gamepad budget等pump内生产者仍可能创建新的capacity-one request；最终control-flow现在读取该pending token，Reactive/LowPower发布一次`Poll`，下一次`take_frame_request`消费后恢复`Wait/WaitUntil`。runtime `Idle`只清runtime deadline，`Immediate`即使与本地pending request合并也不会让该帧永久沉睡。新增上述两条回归；scoped rustfmt、diff-check与静态合同通过，未执行Cargo或产品计数验收。

Open state: `app_cadence_source_repair_complete_pending_managed_cargo_and_product_counters`。
