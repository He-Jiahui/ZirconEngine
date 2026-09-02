# M4 证据

- Gate: design-ready
- Owner session(s): current session
- Changed scope: 统一 Workbench Button、Tab、Activity Rail Button 的 selected visual contract，使用现有 accent/text tokens 提供选中边框、选中前景与 tab underline。
- Manifest: `docs/plans/zircon_editor/editor_ui/manifests/m4-component-state-contract.yaml`
- Commands actually run: `python -m unittest tools.tests.test_editor_zui_workbench_layout_contract tools.tests.test_editor_zui_workbench_toolbar_radius_contract tools.tests.test_editor_zui_base_radius_hierarchy_contract tools.tests.test_editor_ui_focus_visible_contract tools.tests.test_editor_zui_product_interaction_event_contract`
- Result summary: 76 tests passed；disabled/focus/hover/pressed/selected 基础属性保持，selected 状态获得一致 accent 强调，不新增 raw color。
- Repaired failures: none
- Deferred external checks: Editor 产品截图、键盘 focus ring 实机验证、Penpot A2-P rendered parity。
- Evidence links: `workbench_button.zui`; `workbench_tab.zui`; `workbench_rail_button.zui`。
- Unlocks: M5 产品壳与 Penpot parity 验证。

## 状态优先级

组件继续声明 `selected`、`focused`、`hovered`、`pressed`、`disabled` 语义状态，并由既有 selector/runtime 状态优先级解析。新增 visual token 只补齐 selected 状态表达：

- Button：accent border + primary foreground。
- Tab：accent border + accent underline + primary foreground。
- Rail Button：accent border，既有 selected icon accent 保持。

所有颜色均来自 `$editor.*` semantic tokens；没有把 Penpot 色值直接写入组件。
