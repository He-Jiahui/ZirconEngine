---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: plugin-manager-inspector-customization-guard-drift
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - tools/tests/test_editor12_plugin_manager_contract.py
  - zircon_editor/src/core/extension/inspector.rs
  - zircon_editor/src/core/editor_extension/contribution_descriptors.rs
tests:
  - python -m unittest tools.tests.test_editor12_plugin_manager_contract
  - cargo test -p zircon_editor --lib --locked
---

# Editor12: plugin manager descriptor guard still requires the retired component drawer type

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：M2 InspectorCustomization responsibility-chain hard cut.
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：the failing assertion belongs to the Editor12 plugin-manager contract suite and
  defines its descriptor ownership rule.

## 失败现象与复现证据

`test_extension_registry_keeps_contribution_descriptor_models_in_a_leaf_owner` asserts that
`core/editor_extension/contribution_descriptors.rs` contains `pub struct
ComponentDrawerDescriptor`. Editor06 deliberately removed that retired descriptor and made
`core/extension/inspector.rs::InspectorCustomizationDescriptor` the canonical class-level
Inspector declaration. The guard would fail even though the new Store/snapshot path is the
implemented architecture.

## 最低共享层根因

The Editor12 guard encodes the former EditorExtensionRegistry descriptor topology instead of the
current ownership boundary: generic plugin-manager descriptor leaves remain under
`core/editor_extension`, while Inspector customizations are owned by the dedicated
`core/extension/inspector` module and enter `ContributionBatch` as trait objects.

## 架构修复验收

- The Editor12 contract proves `ComponentDrawerDescriptor` is absent and
  `InspectorCustomizationDescriptor` is owned by `core/extension/inspector.rs`.
- The guard validates the public plugin path registers the descriptor through
  `register_inspector_customization`, then preserves the existing leaf-owner assertions for
  Drawer, menu, and importer descriptors that still belong to Editor12.
- Focused Python guard and the declared editor library acceptance pass without restoring a
  registry alias or compatibility type.

## 禁止临时方案

- Do not reintroduce `ComponentDrawerDescriptor`, `component_drawers`, or
  `register_component_drawer` as aliases or test fixtures.
- Do not move `InspectorCustomizationDescriptor` back into the legacy registry descriptor file
  just to satisfy a static string assertion.
- Do not weaken the guard into a broad file-exists assertion.

## 修复结果与回传

Open state: `待 Editor12 将 plugin-manager descriptor guard 前向迁至 InspectorCustomization
canonical owner；Editor06 M2 不得恢复旧 descriptor。`

## 产出记录与时间

- 2026-08-01：状态 `open_handoff_recorded`。发现 Editor12 静态守卫要求已删除的
  `ComponentDrawerDescriptor`；已路由为新的 canonical inspector owner 断言，要求前向修复。
