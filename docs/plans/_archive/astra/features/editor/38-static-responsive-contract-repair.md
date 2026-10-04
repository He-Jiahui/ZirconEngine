---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window/visible_rows.rs
tests:
  - tools/tests/test_editor_settings_window_product_wiring_contract.py
  - tools/tests/test_editor_zui_base_radius_hierarchy_contract.py
  - tools/tests/test_editor_zui_dynamic_product_view_family_contract.py
  - tools/tests/test_editor_zui_extension_details_property_rows_contract.py
  - tools/tests/test_editor_zui_inspector_filter_contract.py
  - tools/tests/test_editor_zui_module_header_budget_contract.py
  - tools/tests/test_editor_zui_welcome_visual_density_contract.py
  - tools/tests/test_editor_zui_workbench_layout_contract.py
  - tools/tests/test_editor_zui_workbench_panel_header_layout_contract.py
  - tools/tests/test_editor_zui_workbench_toolbar_radius_contract.py
---

# Editor Static Responsive Contract Repair

The Editor static contracts now follow the current flat-panel token hierarchy,
responsive welcome and command-palette layouts, hidden horizontal header
scrollbars, and Settings paint-module ownership. Asset Browser details retain
their nested content through the narrow tier. Inspector transform wrapping now
uses a 96px item minimum that matches each authored value group, preventing the
WrapBox from budgeting narrower slots than its children can occupy.

This is a source-and-contract alignment repair. It does not establish Cargo,
frame-time, allocation, p50, p95, or p99 performance acceptance.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E38 | Reconcile Editor static contracts with current responsive surface owners and repair narrow details/Inspector WrapBox constraints | implemented_pending_validation | Scoped `git diff --check` and TOML parsing passed. The combined local `test_editor_*.py` batch ran 1,241 tests; the 1,240 in-scope Editor tests passed. The sole remaining failure is `test_editor_workbench_preview_token_parity`, which targets excluded tooling preview CSS and was intentionally not changed. Managed Cargo admission previously stopped before a ticket was created because external worktree `E:\\Git\\zr_vm` is dirty; no coordinator status was queried and no performance claim is made. |
