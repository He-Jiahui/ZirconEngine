use super::*;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn ultra_narrow_toolbar_controls_are_hidden_or_contained_by_the_toolbar() {
    let nodes =
        asset_browser_pane_nodes(&AssetWorkspaceSnapshot::default(), UiSize::new(20.0, 224.0));
    let toolbar = find_node(&nodes, "AssetBrowserToolbarPanel");

    for control_id in [
        "SearchEdited",
        "AssetBrowserKindAllChip",
        "AssetBrowserKindTextureChip",
        "AssetBrowserKindMaterialChip",
        "AssetBrowserKindSceneChip",
        "AssetBrowserKindModelChip",
        "AssetBrowserKindShaderChip",
        "AssetBrowserViewModeListButton",
        "AssetBrowserViewModeThumbButton",
        "LocateSelectedAsset",
        "AssetBrowserImportPathField",
        "ImportModel",
    ] {
        let control = find_node(&nodes, control_id);
        if control.frame.width == 0.0 || control.frame.height == 0.0 {
            continue;
        }

        assert!(
            control.frame.x >= toolbar.frame.x
                && control.frame.x + control.frame.width <= toolbar.frame.x + toolbar.frame.width,
            "visible {control_id} must remain inside the toolbar: control={:?}, toolbar={:?}",
            control.frame,
            toolbar.frame
        );
    }
}

fn find_node(nodes: &ModelRc<ViewTemplateNodeData>, control_id: &str) -> ViewTemplateNodeData {
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        if node.control_id.as_str() == control_id {
            return node;
        }
    }
    panic!("missing node {control_id}");
}
