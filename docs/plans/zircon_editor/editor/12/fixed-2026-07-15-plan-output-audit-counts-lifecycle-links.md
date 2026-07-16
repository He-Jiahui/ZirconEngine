---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: plan-output-audit-counts-lifecycle-links
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py
tests:
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --self-test
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
resolved_at: 2026-07-15
---


# Tooling 01: plan output audit counts lifecycle links as concrete records

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：Editor12 plugin validation failure return / repository plan-output audit
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：误报由共享产出审计器的列表计数规则产生，Render18 只是首个超过十条简短 lifecycle 链接的上层消费者。

## 失败现象与复现证据

运行：

```powershell
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
```

审计器错误报告：

```text
docs/plans/zircon_runtime/render/18-advanced-lighting-features.md:397: child-record-limit: child plan contains 11 direct records; move all records to 18/
```

Render18 的 `## 状态与产出记录` 已包含精确归档提示，具体历史记录只有一条迁入链接；其余 11 行均为 `fixed-*`/`failure-*` canonical artifact 的简短状态链接。`write-plan-output-records` 明确规定这类 concise lifecycle link 不属于具体产出行，也不改变十条上限计数。

当前审计器的 `count_list_records` 只要列表行包含日期或具体验证签名便计数。`failure-2026-07-15-*`、`fixed-2026-07-14-*` 路径天然带日期，因此 11 条 lifecycle 链接被全部误计为 direct records。

## 最低共享层根因

候选支撑层包括 Render18 计划布局、Failure/Fixed 生命周期规则、Markdown 列表识别和具体记录计数。Render18 布局与 lifecycle 规则一致；最低共享故障位于 `audit_plan_output_records.py` 的列表记录分类，它没有在日期/签名判断前排除 concise `failure-*`/`fixed-*` plan link。

## 架构修复验收

- 为审计器增加 focused fixture：一个编号子计划可保留 11 条以上简短 `failure-*`/`fixed-*` 状态链接而不触发 `child-record-limit`。
- 保持真实的 11 条表格记录或带日期/验证证据的普通列表记录仍触发 `child-record-limit`。
- `--self-test` 通过；仓库审计不再报告 Render18 lifecycle-link 误报，当前其余真实违规继续如实报告。
- 重新运行 Failure handoff validator，确认 artifact 和双方计划链接仍满足唯一 canonical 生命周期。

## 禁止临时方案

- 不得删除或隐藏 Render18 的 `failure-*`/`fixed-*` 链接。
- 不得把 lifecycle artifact 当普通十条归档记录迁移、复制或重命名。
- 不得通过放宽表格记录、普通带证据列表或缺少提示的检查来消除误报。
- 不得加入兼容别名、静默 fallback、测试专用绕过或调用方例外。

## 修复结果与回传

- 根因：计划输出审计把 failure/fixed 生命周期链接的目标文件日期当成独立产出日期，因此一条纯状态链接被错误计入子计划记录上限。
- 架构修复：审计器先识别其可见文本不含日期或具体签名且全部链接目标为 failure-/fixed-.md 的简洁生命周期链接，并仅跳过这一类状态链接；普通带日期的产出记录仍会计数。
- 验证：audit_plan_output_records.py --self-test 通过；全库审计不再报告 Render18 或 Plugins05 的 child-record-limit，剩余四项为无关的既有 archive-notice 问题。
- 回传：生命周期状态链接不再虚增子计划记录，来源计划可继续以真实产出记录接受上限审计。
