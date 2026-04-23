use crate::ui::layouts::views::welcome_pane_nodes;
use slint::Model;
use zircon_runtime::ui::layout::UiSize;

const WELCOME_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/welcome.ui.toml"
));

#[test]
fn welcome_bootstrap_layout_self_hosts_shell_sections() {
    let layout =
        crate::tests::support::load_test_ui_asset(WELCOME_LAYOUT_TOML).expect("welcome layout");

    for required_node in [
        "welcome_root",
        "outer_panel",
        "recent_panel",
        "recent_header_panel",
        "recent_list_panel",
        "main_panel",
        "hero_panel",
        "status_panel",
        "new_project_header_panel",
        "project_name_field",
        "location_field",
        "preview_panel",
        "validation_panel",
        "actions_row",
    ] {
        assert!(
            layout.contains_node(required_node),
            "welcome bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn welcome_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = welcome_pane_nodes(UiSize::new(1280.0, 720.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    for label in [
        "WelcomeOuterPanel",
        "WelcomeRecentPanel",
        "WelcomeRecentHeaderPanel",
        "WelcomeRecentListPanel",
        "WelcomeMainPanel",
        "WelcomeHeroPanel",
        "WelcomeStatusPanel",
        "WelcomeNewProjectHeaderPanel",
        "WelcomeProjectNameField",
        "WelcomeLocationField",
        "WelcomePreviewPanel",
        "WelcomeValidationPanel",
        "WelcomeActionsRow",
    ] {
        let frame = nodes
            .iter()
            .find(|node| node.control_id == label)
            .expect("welcome mount node")
            .frame
            .clone();
        assert!(
            frame.width > 0.0 && frame.height > 0.0,
            "expected `{label}` frame to be laid out by the bootstrap asset"
        );
    }

    let outer = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeOuterPanel")
        .expect("outer panel node");
    let recent = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeRecentPanel")
        .expect("recent panel node");
    let recent_header = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeRecentHeaderPanel")
        .expect("recent header node");
    let recent_list = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeRecentListPanel")
        .expect("recent list node");
    let main = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeMainPanel")
        .expect("main panel node");
    let hero = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeHeroPanel")
        .expect("hero node");
    let status = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeStatusPanel")
        .expect("status node");
    let new_project_header = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeNewProjectHeaderPanel")
        .expect("new project header node");
    let project_name = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeProjectNameField")
        .expect("project name node");
    let location = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeLocationField")
        .expect("location node");
    let preview = nodes
        .iter()
        .find(|node| node.control_id == "WelcomePreviewPanel")
        .expect("preview node");
    let validation = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeValidationPanel")
        .expect("validation node");
    let actions = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeActionsRow")
        .expect("actions node");

    assert_eq!(outer.role.to_string(), "Mount");
    assert!(recent.frame.x >= outer.frame.x);
    assert!(main.frame.x >= recent.frame.x + recent.frame.width);
    assert!(recent_header.frame.y >= recent.frame.y);
    assert!(recent_list.frame.y >= recent_header.frame.y);
    assert!(hero.frame.y >= main.frame.y);
    assert!(status.frame.y >= hero.frame.y + hero.frame.height);
    assert!(new_project_header.frame.y >= status.frame.y + status.frame.height);
    assert!(project_name.frame.y >= new_project_header.frame.y + new_project_header.frame.height);
    assert!(location.frame.y >= project_name.frame.y + project_name.frame.height);
    assert!(preview.frame.y >= location.frame.y + location.frame.height);
    assert!(validation.frame.y >= preview.frame.y + preview.frame.height);
    assert!(actions.frame.y >= validation.frame.y + validation.frame.height);
}
