use super::super::super::super::data::TemplatePaneNodeData;
use super::super::is_workbench_button;
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
