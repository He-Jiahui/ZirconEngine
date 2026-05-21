use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::layouts::windows::workbench_host_window::AssetsActivityPaneViewData;
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::ui::layout::UiSize;

const ASSETS_ACTIVITY_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/assets_activity.v2.ui.toml";
const ASSETS_ACTIVITY_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.v2.ui.toml";
const ASSETS_ACTIVITY_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.v2.ui.toml";

pub(crate) fn assets_activity_pane_data(
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) -> AssetsActivityPaneViewData {
    let mut text_overrides = BTreeMap::new();
    let project_root = if snapshot.project_root.is_empty() {
        "Browse project assets".to_string()
    } else {
        snapshot.project_root.clone()
    };
    let selection_text = if snapshot.selection.display_name.is_empty() {
        "Select an asset to inspect".to_string()
    } else {
        snapshot.selection.display_name.clone()
    };
    let kind_filter = snapshot.kind_filter;
    let selection_locator = if snapshot.selection.locator.is_empty() {
        "No project locator".to_string()
    } else {
        snapshot.selection.locator.clone()
    };
    let selection_kind = snapshot
        .selection
        .kind
        .map(resource_kind_label)
        .unwrap_or("Unknown Type")
        .to_string();
    let selection_identity = snapshot
        .selection
        .uuid
        .clone()
        .unwrap_or_else(|| "No UUID".to_string());
    let selection_adapter = if snapshot.selection.adapter_key.is_empty() {
        "No adapter".to_string()
    } else {
        snapshot.selection.adapter_key.clone()
    };
    let selection_meta_path = if snapshot.selection.meta_path.is_empty() {
        "No meta path".to_string()
    } else {
        snapshot.selection.meta_path.clone()
    };
    let selection_diagnostics = if snapshot.selection.diagnostics.is_empty() {
        "No active diagnostics".to_string()
    } else {
        snapshot.selection.diagnostics.join("\n")
    };

    text_overrides.insert(
        "AssetsActivitySubtitleText".to_string(),
        project_root.clone(),
    );
    text_overrides.insert(
        "AssetsActivityTreeSubtitleText".to_string(),
        project_root.clone(),
    );
    text_overrides.insert(
        "AssetsActivityTreeRowNameText".to_string(),
        snapshot
            .selected_folder_id
            .clone()
            .unwrap_or_else(|| "Assets".to_string()),
    );
    text_overrides.insert(
        "AssetsActivityTreeRowCountText".to_string(),
        snapshot.visible_assets.len().to_string(),
    );
    text_overrides.insert("AssetsActivitySelectionText".to_string(), selection_text);
    text_overrides.insert(
        "AssetsActivityPreviewNameText".to_string(),
        if snapshot.selection.display_name.is_empty() {
            "No Asset Selected".to_string()
        } else {
            snapshot.selection.display_name.clone()
        },
    );
    text_overrides.insert(
        "AssetsActivityPreviewLocatorText".to_string(),
        selection_locator,
    );
    text_overrides.insert("AssetsActivityPreviewKindText".to_string(), selection_kind);
    text_overrides.insert(
        "AssetsActivityPreviewIdentityText".to_string(),
        selection_identity,
    );
    text_overrides.insert(
        "AssetsActivityPreviewAdapterText".to_string(),
        selection_adapter,
    );
    text_overrides.insert(
        "AssetsActivityPreviewMetaPathText".to_string(),
        selection_meta_path,
    );
    text_overrides.insert(
        "AssetsActivityPreviewDiagnosticsText".to_string(),
        selection_diagnostics,
    );
    text_overrides.insert(
        "AssetsActivityReferenceLeftEmptyText".to_string(),
        if snapshot.selection.references.is_empty() {
            "No direct references".to_string()
        } else {
            format!("{} direct references", snapshot.selection.references.len())
        },
    );
    text_overrides.insert(
        "AssetsActivityReferenceRightEmptyText".to_string(),
        if snapshot.selection.used_by.is_empty() {
            "No direct references".to_string()
        } else {
            format!("{} usages", snapshot.selection.used_by.len())
        },
    );
    text_overrides.insert(
        "AssetsActivityViewModeListButton".to_string(),
        if snapshot.view_mode == AssetViewMode::List {
            "List • Active".to_string()
        } else {
            "List".to_string()
        },
    );
    text_overrides.insert(
        "AssetsActivityViewModeThumbButton".to_string(),
        if snapshot.view_mode == AssetViewMode::Thumbnail {
            "Thumb • Active".to_string()
        } else {
            "Thumb".to_string()
        },
    );
    text_overrides.insert(
        "AssetsActivityKindAllChip".to_string(),
        if kind_filter.is_none() {
            "All • Active".to_string()
        } else {
            "All".to_string()
        },
    );
    text_overrides.insert(
        "AssetsActivityKindTextureChip".to_string(),
        kind_filter_label(
            kind_filter,
            "Tex",
            zircon_runtime_interface::resource::ResourceKind::Texture,
        ),
    );
    text_overrides.insert(
        "AssetsActivityKindMaterialChip".to_string(),
        kind_filter_label(
            kind_filter,
            "Mat",
            zircon_runtime_interface::resource::ResourceKind::Material,
        ),
    );
    text_overrides.insert(
        "AssetsActivityKindSceneChip".to_string(),
        kind_filter_label(
            kind_filter,
            "Scn",
            zircon_runtime_interface::resource::ResourceKind::Scene,
        ),
    );
    text_overrides.insert(
        "AssetsActivityKindModelChip".to_string(),
        kind_filter_label(
            kind_filter,
            "Mesh",
            zircon_runtime_interface::resource::ResourceKind::Model,
        ),
    );
    text_overrides.insert(
        "AssetsActivityKindShaderChip".to_string(),
        kind_filter_label(
            kind_filter,
            "Shd",
            zircon_runtime_interface::resource::ResourceKind::Shader,
        ),
    );
    text_overrides.insert(
        "AssetsActivityPreviewButton".to_string(),
        utility_tab_label(snapshot.utility_tab, AssetUtilityTab::Preview, "Preview"),
    );
    text_overrides.insert(
        "AssetsActivityReferencesTabButton".to_string(),
        utility_tab_label(
            snapshot.utility_tab,
            AssetUtilityTab::References,
            "References",
        ),
    );
    text_overrides.insert(
        "SearchEdited".to_string(),
        if snapshot.search_query.is_empty() {
            "Search".to_string()
        } else {
            snapshot.search_query.clone()
        },
    );

    let mut nodes = build_view_template_nodes(
        "assets_activity.template_projection",
        ASSETS_ACTIVITY_LAYOUT_ASSET_PATH,
        &[(
            ASSETS_ACTIVITY_STYLE_ASSET_ID,
            ASSETS_ACTIVITY_STYLE_ASSET_PATH,
        )],
        size,
        &text_overrides,
    )
    .unwrap_or_default();
    apply_assets_activity_visual_state(&mut nodes, snapshot);

    AssetsActivityPaneViewData {
        nodes: model_rc(nodes),
    }
}

fn utility_tab_label(active: AssetUtilityTab, expected: AssetUtilityTab, label: &str) -> String {
    if active == expected {
        format!("{label} • Active")
    } else {
        label.to_string()
    }
}

fn kind_filter_label(
    active: Option<zircon_runtime_interface::resource::ResourceKind>,
    label: &str,
    expected: zircon_runtime_interface::resource::ResourceKind,
) -> String {
    if active == Some(expected) {
        format!("{label} • Active")
    } else {
        label.to_string()
    }
}

fn resource_kind_label(kind: zircon_runtime_interface::resource::ResourceKind) -> &'static str {
    use zircon_runtime_interface::resource::ResourceKind;
    match kind {
        ResourceKind::Texture => "Texture",
        ResourceKind::Material => "Material",
        ResourceKind::Scene => "Scene",
        ResourceKind::Model => "Model",
        ResourceKind::Mesh => "Mesh",
        ResourceKind::Shader => "Shader",
        ResourceKind::Sound => "Sound",
        ResourceKind::Font => "Font",
        ResourceKind::PhysicsMaterial => "PhysicsMaterial",
        ResourceKind::NavMesh => "NavMesh",
        ResourceKind::NavigationSettings => "NavigationSettings",
        ResourceKind::Terrain => "Terrain",
        ResourceKind::TerrainLayerStack => "TerrainLayerStack",
        ResourceKind::TileSet => "TileSet",
        ResourceKind::TileMap => "TileMap",
        ResourceKind::Prefab => "Prefab",
        ResourceKind::AnimationSkeleton => "AnimationSkeleton",
        ResourceKind::AnimationClip => "AnimationClip",
        ResourceKind::AnimationSequence => "AnimationSequence",
        ResourceKind::AnimationGraph => "AnimationGraph",
        ResourceKind::AnimationStateMachine => "AnimationStateMachine",
        ResourceKind::UiLayout => "UiLayout",
        ResourceKind::UiWidget => "UiWidget",
        ResourceKind::UiStyle => "UiStyle",
        ResourceKind::Data => "Data",
        ResourceKind::MaterialGraph => "MaterialGraph",
    }
}

fn apply_assets_activity_visual_state(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    mark_toggle_state(
        nodes,
        "AssetsActivityViewModeListButton",
        snapshot.view_mode == AssetViewMode::List,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityViewModeThumbButton",
        snapshot.view_mode == AssetViewMode::Thumbnail,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityPreviewButton",
        snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityReferencesTabButton",
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindAllChip",
        snapshot.kind_filter.is_none(),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindTextureChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Texture),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindMaterialChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Material),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindSceneChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Scene),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindModelChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Model),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindShaderChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Shader),
    );
    let has_selection =
        snapshot.selected_asset_uuid.is_some() || !snapshot.selection.display_name.is_empty();
    mark_panel_selected(
        nodes,
        "AssetsActivityTreeRowPanel",
        snapshot.selected_folder_id.is_some(),
    );
    mark_panel_selected(
        nodes,
        "AssetsActivityPreviewPanel",
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_panel_selected(
        nodes,
        "AssetsActivityPreviewVisualPanel",
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_panel_group_selected(
        nodes,
        &[
            "AssetsActivityReferenceLeftPanel",
            "AssetsActivityReferenceRightPanel",
            "AssetsActivityReferenceLeftRowPanel",
            "AssetsActivityReferenceRightRowPanel",
        ],
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_text_state(
        nodes,
        &[
            "AssetsActivityPreviewNameText",
            "AssetsActivityPreviewLocatorText",
            "AssetsActivityPreviewKindText",
            "AssetsActivityPreviewIdentityText",
            "AssetsActivityPreviewAdapterText",
            "AssetsActivityPreviewMetaPathText",
            "AssetsActivityPreviewDiagnosticsText",
        ],
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
}

fn mark_toggle_state(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_id: &str,
    active: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = active;
        node.surface_variant = if active { "inset".into() } else { "".into() };
        node.text_tone = if active {
            "default".into()
        } else {
            "subtle".into()
        };
    }
}

fn mark_panel_selected(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_id: &str,
    selected: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = selected;
        node.focused = selected;
    }
}

fn mark_panel_group_selected(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_ids: &[&str],
    selected: bool,
) {
    for control_id in control_ids {
        mark_panel_selected(nodes, control_id, selected);
    }
}

fn mark_text_state(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_ids: &[&str],
    active: bool,
) {
    for control_id in control_ids {
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == *control_id) {
            node.selected = active;
            node.focused = active;
            node.text_tone = if active {
                "default".into()
            } else {
                "muted".into()
            };
        }
    }
}
