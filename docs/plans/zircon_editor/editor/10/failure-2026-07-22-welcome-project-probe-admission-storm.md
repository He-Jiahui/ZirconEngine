---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: welcome-project-probe-admission-storm
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe/state.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe/job.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe/host.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe/projection.rs
tests:
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe/tests.rs
---

# Editor10：Welcome project probe准入风暴

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-559 Welcome project probe admission storm
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：Welcome draft probe 的 generation、canonical path/security owner 与 admission contract 由 Editor10 联动 Editor14 收敛。

## 失败现象与复现证据

现有generation拒绝迟到结果、Drop/cancel与typed failure合同正确，但每次draft变化立即cancel+submit Background/Index job；`validate_for_creation`和`ProjectAuthority::probe_draft`之间没有中断点。快速输入会排入大量已过时代际job并复制draft Path/String，取消只阻止最终commit，不能保证旧job不做filesystem/security probe。

## 最低共享层根因

取消只阻止迟到结果 commit，不能在 filesystem/security I/O 前 supersede 旧 generation；同时缺少跨 Welcome probe 的共享有界 admission owner。

## 架构修复验收

- first-event debounce与max feedback latency并存，latest target generation single-flight；queued stale generation在I/O前supersede，同target observer共享ticket。
- 联动Editor14让probe准入服从entry+bytes+oldest-age预算，ProjectAuthority继续独占path/security语义。
- 1/1k/1M keystrokes、32B/4KiB path、1ms/1s probe记录submitted/started/merged/cancelled、draft bytes、queue age、filesystem calls与UI p95/RSS；旧generation I/O接近0、内存与反馈延迟硬有界。
- missing/linked/invalid/submit failure/shutdown与F0产品trace/current-source Cargo通过前保持open。

## 禁止临时方案

不得只增加固定长debounce而无最大延迟；不得建立Welcome私有线程池；不得缓存一份可漂移的ProjectAuthority结果绕过canonical/security检查。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

## 产出记录与时间

- 2026-07-23 | Welcome local admission state machine | `open / source-and-static-review-complete` | `WelcomeProjectProbeState` now keeps only one latest pending draft generation, applies 50ms debounce with a 250ms absolute feedback deadline, shares unchanged pending/active work, and cancels a replaced worker before its next authority probe. Inline regression coverage includes invalid/missing/linked/current generation, submit/shutdown failure, 1k and 1M draft bursts, and a controlled post-validation cancellation boundary. `rustfmt --check` and source structural guards pass; final independent review is `Critical/Important/Minor = 0/0/0`. No Cargo, product trace, or performance acceptance is claimed.
- 2026-07-23 | Shared queue-budget dependency | `open / routed-to-editor14` | Global entry/bytes/oldest-age/RSS budgets and accepted/merged/backpressured admission outcomes are owned by [Editor14 welcome probe admission budget failure](../14/failure-2026-07-23-welcome-project-probe-admission-budget.md). This failure remains open until that return, current-source managed Cargo, and F0 storm/product evidence complete.
