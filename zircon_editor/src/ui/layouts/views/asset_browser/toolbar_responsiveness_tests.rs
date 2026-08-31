use super::*;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn narrow_toolbar_keeps_search_and_one_kind_filter_trigger_reachable() {
    let nodes = asset_browser_pane_nodes(
        &AssetWorkspaceSnapshot::default(),
        UiSize::new(640.0, 520.0),
    );
    let search = find_node(&nodes, "SearchEdited");
    let filter = find_node(&nodes, "AssetBrowserKindFilterDropdown");
    let import = find_node(&nodes, "ImportModel");

    assert!(search.frame.width >= 160.0 && search.frame.height > 0.0);
    assert!(filter.frame.width >= 124.0 && filter.frame.height > 0.0);
    assert!(import.frame.width > 0.0 && import.frame.height > 0.0);
    assert!(search.frame.x + search.frame.width < filter.frame.x);
    assert!(filter.frame.x + filter.frame.width < import.frame.x);
    assert!(all_legacy_kind_chips_absent(&nodes));
}

#[test]
fn unsupported_current_kind_remains_visible_selected_and_disabled() {
    let nodes = asset_browser_pane_nodes(
        &AssetWorkspaceSnapshot {
            kind_filter: Some(ResourceKind::Sound),
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(640.0, 520.0),
    );
    let filter = find_node(&nodes, "AssetBrowserKindFilterDropdown");
    let options = (0..filter.options.row_count())
        .filter_map(|index| filter.options.row_data(index))
        .map(|option| option.to_string())
        .collect::<Vec<_>>();

    assert_eq!(filter.value_text.as_str(), "Sounds");
    assert_eq!(options.len(), 17);
    assert_eq!(
        options.last().map(String::as_str),
        Some("Sound|label=Sounds,selected,disabled")
    );
}

#[test]
fn ultra_narrow_toolbar_controls_are_hidden_or_contained_by_the_toolbar() {
    let nodes =
        asset_browser_pane_nodes(&AssetWorkspaceSnapshot::default(), UiSize::new(20.0, 224.0));
    let toolbar = find_node(&nodes, "AssetBrowserToolbarPanel");

    for control_id in [
        "SearchEdited",
        "AssetBrowserKindFilterDropdown",
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

fn all_legacy_kind_chips_absent(nodes: &ModelRc<ViewTemplateNodeData>) -> bool {
    (0..nodes.row_count()).all(|index| {
        nodes.row_data(index).is_none_or(|node| {
            let control_id = node.control_id.as_str();
            !control_id.starts_with("AssetBrowserKind") || !control_id.ends_with("Chip")
        })
    })
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
