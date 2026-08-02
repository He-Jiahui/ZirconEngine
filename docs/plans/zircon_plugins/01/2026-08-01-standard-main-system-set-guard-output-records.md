# Plugins 01 standard main SystemSet manifest guard output record

Plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
Milestone: M1 / standard plugin SystemSet rollout
Status: guard implementation and independent final review completed; global audit has unrelated fixture-header debt
Date: 2026-08-01

## Scope Delivered

- manifest schema guard now requires every runtime module that declares `system_anchors` to declare its namespace-scoped standard `<namespace>.main` SystemSet.
- Modules with no registered scene-system anchors remain outside this requirement; editor/native modules retain their existing system-field restrictions.
- The rule permits a main set plus specialised sets, so `net.main` and `net.transport` can coexist without weakening the common cross-plugin ordering surface.

## Test-Driven Evidence

- RED: new focused test first failed for runtime modules with anchors but no sets and with only a specialised set.
- GREEN: `python -m unittest tools.tests.test_plugin_structure_audit_manifest_schema_modules.PluginStructureAuditManifestSchemaModulesTests.test_runtime_module_with_system_anchors_requires_standard_main_system_set -v` passed `1/1`.
- Regression: `python -m unittest tools.tests.test_plugin_structure_audit_manifest_schema_modules -v` passed `8/8`; `python -m py_compile tools/plugin_structure_audits/manifest_schema_modules.py tools/tests/test_plugin_structure_audit_manifest_schema_modules.py` and scoped `git diff --check` passed.
- Current source SHA-256: audit leaf = `99E0CF44DD0F7664E288D3AA5E630D721911C99A0D057AF77FF14477822E40CB`; test owner = `2445614D57387E00F9ACD583403076F9CB5E0623F5A2E362065B0CF935DE0FC6`.

## Current Repository Acceptance

After coordinator-authorized recovery of the stale Net exact6 slice, the repository audit reports no standard-SystemSet violation. All runtime modules with `system_anchors` now declare their namespace `.main` set. The same current audit reports two unrelated generated-header violations for `zircon_plugins/native_dynamic_fixture/plugin.toml` and `zircon_plugins/native_window_hosting/plugin.toml`. Both files are owned by active `frameworks04-m1-capability-audit-macro-projection-r1-20260730`; neither is a SystemSet failure or touched by this slice.

## 状态和完成项目

| 项目 | 状态 | 证据 |
|---|---|---|
| runtime anchor -> standard main set schema rule | completed | focused RED/GREEN contract and 8/8 module schema regression suite。 |
| specialised set coexistence | completed | `main + transport` control case accepted by focused test。 |
| repository-wide standard-SystemSet acceptance | completed | Net recovery removed the only SystemSet audit violation. |
| independent final review | completed | fresh read-only review returned `C0/I0/M0` for guard, Net recovery, and current audit evidence. |

## Remaining Scope

- 待 Frameworks04 恢复两处 generated fixture header 后，重跑完整 audit；该外部债不改变 standard-SystemSet guard acceptance。
- 此记录不关闭 Plugins01 M1、Net 计划或 Plugins01 全计划；Net focused Cargo 仍需实际完成。
