---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: navigation-plan-output-record-archive-limit
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_plugins/05
related_code:
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/05/
tests:
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:/Git/ZirconEngine
---

# Plugins05：Navigation 产出记录归档上限治理交接

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-13 | 计划产出审计报告 `docs/plans/zircon_plugins/05-navigation.md` 的直接记录计数为 11，超过 10 条上限。该治理失败已写入 Navigation 自身编号子目录；由 Plugins05 在不丢失证据的前提下完成全部记录归档，不由 Editor09 搬运或删除业务记录。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1 完整验收后的 plan-output ownership audit
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：Navigation 产出记录及 `05/` archive 的主题和证据归 Plugins05 持有。

## 失败现象与复现证据

执行：

```text
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:/Git/ZirconEngine
```

得到：

```text
docs/plans/zircon_plugins/05-navigation.md:185: child-record-limit:
child plan contains 11 direct records; move all records to 05/
```

本轮未移动或压缩 Navigation 业务证据，避免由非 owner 对 M1-M6 状态作语义裁决。

## 最低共享层根因

Navigation 状态节已经链接多个编号 archive，但直接列表仍被审计器识别为 11 条 concrete records；其
owner 尚未完成超过十条时“全部迁移、父计划只留精简概述和 archive links”的治理闭环。

## 架构修复验收

- Plugins05 逐条核对 11 条证据，将 canonical concrete records 全部放入匹配前缀的 `05/` archive。
- `05-navigation.md` 保留 exact notice、精简 current-state overview 与可解析链接，不复制证据内容。
- plan-output audit 不再报告 Plugins05，且 failure coordinator link/schema audit 保持通过。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止删除历史证据、仅移动第 11 条、修改审计器阈值，或把 Navigation 记录迁到 Editor09/会话文件。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
