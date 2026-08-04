use super::assets_activity_pane_data;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn ultra_narrow_activity_controls_stay_inside_the_projected_root() {
    let pane =
        assets_activity_pane_data(&AssetWorkspaceSnapshot::default(), UiSize::new(20.0, 224.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let root = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityRoot")
        .expect("assets activity root node");

    for control_id in [
        "SearchEdited",
        "OpenAssetBrowser",
        "AssetsActivityPreviewTabButton",
        "AssetsActivityReferencesTabButton",
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing ultra-narrow control `{control_id}`"));
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
