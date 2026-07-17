---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: editor-text-input-shared-string-type-inference
origin_plan: docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md
fixing_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/19
fixing_child_dir: docs/plans/performance/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch.rs
tests:
  - cargo test -p zircon_editor --lib gateway:: --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib text_input --locked --jobs 1 -- --test-threads=1
resolved_at: 2026-07-17
---


# Performance01: text-input SharedString inference regression

## 产出记录与时间

| 时间 | 来源门禁 | 状态 | 失败证据 | 修复责任 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | Editor01 M2.2 gateway matrix current-source gate | `待修复（open）` | managed reservation `5ef7b68c61e840e18a9166111b01615c`、job `a4ff529bd2d64b4984fd09783e31af68`、run `5636833c80374faf974920f821287952` 在编译 `zircon_editor` 时得到 `edit.rs:26/30/45/49` 四条 E0282，exit 101；gateway 测试未开始。 | Performance01 PERF-MVP-167 明确 `String -> SharedString` 转换边界并恢复 current-source 编译，不回退已完成的单次转换/单次 focus move 优化。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`
- 来源执行切片：Layout19 retained text-input focus/edit current-source compilation；失败由 Editor01 M2.2 `SessionGateway` 双实现合同矩阵首先观测
- 修复责任计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 修复责任记录：`docs/plans/performance/01/2026-07-17-editor-host-window-static-review.md` 的 PERF-MVP-167
- 交接原因：当前源编译失败位于 Layout19 text-input focus/edit owner，并由 PERF-MVP-167 的单次 `SharedString` 转换改造引入；必须由 Performance01 保留优化语义并修复类型边界，其他计划不得在 gateway 或测试 filter 层规避 UI owner 编译错误。

## 失败现象与复现证据

Performance01 将 insert/backspace 的编辑结果从 `String` 转为共享字符串一次，再同时交给 focus state 与 callback。当前实现写成未定型的 `let value = value.into()`；赋值点、tuple 返回和下游函数参数共同参与推断，rustc 无法选择目标类型，并将不确定性级联到 `focus.edit_target_id()`。

```text
error[E0282]: type annotations needed
  --> zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs:26:17
  --> zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs:30:25
  --> zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs:45:17
  --> zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs:49:25
```

## 最低共享层根因

`dispatch_text_focus_value` 已把 `target_id` 与 `value` 定义为 `SharedString`，但调用者在 mutation block 内先克隆 `value` 写回 state，再把 tuple 返回到调用点。无显式目标类型时，`Into<_>` 在 block 内没有单一约束；这是 Performance01 转换边界的类型标注缺失，不是 Editor01 gateway 合同错误。

## 架构修复验收

- insert/backspace 在唯一 `String -> SharedString` 转换点显式定型为 `SharedString`。
- 保留 control-character 过滤、空插入 idle、空 backspace idle、focus 单次 move、state/callback 共享同一字符串值的行为。
- 不恢复临时过滤 `String`、重复 `to_string()`、focus clone 或兼容 wrapper。
- current-source `text_input` 聚焦测试与 Editor01 `gateway::` 上行门均越过编译并通过。

## 禁止临时方案

- 不得用删除 PERF-MVP-167 优化或恢复双重字符串复制来绕过类型推断。
- 不得在 gateway 代码、测试 filter 或 Cargo feature 上隐藏这个 UI owner 编译错误。
- 不得修改 `dispatch_text_focus_value` 的 `SharedString` 合同为泛型或无类型字符串。

## 修复结果与回传

- 根因：Performance01 PERF-MVP-167 moved the sole String-to-shared conversion inside the text-input mutation block as unconstrained Into<_>; state assignment, tuple return, and callback use did not provide a unique target, producing E0282 and cascading focus inference errors.
- 架构修复：Explicitly typed the two unique insert/backspace conversion boundaries as retained-host SharedString while preserving control-character filtering, idle behavior, one focus move, one conversion, shared state/callback value, and the hard-cut dispatch contract without wrapper or compatibility fallback.
- 验证：Managed CPU reservation 0306436c9d244f1485c25a783e4c29a6, job c5cead50ed4d479dac0749a1084da6a3, run a88babe23ffd4fc3bb0a302b42e2202a ran cargo test -p zircon_editor --lib text_input --locked --jobs 1 -- --test-threads=1 on current source: exit 0, 9 passed, 0 failed, 3349 filtered; rustfmt check and scoped git diff --check passed; independent review Critical/Important/Minor = 0/0/0.
- 回传：Returned the Performance01 SharedString type-boundary repair to Layout19; the original Editor01 gateway compile blocker is removed and no gateway-side workaround was introduced.
