use super::assets_activity_pane_data;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

fn projected_nodes(size: UiSize) -> Vec<ViewTemplateNodeData> {
    projected_nodes_for(&AssetWorkspaceSnapshot::default(), size)
}

fn projected_nodes_for(
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) -> Vec<ViewTemplateNodeData> {
    let pane = assets_activity_pane_data(snapshot, size);
    (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect()
}

fn find_node<'a>(nodes: &'a [ViewTemplateNodeData], control_id: &str) -> &'a ViewTemplateNodeData {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .unwrap_or_else(|| panic!("missing assets activity control `{control_id}`"))
}

#[test]
fn ultra_narrow_activity_controls_stay_inside_the_projected_root() {
    let nodes = projected_nodes(UiSize::new(20.0, 224.0));
    let root = find_node(&nodes, "AssetsActivityRoot");

    for control_id in [
        "SearchEdited",
        "OpenAssetBrowser",
        "AssetsActivityKindFilterDropdown",
        "AssetsActivityViewModeListButton",
        "AssetsActivityViewModeThumbButton",
        "AssetsActivityPreviewTabButton",
        "AssetsActivityReferencesTabButton",
    ] {
        let node = find_node(&nodes, control_id);
        assert!(
            node.frame.x >= root.frame.x - f32::EPSILON,
            "{control_id} must not start before the root: {node:?}"
        );
        assert!(
            node.frame.x + node.frame.width <= root.frame.x + root.frame.width + f32::EPSILON,
            "{control_id} must remain inside the root: {node:?}"
        );
    }
}

#[test]
fn narrow_activity_keeps_the_complete_kind_filter_reachable() {
    let nodes = projected_nodes(UiSize::new(420.0, 360.0));
    let root = find_node(&nodes, "AssetsActivityRoot");
    let dropdown = find_node(&nodes, "AssetsActivityKindFilterDropdown");

    assert!(dropdown.frame.width >= 148.0, "kind filter: {dropdown:?}");
    assert!(dropdown.frame.x >= root.frame.x);
    assert!(dropdown.frame.x + dropdown.frame.width <= root.frame.x + root.frame.width);
    assert_eq!(dropdown.options.row_count(), 16);
}

#[test]
fn kind_filter_projection_uses_readable_labels_and_one_selected_stable_id() {
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.kind_filter = Some(ResourceKind::PhysicsMaterial);
    let nodes = projected_nodes_for(&snapshot, UiSize::new(420.0, 360.0));
    let dropdown = find_node(&nodes, "AssetsActivityKindFilterDropdown");
    let options = dropdown
        .options
        .iter()
        .map(|option| option.to_string())
        .collect::<Vec<_>>();

    assert_eq!(dropdown.value_text.as_str(), "Physics Materials");
    assert_eq!(
        options
            .iter()
            .filter(|option| option.ends_with(",selected"))
            .count(),
        1
    );
    assert!(options
        .iter()
        .any(|option| option == "PhysicsMaterial|label=Physics Materials,selected"));
}

#[test]
fn non_toolbar_kind_filter_remains_visible_without_becoming_an_invalid_action() {
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.kind_filter = Some(ResourceKind::Sound);
    let nodes = projected_nodes_for(&snapshot, UiSize::new(420.0, 360.0));
    let dropdown = find_node(&nodes, "AssetsActivityKindFilterDropdown");
    let options = dropdown
        .options
        .iter()
        .map(|option| option.to_string())
        .collect::<Vec<_>>();

    assert_eq!(dropdown.value_text.as_str(), "Sounds");
    assert_eq!(options.len(), 17);
    assert!(options
        .iter()
        .any(|option| option == "Sound|label=Sounds,selected,disabled"));
}

#[test]
fn narrow_activity_collapses_the_tree_before_static_column_minima_overflow() {
    for size in [UiSize::new(640.0, 520.0), UiSize::new(420.0, 360.0)] {
        let nodes = projected_nodes(size);
        let root = find_node(&nodes, "AssetsActivityRoot");
        let tree = find_node(&nodes, "AssetsActivityTreePanel");
        let content = find_node(&nodes, "AssetsActivityContentPanel");

        assert_eq!(tree.frame.width, 0.0, "tree should collapse at {size:?}");
        assert_eq!(tree.frame.height, 0.0, "tree should collapse at {size:?}");
        assert_eq!(content.frame.x, root.frame.x, "content origin at {size:?}");
        assert_eq!(
            content.frame.width, root.frame.width,
            "content width at {size:?}"
        );
    }
}

#[test]
fn regular_activity_preserves_the_folder_tree() {
    let nodes = projected_nodes(UiSize::new(900.0, 620.0));
    let tree = find_node(&nodes, "AssetsActivityTreePanel");
    let content = find_node(&nodes, "AssetsActivityContentPanel");

    assert!(tree.frame.width >= 188.0, "regular tree width: {tree:?}");
    assert!(
        content.frame.width >= 320.0,
        "regular content width: {content:?}"
    );
}
