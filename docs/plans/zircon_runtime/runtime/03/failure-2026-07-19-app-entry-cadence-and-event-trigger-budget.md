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

Open state: `App demand已静态贯通；event relevance、single control-flow publish、focus/visibility profile与same-size resize早退及产品计数仍待跨Runtime10/Editor01完成`。
