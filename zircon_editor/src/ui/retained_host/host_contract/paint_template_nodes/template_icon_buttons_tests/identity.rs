use super::super::is_workbench_icon_button;
use super::support::icon_node;

#[test]
fn icon_button_kind_matches_workbench_ids_and_excludes_status() {
    assert!(is_workbench_icon_button(&icon_node(
        "WorkbenchToolSelect",
        "zircon_editor_shell/toolbar/select.svg",
        false,
        40.0,
        40.0,
    )));
    assert!(is_workbench_icon_button(&icon_node(
        "WorkbenchMiniAdd",
        "zircon_editor_shell/controls/add.svg",
        false,
        36.0,
        36.0,
    )));
    assert!(!is_workbench_icon_button(&icon_node(
        "WorkbenchStatusTarget",
        "target",
        false,
        34.0,
        30.0,
    )));
}
