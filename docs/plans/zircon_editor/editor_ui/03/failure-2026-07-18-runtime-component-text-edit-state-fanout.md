---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-component-text-edit-state-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/component/state_reducer/text_input.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/surface/input/editable_text/mutation.rs
  - zircon_runtime/src/ui/surface/input/editable_text/state_transition.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
tests:
  - 10k-character continuous edit copy and mutation counter
  - change commit blur validation timing matrix
  - IME selection mirror atomic patch test
---

# Runtime component TextInput全文复制与多字段事务fanout

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：component text_input reducer及surface edit-state接缝
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 联动责任：EditorUI06提供generic component patch；Text09提供grapheme/layout generation结果。
- 交接原因：正文、selection、composition、validation和IME必须共享EditorUI03的唯一text edit-state authority。

## 失败现象与复现证据

PERF-MVP-270：每次KeyboardText clone当前全文构建`UiEditableTextState`，edit后再clone写primary和mirror，并独立写caret、selection及4个composition字段；change validation每次全量grapheme count。单字符输入形成8+字段替换和多份全文copy。该入口不同于PERF-MVP-258的accessibility action，但根因相同。

## 最低共享层根因

component reducer与surface editable-text各自投影text state；通用BTreeMap字段被当作正文authority，没有单一atomic patch或可复用grapheme/edit generation。

## 架构修复验收

- surface与component reducer共享typed text/selection/composition state，一次校验和提交并返回一个changed set/dirty union。
- 正文通过move/borrow或shared buffer传递，full-text copy不超过必要最终ownership一次；mirror只在projection边界生成。
- change validation只在启用对应规则时运行，grapheme长度随edit delta维护或复用Text09结果；commit/blur保持语义。
- 1/100/10k chars连续insert/delete/IME记录full-text bytes、field writes、patch/dirty、grapheme scans与CPU p95；UTF-8/read-only/selection/composition/mirror及Cargo通过。

## 禁止临时方案

- 不得让component和surface各自缓存一份可漂移text state。
- 不得以byte length替代grapheme规则；优化必须保持Unicode校验语义。

## 修复结果与回传

Open state: `等待EditorUI03联动EditorUI06/Text09回传single edit-state patch、bounded text copies与validation/grapheme规模证据`。
