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
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
tests:
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

Open state: `待 Runtime03 当前源码 WPR 基线与 profile-aware cadence 修复/豁免`。
