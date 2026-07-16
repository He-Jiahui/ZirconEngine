---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: failure-return-plan-table-row-corruption
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_failures -v
resolved_at: 2026-07-15
---


# Coordinator01: failure return corrupts plan output table rows

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Editor07 回传 `command-eval-focused-document-projection` 时，lifecycle 正确移动为 fixed，但 `_replace_handoff_link` 将两侧计划内所有含 source link 的整行替换为普通 `- fixed 已修复` bullet。Editor07/08 的 4 列 `## 产出记录与时间` 表格因此被破坏；业务计划行已人工恢复，工具回归仍待 Coordinator01 修复。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 来源执行切片：Editor07 focused-document failure lifecycle return
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：破坏发生在 `FailureService._replace_handoff_link` 的通用计划重写逻辑，影响所有使用表格记录 failure 链接的编号计划，不属于 Editor07/08 业务架构。

## 失败现象与复现证据

调用 `failure return` 后，fixed artifact 与 graph lifecycle 均正确；但 `failures.py` 对每个匹配 source link 的
Markdown 行执行 `output.append(replacement)`，replacement 固定为普通 bullet。Editor07 表格中 2026-07-12 和
2026-07-14 两个完整记录行被吞掉；Editor08 表格中的 focused-document 行也被吞掉。Coordinator01 自身状态表
已有多个同形 fixed bullet，证明这不是单一业务计划格式特例。

## 最低共享层根因

link return 把“更新一个 Markdown link 的目标与状态摘要”和“替换整行计划内容”混成同一操作。验证器只检查
fixed 相对链接存在，不检查表格列结构或原行非链接文本保留，因此事务可以成功但计划记录发生信息丢失。

## 架构修复验收

- `_replace_handoff_link` 只改写匹配 source artifact 的 Markdown link token；保留该行前后文本、表格分隔符与其它链接。
- 普通 bullet handoff 行仍可收敛为 concise fixed 摘要，但表格行必须保持原列数并保留非链接证据。
- 增加至少包含一个普通 bullet、一个表格行、同一 source 多次引用和一个非目标链接的原子 return 回归。
- return 后运行 failure validator 与 plan-output audit，并断言原始表格内容未丢失；失败时继续保持事务回滚。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止要求所有业务计划停止在表格中引用 failure；禁止仅让 validator 忽略破坏后的 bullet。

## 修复结果与回传

- 根因：FailureService replaced an entire Markdown line after matching one source handoff link, so table cells and unrelated evidence were discarded.
- 架构修复：Failure return now rewrites only each matching Markdown link token inside table rows while retaining every cell and unrelated link; ordinary handoff bullets still collapse to one concise fixed summary.
- 验证：Focused FailureGraph regressions passed 10/10, coordinator artifact and Hook regressions passed 22/22, and validate_plan_failure_handoffs.py validated 125 artifacts with 0 errors.
- 回传：Editor07 can resume its affected gate; the canonical fixed handoff now preserves plan-table evidence during future returns.
