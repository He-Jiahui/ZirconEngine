---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: accessibility-text-action-mutation-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/accessibility/action/text_state.rs
  - zircon_runtime/src/ui/accessibility/action/text_state/metadata.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace.rs
  - zircon_runtime/src/ui/accessibility/action/text/selection.rs
  - zircon_runtime/src/ui/accessibility/action/value.rs
  - zircon_runtime/src/ui/surface/input/editable_text
  - zircon_runtime/src/ui/tests/accessibility/value_actions.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
---

# Accessibility文本action串行mutation fanout

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/accessibility/action` 40个文件中的text/value完整链
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 联动责任：EditorUI01提供generation-owned accessibility target contract；Runtime Text03提供UTF-8/grapheme编辑边界。
- 交接原因：editable buffer、selection、composition与IME一致性属于EditorUI03。

## 失败现象与复现证据

PERF-MVP-258：SetTextSelection串行mutation caret/anchor/focus和composition start/end/text/restore共7次；SetValue/Replace先改正文再走同链，最多8次property mutation、binding report/diagnostic与dirty/invalidation。`accessibility_set_value_updates_editable_text_property`当前明确断言8份binding report、合计16个updates，为atomic patch前的可执行基线。

## 最低共享层根因

text edit state被拆成独立反射属性，accessibility action没有atomic typed patch；每个字段都走通用property mutation事务，导致同一语义动作重复查找、通知、格式化和失效。

## 架构修复验收

- 提供atomic typed text edit-state patch，一次提交text、caret、selection、composition；一次校验、binding report、dirty union和component event。
- unchanged字段不mutation；selection-only不重写已空且同offset的composition。
- 1/100/10k chars连续1k SetValue/Replace/Selection记录mutation calls、binding reports/updates、dirty commits、String copied bytes与CPU p95；每action transaction=1、invalidations≤1。
- read-only、max length/pattern/sanitize、UTF-8边界、collapsed/ranged selection、IME preedit/commit/cancel、callback order与AccessKit byte/character offset转换正确。
- editor search/rename/Console输入与current-source Cargo通过。

## 禁止临时方案

- 不得只把7个调用放进循环外但仍逐属性提交。
- 不得跳过composition清理导致IME状态残留。
- 不得用整份text clone cache替代typed delta与明确所有权。

## 修复结果与回传

Open state: `等待EditorUI03回传atomic text edit-state patch、mutation/dirty/copy counter、IME/AT行为矩阵与current-source Cargo`。
