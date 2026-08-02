use crate::ui::layouts::views::hierarchy_pane_nodes;
use crate::ui::workbench::snapshot::{SceneEntries, SceneEntry};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::layout::UiSize;

const HIERARCHY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/hierarchy.zui"
));

#[test]
fn hierarchy_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(HIERARCHY_LAYOUT_TOML);
    assert!(layout.is_ok(), "hierarchy layout should parse");
    let Ok(layout) = layout else {
        return;
    };

    for required_node in [
        "hierarchy_root",
        "content_panel",
        "header_panel",
        "list_panel",
        "empty_state_message",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "hierarchy bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn hierarchy_projection_maps_bootstrap_asset_into_mount_nodes() {
    let scene_entries = SceneEntries::from_entries(
        vec![SceneEntry {
            id: zircon_runtime::scene::NodeId::default(),
            name: "Root".to_string(),
            depth: 0,
        }],
        [zircon_runtime::scene::NodeId::default()],
    );
    let pane = hierarchy_pane_nodes(&scene_entries, UiSize::new(320.0, 640.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    let header = nodes
        .iter()
        .find(|node| node.control_id == "HierarchyHeaderPanel");
    assert!(header.is_some(), "hierarchy header panel node");
    let Some(header) = header else {
        return;
    };
    assert_eq!(header.role.to_string(), "Panel");
    assert_eq!(header.text.to_string(), "Hierarchy");

    let list_panel = nodes
        .iter()
        .find(|node| node.control_id == "HierarchyListPanel");
    assert!(list_panel.is_some(), "hierarchy list panel node");
    let Some(list_panel) = list_panel else {
        return;
    };
    assert_eq!(list_panel.role.to_string(), "Panel");
    assert_eq!(list_panel.text.to_string(), "");
    assert!(list_panel.selected);
    assert!(!list_panel.focused);
    assert_eq!(list_panel.surface_variant.to_string(), "panel");
    assert!(
        list_panel.frame.width > 0.0 && list_panel.frame.height > 0.0,
        "expected hierarchy list panel frame to be laid out by the bootstrap asset"
    );
    assert!(list_panel.frame.x >= 0.0);
    assert!(list_panel.frame.y >= header.frame.y + header.frame.height);
}

#[test]
fn hierarchy_projection_centers_a_muted_empty_state_inside_the_list_surface() {
    let scene_entries = SceneEntries::default();
    let pane = hierarchy_pane_nodes(&scene_entries, UiSize::new(220.0, 320.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    let list_panel = nodes
        .iter()
        .find(|node| node.control_id == "HierarchyListPanel");
    let empty_message = nodes
        .iter()
        .find(|node| node.control_id == "HierarchyEmptyStateMessage");
    assert!(list_panel.is_some(), "hierarchy list panel node");
    assert!(
        empty_message.is_some(),
        "hierarchy empty state message node"
    );
    let (Some(list_panel), Some(empty_message)) = (list_panel, empty_message) else {
        return;
    };

    assert!(!list_panel.selected);
    assert!(!list_panel.focused);
    assert_eq!(list_panel.surface_variant.to_string(), "inset");
    assert_eq!(empty_message.text.to_string(), "No scene nodes");
    assert_eq!(empty_message.text_tone.to_string(), "muted");
    assert!(empty_message.frame.x >= list_panel.frame.x);
    assert!(empty_message.frame.y >= list_panel.frame.y);
    assert!(
        empty_message.frame.x + empty_message.frame.width
            <= list_panel.frame.x + list_panel.frame.width,
        "empty state should stay inside the adaptive list surface"
    );
    assert!(
        empty_message.frame.y + empty_message.frame.height
            <= list_panel.frame.y + list_panel.frame.height,
        "empty state should stay inside the adaptive list surface"
    );
}
