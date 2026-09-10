---
handoff_kind: fixed
status: fixed
created_at: 2026-09-10
summary_slug: plugin-package-role-manifest-schema-drift
origin_plan: docs/plans/astra/optimize/01-review-and-repair.md
fixing_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
origin_child_dir: docs/plans/astra/optimize/01
fixing_child_dir: docs/plans/zircon_plugins/12
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/editor_contribution_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - tools/audit_plugin_structure.py
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/plugin_structure_audits/manifest_schema_root_metadata.py
  - tools/tests/test_plugin_structure_audit_manifest_schema.py
tests:
  - python -m unittest tools.tests.test_plugin_structure_audit_manifest_schema
  - python tools/audit_plugin_structure.py --json
resolved_at: 2026-09-10
---

# Plugins12: package role manifest schema drift

## 来源执行者

- 来源计划：`docs/plans/astra/optimize/01-review-and-repair.md`
- 来源执行切片：Astra optimize review finding for generated plugin manifest schema conformance
- 修复责任计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 交接原因：generated plugin manifests now emit the existing Rust
  `PluginPackageRole` contract, while the shared Plugins12 schema audit still
  rejects the field before it can validate its value.

## 失败现象与复现证据

Before the repair, run:

```powershell
python tools/audit_plugin_structure.py --json
```

The audit reported `manifest_schema_violations = 3`:

- `zircon_plugins/editor_contribution_fixture/plugin.toml: package_role is not a known manifest root field`
- `zircon_plugins/native_dynamic_fixture/plugin.toml: package_role is not a known manifest root field`
- `zircon_plugins/plugin_sdk_examples/plugin.toml: package_role is not a known manifest root field`

The generated values are `test_fixture`, `test_fixture`, and `sample`.
They match the Rust `PluginPackageRole` enum, whose serialized values are
`production`, `developer_tool`, `sample`, and `test_fixture`.

## 最低共享层根因

The runtime manifest model and Rust declaration generator had already adopted
`package_role`, but the Python audit's root-field allowlist and root-metadata
validation did not model it. This made valid generated manifests fail the
repository-wide plugin structure gate, and therefore blocked independent
Plugins12 neural package acceptance.

## 架构修复验收

- `package_role` is a recognized optional root field.
- The audit accepts exactly `production`, `developer_tool`, `sample`, and
  `test_fixture`.
- An omitted `package_role` preserves the runtime `production` default;
  malformed or unsupported explicit values retain diagnostics.
- The three generated manifests pass the full plugin structure audit without
  weakening unknown-field detection.

## 禁止临时方案

- Do not remove `package_role` from generated manifests or suppress its audit.
- Do not accept arbitrary role strings.
- Do not make fixture and sample packages appear as production packages.

## 修复结果与回传

- 根因：The Python manifest-schema root allowlist and root-metadata validator did not model the existing Rust PluginPackageRole contract, so valid generated package_role values were rejected.
- 架构修复：Model package_role as an optional strict root field with the four serialized Rust enum values, preserving the runtime production default for an omitted field and diagnostics for malformed or unsupported explicit values.
- 验证：Managed ticket 4e1e71fe8c3b478ea4b3f2606dfb0fcf passed: 18 focused schema regressions, including the omitted-field production-default contract, and python tools/audit_plugin_structure.py --json completed with manifest_schema_violations=0.
- 回传：Plugins12 package-role manifests now validate against the same strict role contract used by the Rust runtime; the original failure is replaced by one fixed record and one child return receipt.

## 验证闭环记录

The focused RED regression reproduced the stale allowlist. The source repair
and local verification are complete. Managed ticket
`2a9d5bd55b4845f2b1165d854bcb34de` ran the focused regression successfully,
then failed because its first immutable copy omitted the directly executed
`tools/audit_plugin_structure.py` input. The failed ticket is retained as
validation-closure evidence and is not reusable. That script is now owned by
this lifecycle and will be included in the replacement immutable manifest;
independent review, fixed return, and coordinator closeout remain required.

Replacement ticket `e11e627497214b398bfb8c21d2b8b525` included the top-level
script and again passed the 17 focused tests. Its full audit then reached the
existing feature-provider projection path and failed only because the immutable
dependency copy omitted `tools/zircon_export`, which supplies the imported
package identity, template, and projection validators. The traceback names
`manifest_schema_feature_provider_packages.py` and `zircon_export`; it is a
second validation-input closure failure rather than a source regression. The
next immutable ticket will add that exact read-only dependency root while
keeping the same eight owned source files and acceptance command.

Historical replacement ticket `fc4b403b713e42fd9603fbfca9747da1` added the exact
`tools/zircon_export` dependency root and passed 17 focused regressions plus the
full audit. Independent review then identified that the acceptance wording did
not distinguish an omitted optional field from a malformed explicit value. The
Python audit now has a regression that locks the Rust `production` default for
an omitted `package_role`.

Replacement ticket `4e1e71fe8c3b478ea4b3f2606dfb0fcf` validates that amended
immutable source snapshot. It passed all 18 focused regressions and the full
`python tools/audit_plugin_structure.py --json` gate with
`manifest_schema_violations = 0`; it supersedes the historical ticket as the
reusable validation evidence for this lifecycle.
