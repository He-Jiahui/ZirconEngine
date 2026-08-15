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

- 当前源码已将守卫前向迁移为：拒绝退休的 `ComponentDrawerDescriptor`，确认
  `core/extension/inspector.rs::InspectorCustomizationDescriptor` 是声明 owner，并确认
  `extension_materialization` 通过 `register_inspector_customization` 完成公开注册。
- 协调器不可变验证 job `7b73833d972249c0b791842401d024db`（run
  `a390d299c5a44933a06eb317138d6165`）中，精确 Python 守卫为 `1 passed / 0 failed`；随后 Cargo
  在进入 Editor 编译前因验证副本缺少 17 个 `templates/projects/renderable-empty` 文件而退出 `101`，
  该结果只证明副本物化阻塞，不能作为完整门通过。
- 协调器管理的当前源码 job `c08ec1474cfc4feab866e36cc9b18e27` 使用受管 Windows target
  执行 `cargo test -p zircon_editor --locked --verbose --lib`；它进入 Editor 编译后被当前共享工作树
  中与本守卫无关的 699 个编译错误阻断（114 warnings，0 tests）。因此声明的 Editor library 门尚未
  通过，本 handoff 继续保持 `open`，不执行 `failure return`。

## 产出记录与时间

- 2026-08-01：状态 `open_handoff_recorded`。发现 Editor12 静态守卫要求已删除的
  `ComponentDrawerDescriptor`；已路由为新的 canonical inspector owner 断言，要求前向修复。
- 2026-08-05：状态 `open_target_guard_green_full_gate_blocked`。目标 Python 守卫已由协调器验证通过；
  完整 Cargo 门分别受不可变副本物化缺件和共享工作树无关 Editor 编译错误阻断，未声明解决。
