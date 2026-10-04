---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-09-13
summary_slug: runtime-editor-host-naming-classification-drift
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - tools/tests/test_runtime_init_level_naming.py
  - zircon_runtime/src/text/module.rs
  - zircon_runtime/src/ui/surface/host_font_assets.rs
resolved_at: 2026-09-15
---

# runtime-editor-host-naming-classification-drift: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Runtime15 naming-boundary audit currently emits three unclassified production references: two RuntimeTargetMode::EditorHost references in zircon_runtime/src/text/module.rs and one Editor host-font asset reference in zircon_runtime/src/ui/surface/host_font_assets.rs; repair only the shared classifier and its exact regression assertions, preserving the production files as observed foreign scope.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Runtime15 naming-boundary audit currently emits three unclassified production references: two RuntimeTargetMode::EditorHost references in zircon_runtime/src/text/module.rs and one Editor host-font asset reference in zircon_runtime/src/ui/surface/host_font_assets.rs; repair only the shared classifier and its exact regression assertions, preserving the production files as observed foreign scope.` — python -B -m unittest tools.tests.test_runtime_init_level_naming.RuntimeInitLevelNamingTests.test_runtime_editor_metadata_owners_are_explicitly_classified -v

## 最低共享层根因

runtime_naming_boundary._classify_editor_reference lacks explicit owner mappings for the current text module EditorHost contract and the UI host_font_assets Editor contract, so valid host references fall through to unclassified-runtime-naming-reference.

## 架构修复验收

- The exact original unittest executes and passes with zero unclassified editor locations for the current metadata inventory.
- The full RuntimeInitLevelNamingTests suite and negative editor_authoring_state classification assertions remain green.
- py_compile and scoped git diff checks pass; no production text/module.rs or host_font_assets.rs bytes are changed.
- Submit a managed Windows-native Python ticket and independent Critical/Important/Moderate review before failure return; keep the lifecycle open for any pending evidence.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：runtime_naming_boundary._classify_editor_reference had no explicit owner mapping for the current RuntimeTargetMode::EditorHost references in zircon_runtime/src/text/module.rs and the Editor host-font contract in zircon_runtime/src/ui/surface/host_font_assets.rs, so valid metadata fell through to unclassified-runtime-naming-reference.
- 架构修复：Added explicit shared classifier ownership mappings for the text EditorHost contract and UI editor-host font contract, with exact regression assertions; production text/module.rs and host_font_assets.rs remain unchanged.
- 验证：Managed Windows-native Python ticket d2358eda94c44a298af7af5e0be000 ran python -B -m unittest tools.tests.test_runtime_init_level_naming -v against the immutable current-source manifest, executing all 6 tests with exit code 0; local focused and full suite also passed.
- 回传：Forward-fixed the Runtime15 naming boundary at the shared classifier, preserving the original lifecycle evidence and returning one canonical fixed artifact plus child return receipt.
