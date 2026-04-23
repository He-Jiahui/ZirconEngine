use crate::ui::layouts::views::console_pane_nodes;
use slint::Model;
use zircon_runtime::ui::layout::UiSize;

const CONSOLE_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/console.ui.toml"
));

#[test]
fn console_bootstrap_layout_self_hosts_shell_sections() {
    let layout =
        crate::tests::support::load_test_ui_asset(CONSOLE_LAYOUT_TOML).expect("console layout");

    for required_node in ["console_root", "text_panel"] {
        assert!(
            layout.contains_node(required_node),
            "console bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn console_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = console_pane_nodes(UiSize::new(640.0, 280.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    let text_panel = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleTextPanel")
        .expect("console text panel node");
    assert_eq!(text_panel.role.to_string(), "Mount");
    assert!(text_panel.frame.width > 0.0 && text_panel.frame.height > 0.0);
    assert!(text_panel.frame.x >= 0.0);
    assert!(text_panel.frame.y >= 0.0);
}
