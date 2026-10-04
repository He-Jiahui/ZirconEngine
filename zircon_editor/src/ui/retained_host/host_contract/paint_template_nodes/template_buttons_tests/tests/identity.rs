use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    is_asset_browser_toolbar_chip_button, WorkbenchButtonKind,
};
use super::super::super::template_button_glyphs::ButtonGlyph;
use super::super::{button_glyph, button_kind, is_workbench_button};
use super::support::button_node;

#[test]
fn workbench_button_matches_button_nodes_without_icon_or_tab_nodes() {
    assert!(is_workbench_button(&button_node(
        "WorkbenchPrimaryButton",
        "Primary",
        "filled",
    )));
    assert!(is_workbench_button(&button_node(
        "WorkbenchButtonRoot",
        "Button",
        "outlined",
    )));
    assert!(is_workbench_button(&button_node(
        "OpenAssetBrowser",
        "Asset Browser",
        "secondary",
    )));
    assert!(is_workbench_button(&button_node(
        "PageTab0", "Effect", "ghost",
    )));
    assert!(is_workbench_button(&button_node("DockTab1", "Effect", "")));
    assert!(is_workbench_button(&button_node(
        "AssetBrowserKindAllChip",
        "All",
        "",
    )));
    assert!(is_workbench_button(&button_node(
        "WorkbenchModuleEffect",
        "Effect",
        "tab",
    )));
    assert!(is_workbench_button(&button_node(
        "WorkbenchModuleCompile",
        "Compile",
        "filled",
    )));
    assert!(!is_workbench_button(&button_node(
        "WorkbenchDrawerTabComponents",
        "UI Components",
        "tab",
    )));
    assert!(!is_workbench_button(&TemplatePaneNodeData {
        control_id: "WorkbenchMiniAdd".into(),
        role: "IconButton".into(),
        ..TemplatePaneNodeData::default()
    }));
}

#[test]
fn mixed_case_button_identity_preserves_kind_and_glyph() {
    let node = button_node("WorkbenchTrAsHAction", "Delete", "FiLlEd");

    assert_eq!(button_kind(&node), WorkbenchButtonKind::Danger);
    assert_eq!(button_glyph(&node), ButtonGlyph::Trash);
}

#[test]
fn asset_browser_kind_filter_dropdown_is_not_a_legacy_toolbar_chip_button() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserKindFilterDropdown".into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        ..TemplatePaneNodeData::default()
    };

    assert!(!is_asset_browser_toolbar_chip_button(&node));
    assert!(!is_workbench_button(&node));
}
