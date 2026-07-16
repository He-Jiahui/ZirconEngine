---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
resolved_at: 2026-07-15
summary_slug: plugin-structure-audit-report-fixture-drift
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_plugins/12
related_code:
  - tools/plugin_structure_audits/registration.py
  - tools/audit_plugin_structure.py
  - tools/tests/test_audit_plugin_structure_report.py
tests:
  - python -m unittest tools.tests.test_audit_plugin_structure_report
---


# Plugins 12：Plugin structure audit report fixture 未同步 descriptor 单源字段

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-13 | Editor09 按优先结构计划执行静态守卫时，`test_audit_plugin_structure_report` 四项在 `_registration_conformance()` mock payload 缺少新 descriptor 单源字段后统一抛 `KeyError`。该失败已归 Plugins12 审计工具 owner；不在 Editor09 资产代码或生产 audit 中加入缺省 fallback。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1 完整验收期间的 `engine-code-structure-convention` / review-findings 优先静态复核
- 修复责任计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 交接原因：Plugins12 的 `implementation_files` 明确拥有 `tools/audit_plugin_structure.py`、
  `tools/plugin_structure_audits/registration.py` 与插件结构审计测试；Editor09 只消费该门禁。

## 失败现象与复现证据

命令：

```text
python -m unittest tools.tests.test_audit_plugin_structure_report
```

四个 report tests 均在 `build_report()` 读取
`registration_conformance["runtime_plugin_descriptor_root_count"]` 时抛 `KeyError`。生产
`registration.py`/`audit_plugin_structure.py` 已要求三项新字段：

- `runtime_plugin_descriptor_root_count`
- `runtime_plugin_descriptor_single_source_violation_count`
- `frameworks_02_runtime_plugin_descriptor_status`

但 `tools/tests/test_audit_plugin_structure_report.py::_registration_conformance()` 的 mock 仍停在旧 payload。
聚合日志：`.codex/tmp/editor09-m1-structure-static-tests-20260713.log`。

## 最低共享层根因

插件结构审计 JSON contract 扩展后，Plugins12 的 report fixture 没有和生产 report schema 原子演进；
测试当前未验证 descriptor root/single-source/status 的 summary 与 Markdown projection，反而在旧 mock
边界提前崩溃。最低 owner 是插件结构审计 contract/fixture，而非 Editor asset registry。

## 架构修复验收

- report fixture 显式提供并断言三项新 descriptor 单源字段及 Markdown projection，不使用 `.get()`
  默认值掩盖 schema 漂移。
- `python -m unittest tools.tests.test_audit_plugin_structure_report` 全部自然通过。
- 聚合 plugin structure report/static audit 重跑，保留 descriptor single-source gate，不降低断言。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在生产 `build_report()` 用缺省值兼容旧 mock；应迁移 fixture 并新增新字段断言。

## 修复结果与回传

- 根因：Audit report fixture retained the old three-root runtime mirror payload after the production schema added Navigation.
- 架构修复：Migrated the fixture and assertions to the four-root schema and added explicit Navigation host-delivery and Markdown projection guards without production fallbacks.
- 验证：24 plugin audit/schema/export unit tests passed and the real plugin structure audit reported zero violations.
- 回传：Returned the synchronized plugin structure audit report fixture contract to Editor09.
