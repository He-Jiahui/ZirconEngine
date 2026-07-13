---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: extension-registry-finalize-coverage-guard-drift
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_plugins/01
related_code:
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs
  - tools/tests/test_plugin_extension_registry_finalize_coverage.py
tests:
  - python -m unittest tools.tests.test_plugin_extension_registry_finalize_coverage
---

# Plugins 01：Extension registry finalize coverage guard 未跟随 current read boundary

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-13 | Plugins01 M2 已声明 catalog/apply 入口在读取前 finalize，但当前 Python guard 仍寻找已删除/改名的 `apply_finalized_to_world`、旧 project report 函数签名等文本锚，导致结构门 error。需由 typed extension/finalize owner重新锁定真实语义边界，不在 Editor09 跳过该守卫。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1 完整验收期间的 `engine-code-structure-convention` / review-findings 优先静态复核
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：Plugins01 M2-T2～T4 明确拥有 `TypedExtensionPoint`、`FrozenExtensionTable`、
  `RuntimeExtensionRegistry::finalize()` 与 catalog/apply-before-read 合同；Editor09 不应猜测其新读取锚点。

## 失败现象与复现证据

命令：

```text
python -m unittest tools.tests.test_plugin_extension_registry_finalize_coverage
```

`test_catalog_and_apply_boundaries_finalize_before_runtime_reads` 抛 `ValueError: substring not found`。
当前事实包括：

- `apply_to_world` 已从 `self.apply_finalized_to_world` 改为
  `self.world_runtime_extension_plan()?.apply_to_world(world)`，仍先 `self.finalize()`；
- runtime catalog 当前在构造 `RuntimeExtensionCatalogReport` 前显式 `registry.finalize()`，而旧 guard
  仍按 `self.finalize()` 和旧 report 函数/读取 token 搜索；
- project catalog 函数当前名为 `runtime_extension_report_for_project`，旧 guard 签名已漂移。

聚合日志：`.codex/tmp/editor09-m1-structure-static-tests-20260713.log`。

## 最低共享层根因

Plugins01 的 finalize/apply 实现从直接 finalized helper 调用演进为显式 plan 构造与 report-owned
registry freeze，但静态守卫仍以旧函数名和旧具体调用文本代表语义，导致架构已变化时 guard 失去
coverage。最低 owner 必须重新裁决并锁定“所有 runtime read boundary 之前 registry 已 frozen”的当前
结构，而不是由上行 Editor plan 删除失败锚。

## 架构修复验收

- guard 覆盖 current world plan、component/UI/module/asset-manager apply，以及 runtime/project catalog
  两条 report 构造路径；每条路径能证明 read 前 finalize，不仅替换一个字符串。
- `python -m unittest tools.tests.test_plugin_extension_registry_finalize_coverage` 全部自然通过。
- Plugins01 module-local typed extension tests 与相关结构守卫重跑；不得削弱 20 typed fields freeze
  completeness、stable slot 或 hash-free frozen table 断言。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止恢复 `apply_finalized_to_world` wrapper 或旧 project report 名称只为满足文本测试；守卫必须迁到
  current canonical owner/语义。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
