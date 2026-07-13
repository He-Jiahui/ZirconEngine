use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile,
};
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSETS_ACTIVITY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/assets_activity.zui"
));

#[test]
fn assets_activity_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(ASSETS_ACTIVITY_LAYOUT_TOML)
        .expect("assets activity layout");

    for required_node in [
        "assets_activity_root",
        "toolbar_panel",
        "toolbar_title_row",
        "toolbar_title_text",
        "toolbar_open_browser_button",
        "toolbar_subtitle_row",
        "toolbar_subtitle_text",
        "toolbar_search_row",
        "toolbar_search_field",
        "toolbar_view_mode_list_button",
        "toolbar_view_mode_thumb_button",
        "toolbar_kind_primary_row",
        "toolbar_kind_all_chip",
        "toolbar_kind_texture_chip",
        "toolbar_kind_material_chip",
        "toolbar_kind_scene_chip",
        "toolbar_kind_model_chip",
        "toolbar_kind_shader_chip",
        "toolbar_kind_secondary_row",
        "toolbar_kind_physics_chip",
        "toolbar_kind_skeleton_chip",
        "toolbar_kind_clip_chip",
        "toolbar_kind_sequence_chip",
        "toolbar_kind_graph_chip",
        "toolbar_kind_state_chip",
        "main_panel",
        "tree_panel",
        "tree_header_panel",
        "tree_title_text",
        "tree_subtitle_text",
        "tree_divider",
        "tree_scroll_body",
        "tree_row_panel",
        "tree_row_icon",
        "tree_row_name_text",
        "tree_row_count_text",
        "content_panel",
        "utility_panel",
        "utility_tabs_row",
        "utility_preview_button",
        "utility_references_button",
        "utility_selection_text",
        "utility_tabs_divider",
        "utility_content_panel",
        "preview_panel",
        "reference_left_panel",
        "reference_right_panel",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "assets activity bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn assets_activity_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            utility_tab: AssetUtilityTab::References,
            kind_filter: Some(ResourceKind::Texture),
            selected_folder_id: Some("res://textures".to_string()),
            selected_asset_uuid: Some("33333333-3333-3333-3333-333333333333".to_string()),
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(1280.0, 820.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();

    assert!(
        !nodes.is_empty(),
        "assets activity projection should produce template mount nodes"
    );

    let toolbar = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityToolbarPanel")
        .expect("toolbar panel node");
    assert_eq!(toolbar.role.to_string(), "Mount");
    assert!(toolbar.frame.width > 0.0 && toolbar.frame.height > 0.0);

    let tree = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreePanel")
        .expect("tree panel node");
    let tree_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeTitleText")
        .expect("tree title node");
    let tree_subtitle = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeSubtitleText")
        .expect("tree subtitle node");
    let tree_scroll_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeScrollBody")
        .expect("tree scroll body node");
    let title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTitleText")
        .expect("title node");
    let open_browser = nodes
        .iter()
        .find(|node| node.control_id == "OpenAssetBrowser")
        .expect("open browser button node");
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.control_id == "SearchEdited")
            .count(),
        1,
        "TextField projection should not keep both component and generated text nodes"
    );
    let search = nodes
        .iter()
        .find(|node| node.control_id == "SearchEdited")
        .expect("search field node");
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("content panel node");
    let utility = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityUtilityPanel")
        .expect("utility panel node");
    let utility_selection = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivitySelectionText")
        .expect("utility selection node");
    let utility_divider = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityUtilityDivider")
        .expect("utility divider node");
    let preview_panel = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewPanel")
        .expect("preview panel node");
    let preview_visual = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewVisualPanel")
        .expect("preview visual node");
    let preview_name = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewNameText")
        .expect("preview name node");
    let preview_locator = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewLocatorText")
        .expect("preview locator node");
    let preview_kind = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewKindText")
        .expect("preview kind node");
    let preview_identity = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewIdentityText")
        .expect("preview identity node");
    let preview_adapter = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewToolkitText")
        .expect("preview adapter node");
    let reference_left_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftTitleText")
        .expect("left references title node");
    let reference_left_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftScrollBody")
        .expect("left references body node");
    let reference_left_row = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftRowPanel")
        .expect("left references row node");
    let references_right = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightPanel")
        .expect("right references node");
    let reference_right_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightTitleText")
        .expect("right references title node");
    let reference_right_row = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightRowPanel")
        .expect("right references row node");
    let thumb_mode = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityViewModeThumbButton")
        .expect("thumb mode node");
    let references_tab = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferencesTabButton")
        .expect("references tab node");
    let texture_chip = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityKindTextureChip")
        .expect("texture chip node");

    assert_eq!(title.text.to_string(), "Assets");
    assert_eq!(tree_title.text.to_string(), "Folders");
    assert_eq!(tree_subtitle.text.to_string(), "Browse project assets");
    assert_eq!(tree_scroll_body.role.to_string(), "Panel");
    assert_eq!(tree_scroll_body.surface_variant.to_string(), "scroll-body");
    assert_eq!(open_browser.role.to_string(), "Button");
    assert_eq!(open_browser.dispatch_kind.to_string(), "asset");
    assert_eq!(
        open_browser.binding_id.to_string(),
        "AssetSurface/OpenAssetBrowser"
    );
    assert_eq!(search.role.to_string(), "InputField");
    assert_eq!(search.component_role.to_string(), "input-field");
    assert_eq!(search.text.to_string(), "Search");
    assert_eq!(search.value_text.to_string(), "");
    assert_eq!(search.dispatch_kind.to_string(), "asset");
    assert_eq!(search.binding_id.to_string(), "AssetSurface/SearchEdited");
    assert_eq!(
        search.edit_action_id.to_string(),
        "AssetSurface/SearchEdited"
    );
    assert_eq!(search.commit_action_id.to_string(), "");
    assert!(tree.frame.x >= toolbar.frame.x);
    assert!(content.frame.x >= tree.frame.x + tree.frame.width);
    assert!(utility.frame.y >= tree.frame.y + tree.frame.height);
    assert!(utility_selection.frame.width > 0.0);
    assert!(utility_divider.frame.height > 0.0);
    assert_eq!(preview_panel.role.to_string(), "Panel");
    assert_eq!(preview_panel.surface_variant.to_string(), "asset-preview");
    assert!(!preview_panel.selected);
    assert_eq!(preview_visual.role.to_string(), "Panel");
    assert!(!preview_visual.selected);
    assert_eq!(
        preview_visual.surface_variant.to_string(),
        "asset-preview-visual"
    );
    assert_eq!(preview_name.text.to_string(), "No Asset Selected");
    assert_eq!(preview_locator.text.to_string(), "No project locator");
    assert_eq!(preview_kind.text.to_string(), "Unknown Type");
    assert_eq!(preview_identity.text.to_string(), "No UUID");
    assert_eq!(preview_adapter.text.to_string(), "No toolkit");
    assert_eq!(reference_left_title.text.to_string(), "References");
    assert_eq!(reference_left_body.role.to_string(), "Panel");
    assert_eq!(
        reference_left_body.surface_variant.to_string(),
        "scroll-body"
    );
    assert!(reference_left_row.selected);
    assert_eq!(reference_right_title.text.to_string(), "Used By");
    assert!(reference_right_row.selected);
    assert!(!preview_name.selected);
    assert_eq!(preview_name.text_tone.to_string(), "muted");
    assert!(!preview_locator.selected);
    assert_eq!(preview_locator.text_tone.to_string(), "muted");
    assert!(!preview_kind.selected);
    assert_eq!(preview_kind.text_tone.to_string(), "muted");
    assert!(!preview_identity.selected);
    assert_eq!(preview_identity.text_tone.to_string(), "muted");
    assert!(!preview_adapter.selected);
    assert_eq!(preview_adapter.text_tone.to_string(), "muted");
    assert!(thumb_mode.selected);
    assert_eq!(thumb_mode.surface_variant.to_string(), "inset");
    assert_eq!(thumb_mode.dispatch_kind.to_string(), "asset");
    assert_eq!(
        thumb_mode.action_id.to_string(),
        "workbench.asset.view_mode.set"
    );
    assert_eq!(
        thumb_mode.binding_id.to_string(),
        "AssetSurface/SetViewMode"
    );
    assert_eq!(thumb_mode.value_text.to_string(), "thumbnail");
    assert!(references_tab.selected);
    assert_eq!(references_tab.surface_variant.to_string(), "inset");
    assert_eq!(references_tab.dispatch_kind.to_string(), "asset");
    assert_eq!(
        references_tab.action_id.to_string(),
        "workbench.asset.utility_tab.set"
    );
    assert_eq!(
        references_tab.binding_id.to_string(),
        "AssetSurface/SetUtilityTab"
    );
    assert_eq!(references_tab.value_text.to_string(), "references");
    assert!(texture_chip.selected);
    assert_eq!(texture_chip.surface_variant.to_string(), "inset");
    assert_eq!(texture_chip.dispatch_kind.to_string(), "asset");
    assert_eq!(
        texture_chip.action_id.to_string(),
        "workbench.asset.kind_filter.set"
    );
    assert_eq!(
        texture_chip.binding_id.to_string(),
        "AssetSurface/SetKindFilter"
    );
    assert_eq!(texture_chip.value_text.to_string(), "Texture");
    assert!(references_right.frame.width > 0.0 && references_right.frame.height > 0.0);
}

#[test]
fn assets_activity_regular_drawer_compacts_toolbar_and_reclaims_content_width() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            utility_tab: AssetUtilityTab::Preview,
            kind_filter: Some(ResourceKind::Texture),
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(226.0, 346.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing compact Assets activity node `{control_id}`"))
    };

    let toolbar = node("AssetsActivityToolbarPanel");
    let browser = node("OpenAssetBrowser");
    let search = node("SearchEdited");
    let list = node("AssetsActivityViewModeListButton");
    let thumb = node("AssetsActivityViewModeThumbButton");
    let all = node("AssetsActivityKindAllChip");
    let texture = node("AssetsActivityKindTextureChip");
    let material = node("AssetsActivityKindMaterialChip");
    let tree = node("AssetsActivityTreePanel");
    let content = node("AssetsActivityContentPanel");
    let main = node("AssetsActivityMainPanel");
    let selection = node("AssetsActivitySelectionText");
    let preview = node("AssetsActivityPreviewTabButton");
    let references = node("AssetsActivityReferencesTabButton");

    assert_eq!(thumb.text.to_string(), "Thumb");
    assert_eq!(texture.text.to_string(), "Tex");
    assert_eq!(preview.text.to_string(), "Preview");
    assert!(preview.selected);
    assert!(toolbar.frame.height <= 68.0);
    for control in [
        browser, search, list, thumb, all, texture, preview, references,
    ] {
        assert!(
            control.frame.width > 0.0,
            "compact control should remain visible: {control:?}"
        );
        assert!(
            control.frame.x + control.frame.width <= 226.0 + f32::EPSILON,
            "compact control should stay inside the drawer: {control:?}"
        );
    }
    assert_eq!(material.frame.width, 0.0);
    assert_eq!(tree.frame.width, 0.0);
    assert!(content.frame.width >= 210.0);
    assert!(main.frame.height >= 120.0);
    assert_eq!(selection.frame.width, 0.0);
}

#[test]
fn assets_activity_regular_drawer_references_use_one_readable_summary_column() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            utility_tab: AssetUtilityTab::References,
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(226.0, 346.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing compact reference node `{control_id}`"))
    };
    let left = frame("AssetsActivityReferenceLeftPanel");
    let right = frame("AssetsActivityReferenceRightPanel");
    let summary = frame("AssetsActivityReferenceLeftEmptyText");

    assert!(left.width >= 210.0);
    assert_eq!(right.width, 0.0);
    assert!(summary.width > 0.0);
    assert!(summary.x + summary.width <= 226.0 + f32::EPSILON);
}

#[test]
fn assets_activity_content_rows_share_the_activity_pointer_geometry() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        selected_asset_uuid: Some("asset-selected".to_string()),
        visible_assets: vec![
            activity_asset(
                "asset-first",
                "editor_base.zui",
                ResourceKind::UiStyle,
                false,
            ),
            activity_asset(
                "asset-selected",
                "workbench_page_chrome.zui",
                ResourceKind::UiLayout,
                true,
            ),
        ],
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 346.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing Assets activity content node `{control_id}`"))
    };
    let content = node("AssetsActivityContentPanel");
    let first = node("AssetsActivityContentItemRow00");
    let selected = node("AssetsActivityContentItemRow01");
    let selected_name = node("AssetsActivityContentItemName01");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );

    assert_eq!(first.frame.x, content.frame.x + metrics.row_x);
    assert_eq!(first.frame.y, content.frame.y + metrics.first_row_y());
    assert_eq!(first.frame.width, metrics.row_width(content.frame.width));
    assert_eq!(first.frame.height, metrics.item_height);
    assert_eq!(
        selected.frame.y,
        first.frame.y + metrics.item_height + metrics.row_gap
    );
    assert!(selected.selected);
    assert!(!selected_name.selected);
    assert_eq!(selected_name.border_width, 0.0);
    assert!(selected_name.text.to_string().ends_with(".zui"));
    assert!(selected_name.text.to_string().contains("..."));
    assert!(selected_name.frame.x >= selected.frame.x);
    assert!(
        selected_name.frame.x + selected_name.frame.width
            <= selected.frame.x + selected.frame.width
    );
    assert!(!nodes
        .iter()
        .any(|node| node.control_id == "AssetsActivityContentEmptyText"));
}

#[test]
fn empty_assets_activity_content_has_an_explicit_readable_state() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot::default(),
        UiSize::new(226.0, 346.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("content panel");
    let empty = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentEmptyText")
        .expect("explicit empty content state");

    assert_eq!(empty.text.to_string(), "No assets in this folder");
    assert_eq!(empty.text_tone.to_string(), "muted");
    assert!(empty.frame.width > 0.0);
    assert!(empty.frame.x + empty.frame.width <= content.frame.x + content.frame.width);
}

#[test]
fn short_assets_activity_drawer_preserves_one_complete_asset_row_before_preview() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: vec![activity_asset(
            "asset-visible",
            "workbench_page_chrome.zui",
            ResourceKind::UiLayout,
            true,
        )],
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 224.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing short drawer node `{control_id}`"))
    };
    let content = frame("AssetsActivityContentPanel");
    let row = frame("AssetsActivityContentItemRow00");
    let utility = frame("AssetsActivityUtilityPanel");
    let preview = frame("AssetsActivityPreviewPanel");

    assert!(row.width > 0.0 && row.height > 0.0);
    assert!(row.y + row.height <= content.y + content.height + f32::EPSILON);
    assert!(content.y + content.height <= utility.y + f32::EPSILON);
    assert!(preview.width > 0.0 && preview.height > 0.0);
}

#[test]
fn short_assets_activity_keeps_below_viewport_rows_in_scroll_source_geometry() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: (0..5)
            .map(|index| {
                activity_asset(
                    &format!("asset-{index}"),
                    &format!("workbench_asset_{index}.zui"),
                    ResourceKind::UiLayout,
                    false,
                )
            })
            .collect(),
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 224.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing short scroll-source node `{control_id}`"))
    };
    let content_node = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("short content panel");
    let content = content_node.frame.clone();
    let late_row = frame("AssetsActivityContentItemRow04");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );

    assert!(late_row.width > 0.0 && late_row.height > 0.0);
    assert!(late_row.y >= content.y + content.height);
    assert_eq!(content_node.value_number, metrics.list_height(0, 5));
}

fn activity_asset(
    uuid: &str,
    display_name: &str,
    kind: ResourceKind,
    selected: bool,
) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: format!("res://ui/{display_name}"),
        display_name: display_name.to_string(),
        file_name: display_name.to_string(),
        extension: display_name
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or_default()
            .to_string(),
        kind,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            kind,
        ),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: None,
        resource_revision: Some(1),
    }
}
