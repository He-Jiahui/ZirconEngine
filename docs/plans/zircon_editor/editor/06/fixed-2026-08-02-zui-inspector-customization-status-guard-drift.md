---
handoff_kind: fixed
status: fixed
created_at: 2026-08-01
resolved_at: 2026-08-02
summary_slug: zui-inspector-customization-status-guard-drift
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
fixing_plan: docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_editor/editor_ui/11
plan_link_mode: child_record_only
related_code:
  - tools/tests/test_zui_docs_suffix_convergence.py
  - tools/tests/test_editor06_inspector_customization_contract.py
  - docs/editor-and-tooling/editor-command-workflow.md
  - zircon_editor/src/core/extension/inspector.rs
tests:
  - python -m unittest tools.tests.test_zui_docs_suffix_convergence tools.tests.test_editor06_inspector_customization_contract
---


# EditorUI11: ZUI status guard requires the retired inspector registration API

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：M2 InspectorCustomization descriptor and `.zui` surface hard cut.
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md`
- 交接原因：the EditorUI11 documentation-status guard owns the obsolete phrases and their
  cross-document assertions.

## 失败现象与复现证据

`test_editor_extension_contract_zui_guard_status_is_recorded` currently errors before assertions
because it reads the deleted transient file
`.codex/sessions/20260628-0317-zui-migration-validation.md`. Once that is bypassed, its required
phrases still demand every listed document, including read-only engine-code convention/review
files, contain `register_component_drawer`. Editor06 correctly updates current module
documentation to `InspectorCustomizationDescriptor` and `register_inspector_customization`;
retaining the old phrase only to satisfy the guard would publish a compatibility contract for a
removed API.

## 最低共享层根因

The UI11 guard treats a historical status phrase as a universal live API assertion. It conflates
the durable `.zui` suffix requirement with the retired component-drawer registration name and
relies on an ephemeral session note plus plan-definition documents outside the current
module-contract ownership boundary.

## 架构修复验收

- The UI11 guard proves the `.zui` authority from current module documentation and active source
  contracts without reading old plan-definition or session-note text.
- Its expected API names are `InspectorCustomizationDescriptor` and
  `register_inspector_customization`; it rejects retired `.ui.toml` and `.v2.ui.toml` documents
  through the new registration path.
- Focused UI11 guard and Editor06 inspector hard-cut contract pass without reinstating an old
  API token in code or documentation.

## 禁止临时方案

- Do not add `register_component_drawer` to a document, session note, test fixture, or source
  comment merely to satisfy the status guard.
- Do not recreate a deleted session note or change the read-only engine-code convention/review
  documents to preserve obsolete API wording.
- Do not weaken the `.zui` suffix check or permit legacy document suffixes.

## 修复结果与回传

- 根因：Historical status-receipt assertions mixed durable .zui authority with the retired component-drawer API and an ignored session note.
- 架构修复：The focused ZUI convergence guard owns suffix authority; the Editor06 inspector hard-cut guard owns current InspectorCustomizationDescriptor and register_inspector_customization source/document assertions; the obsolete status guard remains deleted.
- 验证：python -m unittest tools.tests.test_zui_docs_suffix_convergence tools.tests.test_editor06_inspector_customization_contract passed 13/13.
- 回传：Editor06 inspector customization may resume without restoring register_component_drawer, retired suffixes, or an ephemeral session note.
