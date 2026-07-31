---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-focus-input-history-unbounded
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/focus_navigation
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
tests:
  - one-million focused-input bounded-live-bytes test
  - observer delivery ordering and no-duplicate test
  - optional diagnostic ring entry-byte-age budget test
---

# Runtime UI focus与focused-input历史无界增长

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface focus/frame逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 交接原因：input manager、dispatch outcome、focus path与timer/history ownership由EditorUI01统一管理。

## 失败现象与复现证据

PERF-MVP-282：focus change和每个focused keyboard/gamepad/accessibility input把owned event/route push到两个Vec；生产代码无drain/cap，长会话线性增长，并随每份surface frame全量复制。只有测试会clear。

## 最低共享层根因

当前状态DTO同时承担current state、事件队列和诊断历史三种ownership，没有dispatch batch消费边界或bounded telemetry sink。

## 架构修复验收

- dispatch outcome/observer是事件authority，批次消费后释放；UiFocusState只保留current/previous/pending/capture等状态。
- 可选诊断历史进入entry+byte+age有界ring并报告drop count，默认关闭。
- 1M inputs后live bytes有界、surface frame历史clone=0，delivery顺序不丢不重。
- modal restore、IME/capture、window focus、serde与Cargo/产品trace通过。

## 禁止临时方案

- 不得只在surface rebuild时clear而使未消费事件静默丢失。
- 不得给Vec设置大固定上限却没有消费语义、byte预算和drop diagnostics。

## 修复结果与回传

Open state: `等待EditorUI01回传dispatch-scoped focus events和bounded diagnostic history证据`。
