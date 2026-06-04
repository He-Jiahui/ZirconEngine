mod observability;

use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

pub(super) use self::observability::{
    CONSOLE_DIAGNOSTICS_NAVIGATION_SPEC, PERFORMANCE_NAVIGATION_SPEC,
    RUNTIME_DIAGNOSTICS_NAVIGATION_SPEC, TELEMETRY_DASHBOARD_NAVIGATION_SPEC,
};

const UI_ASSET_EDITOR_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionUiAssetEditorHierarchyTab",
    "WorkbenchExtensionUiAssetEditorStatesTab",
    "WorkbenchExtensionUiAssetEditorBindingsTab",
];
const UI_ASSET_EDITOR_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.ui_asset_editor.hierarchy_tab.select",
        "WorkbenchExtensionUiAssetEditorHierarchyTab",
    ),
    action(
        "workbench.extension.ui_asset_editor.states_tab.select",
        "WorkbenchExtensionUiAssetEditorStatesTab",
    ),
    action(
        "workbench.extension.ui_asset_editor.bindings_tab.select",
        "WorkbenchExtensionUiAssetEditorBindingsTab",
    ),
];
const UI_ASSET_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionUiAssetEditorInventoryRow",
    "WorkbenchExtensionUiAssetEditorPanelRootRow",
    "WorkbenchExtensionUiAssetEditorEquipButtonRow",
    "WorkbenchExtensionUiAssetEditorRootPanelRow",
    "WorkbenchExtensionUiAssetEditorInventoryGridRow",
    "WorkbenchExtensionUiAssetEditorSelectedButtonRow",
    "WorkbenchExtensionUiAssetEditorBindingRow",
    "WorkbenchExtensionUiAssetEditorOutputRow",
];
const UI_ASSET_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.ui_asset_editor.inventory_row.select",
        "WorkbenchExtensionUiAssetEditorInventoryRow",
    ),
    action(
        "workbench.extension.ui_asset_editor.panel_root_row.select",
        "WorkbenchExtensionUiAssetEditorPanelRootRow",
    ),
    action(
        "workbench.extension.ui_asset_editor.equip_button_row.select",
        "WorkbenchExtensionUiAssetEditorEquipButtonRow",
    ),
    action(
        "workbench.extension.ui_asset_editor.root_panel_row.select",
        "WorkbenchExtensionUiAssetEditorRootPanelRow",
    ),
    action(
        "workbench.extension.ui_asset_editor.inventory_grid_row.select",
        "WorkbenchExtensionUiAssetEditorInventoryGridRow",
    ),
    action(
        "workbench.extension.ui_asset_editor.selected_button_row.select",
        "WorkbenchExtensionUiAssetEditorSelectedButtonRow",
    ),
    action(
        "workbench.extension.ui_asset_editor.binding_row.select",
        "WorkbenchExtensionUiAssetEditorBindingRow",
    ),
    action(
        "workbench.extension.ui_asset_editor.output.select",
        "WorkbenchExtensionUiAssetEditorOutputRow",
    ),
];
const UI_ASSET_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudUiAssetEditorButton",
    "WorkbenchExtensionUiAssetEditorPreviewButton",
    "WorkbenchExtensionUiAssetEditorValidateButton",
];
const UI_ASSET_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.ui_asset_editor.open",
        "WorkbenchHudUiAssetEditorButton",
    ),
    action(
        "workbench.extension.ui_asset_editor.preview.invoke",
        "WorkbenchExtensionUiAssetEditorPreviewButton",
    ),
    action(
        "workbench.extension.ui_asset_editor.validate.invoke",
        "WorkbenchExtensionUiAssetEditorValidateButton",
    ),
];
const UI_ASSET_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.ui_asset_editor.widget.edit",
    "workbench.extension.ui_asset_editor.widget.commit",
    "workbench.extension.ui_asset_editor.breakpoint.edit",
    "workbench.extension.ui_asset_editor.breakpoint.commit",
    "workbench.extension.ui_asset_editor.theme.edit",
    "workbench.extension.ui_asset_editor.theme.commit",
];

pub(super) const UI_ASSET_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.ui_asset_editor.open",
    "WorkbenchExtensionUiAssetEditorWorkspace",
    UI_ASSET_EDITOR_TAB_CONTROLS,
    UI_ASSET_EDITOR_TAB_ACTIONS,
    UI_ASSET_EDITOR_ROW_CONTROLS,
    UI_ASSET_EDITOR_ROW_ACTIONS,
    UI_ASSET_EDITOR_COMMAND_CONTROLS,
    UI_ASSET_EDITOR_COMMAND_ACTIONS,
    UI_ASSET_EDITOR_FIELD_ACTIONS,
);
const UI_BINDING_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionUiBindingBindingsTab",
    "WorkbenchExtensionUiBindingExpressionsTab",
    "WorkbenchExtensionUiBindingValidationTab",
];
const UI_BINDING_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.ui_binding.bindings_tab.select",
        "WorkbenchExtensionUiBindingBindingsTab",
    ),
    action(
        "workbench.extension.ui_binding.expressions_tab.select",
        "WorkbenchExtensionUiBindingExpressionsTab",
    ),
    action(
        "workbench.extension.ui_binding.validation_tab.select",
        "WorkbenchExtensionUiBindingValidationTab",
    ),
];
const UI_BINDING_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionUiBindingViewModelRow",
    "WorkbenchExtensionUiBindingHealthRow",
    "WorkbenchExtensionUiBindingAmmoRow",
    "WorkbenchExtensionUiBindingViewModelTableRow",
    "WorkbenchExtensionUiBindingHealthTableRow",
    "WorkbenchExtensionUiBindingConverterTableRow",
    "WorkbenchExtensionUiBindingValidationTableRow",
    "WorkbenchExtensionUiBindingOutputRow",
];
const UI_BINDING_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.ui_binding.view_model_row.select",
        "WorkbenchExtensionUiBindingViewModelRow",
    ),
    action(
        "workbench.extension.ui_binding.health_row.select",
        "WorkbenchExtensionUiBindingHealthRow",
    ),
    action(
        "workbench.extension.ui_binding.ammo_row.select",
        "WorkbenchExtensionUiBindingAmmoRow",
    ),
    action(
        "workbench.extension.ui_binding.view_model_table_row.select",
        "WorkbenchExtensionUiBindingViewModelTableRow",
    ),
    action(
        "workbench.extension.ui_binding.health_table_row.select",
        "WorkbenchExtensionUiBindingHealthTableRow",
    ),
    action(
        "workbench.extension.ui_binding.converter_table_row.select",
        "WorkbenchExtensionUiBindingConverterTableRow",
    ),
    action(
        "workbench.extension.ui_binding.validation_table_row.select",
        "WorkbenchExtensionUiBindingValidationTableRow",
    ),
    action(
        "workbench.extension.ui_binding.output.select",
        "WorkbenchExtensionUiBindingOutputRow",
    ),
];
const UI_BINDING_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudUiBindingButton",
    "WorkbenchExtensionUiBindingPreviewButton",
    "WorkbenchExtensionUiBindingValidateButton",
];
const UI_BINDING_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.ui_binding.open",
        "WorkbenchHudUiBindingButton",
    ),
    action(
        "workbench.extension.ui_binding.preview.invoke",
        "WorkbenchExtensionUiBindingPreviewButton",
    ),
    action(
        "workbench.extension.ui_binding.validate.invoke",
        "WorkbenchExtensionUiBindingValidateButton",
    ),
];
const UI_BINDING_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.ui_binding.field.edit",
    "workbench.extension.ui_binding.field.commit",
    "workbench.extension.ui_binding.widget.edit",
    "workbench.extension.ui_binding.widget.commit",
    "workbench.extension.ui_binding.converter.edit",
    "workbench.extension.ui_binding.converter.commit",
];

pub(super) const UI_BINDING_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.ui_binding.open",
    "WorkbenchExtensionUiBindingWorkspace",
    UI_BINDING_TAB_CONTROLS,
    UI_BINDING_TAB_ACTIONS,
    UI_BINDING_ROW_CONTROLS,
    UI_BINDING_ROW_ACTIONS,
    UI_BINDING_COMMAND_CONTROLS,
    UI_BINDING_COMMAND_ACTIONS,
    UI_BINDING_FIELD_ACTIONS,
);
const ICON_LIBRARY_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionIconLibraryIconTab",
    "WorkbenchExtensionIconLibraryUsageTab",
    "WorkbenchExtensionIconLibraryExportTab",
];
const ICON_LIBRARY_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.icon_library.icon_tab.select",
        "WorkbenchExtensionIconLibraryIconTab",
    ),
    action(
        "workbench.extension.icon_library.usage_tab.select",
        "WorkbenchExtensionIconLibraryUsageTab",
    ),
    action(
        "workbench.extension.icon_library.export_tab.select",
        "WorkbenchExtensionIconLibraryExportTab",
    ),
];
const ICON_LIBRARY_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionIconLibraryEditorCoreRow",
    "WorkbenchExtensionIconLibraryWarningRow",
    "WorkbenchExtensionIconLibraryUsageTopbarRow",
    "WorkbenchExtensionIconLibrarySaveTableRow",
    "WorkbenchExtensionIconLibraryPlayTableRow",
    "WorkbenchExtensionIconLibraryWarningTableRow",
    "WorkbenchExtensionIconLibraryLegacyTableRow",
    "WorkbenchExtensionIconLibraryOutputRow",
];
const ICON_LIBRARY_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.icon_library.editor_core_row.select",
        "WorkbenchExtensionIconLibraryEditorCoreRow",
    ),
    action(
        "workbench.extension.icon_library.warning_row.select",
        "WorkbenchExtensionIconLibraryWarningRow",
    ),
    action(
        "workbench.extension.icon_library.usage_topbar_row.select",
        "WorkbenchExtensionIconLibraryUsageTopbarRow",
    ),
    action(
        "workbench.extension.icon_library.save_table_row.select",
        "WorkbenchExtensionIconLibrarySaveTableRow",
    ),
    action(
        "workbench.extension.icon_library.play_table_row.select",
        "WorkbenchExtensionIconLibraryPlayTableRow",
    ),
    action(
        "workbench.extension.icon_library.warning_table_row.select",
        "WorkbenchExtensionIconLibraryWarningTableRow",
    ),
    action(
        "workbench.extension.icon_library.legacy_table_row.select",
        "WorkbenchExtensionIconLibraryLegacyTableRow",
    ),
    action(
        "workbench.extension.icon_library.output.select",
        "WorkbenchExtensionIconLibraryOutputRow",
    ),
];
const ICON_LIBRARY_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudIconLibraryButton",
    "WorkbenchExtensionIconLibraryFindUsageButton",
    "WorkbenchExtensionIconLibraryValidateButton",
];
const ICON_LIBRARY_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.icon_library.open",
        "WorkbenchHudIconLibraryButton",
    ),
    action(
        "workbench.extension.icon_library.find_usage.invoke",
        "WorkbenchExtensionIconLibraryFindUsageButton",
    ),
    action(
        "workbench.extension.icon_library.validate.invoke",
        "WorkbenchExtensionIconLibraryValidateButton",
    ),
];
const ICON_LIBRARY_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.icon_library.icon.edit",
    "workbench.extension.icon_library.icon.commit",
    "workbench.extension.icon_library.set.edit",
    "workbench.extension.icon_library.set.commit",
    "workbench.extension.icon_library.color_token.edit",
    "workbench.extension.icon_library.color_token.commit",
];

pub(super) const ICON_LIBRARY_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.icon_library.open",
    "WorkbenchExtensionIconLibraryWorkspace",
    ICON_LIBRARY_TAB_CONTROLS,
    ICON_LIBRARY_TAB_ACTIONS,
    ICON_LIBRARY_ROW_CONTROLS,
    ICON_LIBRARY_ROW_ACTIONS,
    ICON_LIBRARY_COMMAND_CONTROLS,
    ICON_LIBRARY_COMMAND_ACTIONS,
    ICON_LIBRARY_FIELD_ACTIONS,
);
const ACCESSIBILITY_AUDIT_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionAccessibilityAuditIssuesTab",
    "WorkbenchExtensionAccessibilityAuditRulesTab",
    "WorkbenchExtensionAccessibilityAuditFixesTab",
];
const ACCESSIBILITY_AUDIT_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.accessibility_audit.issues_tab.select",
        "WorkbenchExtensionAccessibilityAuditIssuesTab",
    ),
    action(
        "workbench.extension.accessibility_audit.rules_tab.select",
        "WorkbenchExtensionAccessibilityAuditRulesTab",
    ),
    action(
        "workbench.extension.accessibility_audit.fixes_tab.select",
        "WorkbenchExtensionAccessibilityAuditFixesTab",
    ),
];
const ACCESSIBILITY_AUDIT_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionAccessibilityAuditGameplayHudRow",
    "WorkbenchExtensionAccessibilityAuditContrastIssueRow",
    "WorkbenchExtensionAccessibilityAuditFocusIssueRow",
    "WorkbenchExtensionAccessibilityAuditContrastTableRow",
    "WorkbenchExtensionAccessibilityAuditFocusTableRow",
    "WorkbenchExtensionAccessibilityAuditTargetSizeTableRow",
    "WorkbenchExtensionAccessibilityAuditMotionTableRow",
    "WorkbenchExtensionAccessibilityAuditOutputRow",
];
const ACCESSIBILITY_AUDIT_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.accessibility_audit.gameplay_hud_row.select",
        "WorkbenchExtensionAccessibilityAuditGameplayHudRow",
    ),
    action(
        "workbench.extension.accessibility_audit.contrast_issue_row.select",
        "WorkbenchExtensionAccessibilityAuditContrastIssueRow",
    ),
    action(
        "workbench.extension.accessibility_audit.focus_issue_row.select",
        "WorkbenchExtensionAccessibilityAuditFocusIssueRow",
    ),
    action(
        "workbench.extension.accessibility_audit.contrast_table_row.select",
        "WorkbenchExtensionAccessibilityAuditContrastTableRow",
    ),
    action(
        "workbench.extension.accessibility_audit.focus_table_row.select",
        "WorkbenchExtensionAccessibilityAuditFocusTableRow",
    ),
    action(
        "workbench.extension.accessibility_audit.target_size_table_row.select",
        "WorkbenchExtensionAccessibilityAuditTargetSizeTableRow",
    ),
    action(
        "workbench.extension.accessibility_audit.motion_table_row.select",
        "WorkbenchExtensionAccessibilityAuditMotionTableRow",
    ),
    action(
        "workbench.extension.accessibility_audit.output.select",
        "WorkbenchExtensionAccessibilityAuditOutputRow",
    ),
];
const ACCESSIBILITY_AUDIT_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudAccessibilityAuditButton",
    "WorkbenchExtensionAccessibilityAuditAuditScreenButton",
    "WorkbenchExtensionAccessibilityAuditPreviewFixButton",
];
const ACCESSIBILITY_AUDIT_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.accessibility_audit.open",
        "WorkbenchHudAccessibilityAuditButton",
    ),
    action(
        "workbench.extension.accessibility_audit.audit_screen.invoke",
        "WorkbenchExtensionAccessibilityAuditAuditScreenButton",
    ),
    action(
        "workbench.extension.accessibility_audit.preview_fix.invoke",
        "WorkbenchExtensionAccessibilityAuditPreviewFixButton",
    ),
];
const ACCESSIBILITY_AUDIT_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.accessibility_audit.screen.edit",
    "workbench.extension.accessibility_audit.screen.commit",
    "workbench.extension.accessibility_audit.rule_set.edit",
    "workbench.extension.accessibility_audit.rule_set.commit",
    "workbench.extension.accessibility_audit.breakpoint.edit",
    "workbench.extension.accessibility_audit.breakpoint.commit",
];

pub(super) const ACCESSIBILITY_AUDIT_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.accessibility_audit.open",
    "WorkbenchExtensionAccessibilityAuditWorkspace",
    ACCESSIBILITY_AUDIT_TAB_CONTROLS,
    ACCESSIBILITY_AUDIT_TAB_ACTIONS,
    ACCESSIBILITY_AUDIT_ROW_CONTROLS,
    ACCESSIBILITY_AUDIT_ROW_ACTIONS,
    ACCESSIBILITY_AUDIT_COMMAND_CONTROLS,
    ACCESSIBILITY_AUDIT_COMMAND_ACTIONS,
    ACCESSIBILITY_AUDIT_FIELD_ACTIONS,
);
const MENU_FLOW_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionMenuFlowRoutesTab",
    "WorkbenchExtensionMenuFlowFocusTab",
    "WorkbenchExtensionMenuFlowSimulationTab",
];
const MENU_FLOW_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.menu_flow.routes_tab.select",
        "WorkbenchExtensionMenuFlowRoutesTab",
    ),
    action(
        "workbench.extension.menu_flow.focus_tab.select",
        "WorkbenchExtensionMenuFlowFocusTab",
    ),
    action(
        "workbench.extension.menu_flow.simulation_tab.select",
        "WorkbenchExtensionMenuFlowSimulationTab",
    ),
];
const MENU_FLOW_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionMenuFlowMainMenuRow",
    "WorkbenchExtensionMenuFlowStartScreenRow",
    "WorkbenchExtensionMenuFlowFocusPlayButtonRow",
    "WorkbenchExtensionMenuFlowStartNodeRow",
    "WorkbenchExtensionMenuFlowOptionsNodeRow",
    "WorkbenchExtensionMenuFlowMatchNodeRow",
    "WorkbenchExtensionMenuFlowExitRouteRow",
    "WorkbenchExtensionMenuFlowOutputRow",
];
const MENU_FLOW_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.menu_flow.main_menu_row.select",
        "WorkbenchExtensionMenuFlowMainMenuRow",
    ),
    action(
        "workbench.extension.menu_flow.start_screen_row.select",
        "WorkbenchExtensionMenuFlowStartScreenRow",
    ),
    action(
        "workbench.extension.menu_flow.focus_play_button_row.select",
        "WorkbenchExtensionMenuFlowFocusPlayButtonRow",
    ),
    action(
        "workbench.extension.menu_flow.start_node_row.select",
        "WorkbenchExtensionMenuFlowStartNodeRow",
    ),
    action(
        "workbench.extension.menu_flow.options_node_row.select",
        "WorkbenchExtensionMenuFlowOptionsNodeRow",
    ),
    action(
        "workbench.extension.menu_flow.match_node_row.select",
        "WorkbenchExtensionMenuFlowMatchNodeRow",
    ),
    action(
        "workbench.extension.menu_flow.exit_route_row.select",
        "WorkbenchExtensionMenuFlowExitRouteRow",
    ),
    action(
        "workbench.extension.menu_flow.output.select",
        "WorkbenchExtensionMenuFlowOutputRow",
    ),
];
const MENU_FLOW_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudMenuFlowButton",
    "WorkbenchExtensionMenuFlowPreviewFlowButton",
    "WorkbenchExtensionMenuFlowValidateFocusButton",
];
const MENU_FLOW_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.menu_flow.open",
        "WorkbenchHudMenuFlowButton",
    ),
    action(
        "workbench.extension.menu_flow.preview_flow.invoke",
        "WorkbenchExtensionMenuFlowPreviewFlowButton",
    ),
    action(
        "workbench.extension.menu_flow.validate_focus.invoke",
        "WorkbenchExtensionMenuFlowValidateFocusButton",
    ),
];
const MENU_FLOW_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.menu_flow.screen.edit",
    "workbench.extension.menu_flow.screen.commit",
    "workbench.extension.menu_flow.breakpoint.edit",
    "workbench.extension.menu_flow.breakpoint.commit",
    "workbench.extension.menu_flow.transition.edit",
    "workbench.extension.menu_flow.transition.commit",
];

pub(super) const MENU_FLOW_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.menu_flow.open",
    "WorkbenchExtensionMenuFlowWorkspace",
    MENU_FLOW_TAB_CONTROLS,
    MENU_FLOW_TAB_ACTIONS,
    MENU_FLOW_ROW_CONTROLS,
    MENU_FLOW_ROW_ACTIONS,
    MENU_FLOW_COMMAND_CONTROLS,
    MENU_FLOW_COMMAND_ACTIONS,
    MENU_FLOW_FIELD_ACTIONS,
);
const FONT_ATLAS_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionFontAtlasGlyphsTab",
    "WorkbenchExtensionFontAtlasKerningTab",
    "WorkbenchExtensionFontAtlasCoverageTab",
];
const FONT_ATLAS_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.font_atlas.glyphs_tab.select",
        "WorkbenchExtensionFontAtlasGlyphsTab",
    ),
    action(
        "workbench.extension.font_atlas.kerning_tab.select",
        "WorkbenchExtensionFontAtlasKerningTab",
    ),
    action(
        "workbench.extension.font_atlas.coverage_tab.select",
        "WorkbenchExtensionFontAtlasCoverageTab",
    ),
];
const FONT_ATLAS_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionFontAtlasInterUiRow",
    "WorkbenchExtensionFontAtlasLatinRangeRow",
    "WorkbenchExtensionFontAtlasCjkRangeRow",
    "WorkbenchExtensionFontAtlasLatinTableRow",
    "WorkbenchExtensionFontAtlasCyrillicTableRow",
    "WorkbenchExtensionFontAtlasCjkTableRow",
    "WorkbenchExtensionFontAtlasIconsTableRow",
    "WorkbenchExtensionFontAtlasOutputRow",
];
const FONT_ATLAS_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.font_atlas.inter_ui_row.select",
        "WorkbenchExtensionFontAtlasInterUiRow",
    ),
    action(
        "workbench.extension.font_atlas.latin_range_row.select",
        "WorkbenchExtensionFontAtlasLatinRangeRow",
    ),
    action(
        "workbench.extension.font_atlas.cjk_range_row.select",
        "WorkbenchExtensionFontAtlasCjkRangeRow",
    ),
    action(
        "workbench.extension.font_atlas.latin_table_row.select",
        "WorkbenchExtensionFontAtlasLatinTableRow",
    ),
    action(
        "workbench.extension.font_atlas.cyrillic_table_row.select",
        "WorkbenchExtensionFontAtlasCyrillicTableRow",
    ),
    action(
        "workbench.extension.font_atlas.cjk_table_row.select",
        "WorkbenchExtensionFontAtlasCjkTableRow",
    ),
    action(
        "workbench.extension.font_atlas.icons_table_row.select",
        "WorkbenchExtensionFontAtlasIconsTableRow",
    ),
    action(
        "workbench.extension.font_atlas.output.select",
        "WorkbenchExtensionFontAtlasOutputRow",
    ),
];
const FONT_ATLAS_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudFontAtlasButton",
    "WorkbenchExtensionFontAtlasBakeAtlasButton",
    "WorkbenchExtensionFontAtlasInspectGlyphButton",
];
const FONT_ATLAS_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.font_atlas.open",
        "WorkbenchHudFontAtlasButton",
    ),
    action(
        "workbench.extension.font_atlas.bake_atlas.invoke",
        "WorkbenchExtensionFontAtlasBakeAtlasButton",
    ),
    action(
        "workbench.extension.font_atlas.inspect_glyph.invoke",
        "WorkbenchExtensionFontAtlasInspectGlyphButton",
    ),
];
const FONT_ATLAS_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.font_atlas.font.edit",
    "workbench.extension.font_atlas.font.commit",
    "workbench.extension.font_atlas.range.edit",
    "workbench.extension.font_atlas.range.commit",
    "workbench.extension.font_atlas.size.edit",
    "workbench.extension.font_atlas.size.commit",
];

pub(super) const FONT_ATLAS_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.font_atlas.open",
    "WorkbenchExtensionFontAtlasWorkspace",
    FONT_ATLAS_TAB_CONTROLS,
    FONT_ATLAS_TAB_ACTIONS,
    FONT_ATLAS_ROW_CONTROLS,
    FONT_ATLAS_ROW_ACTIONS,
    FONT_ATLAS_COMMAND_CONTROLS,
    FONT_ATLAS_COMMAND_ACTIONS,
    FONT_ATLAS_FIELD_ACTIONS,
);
