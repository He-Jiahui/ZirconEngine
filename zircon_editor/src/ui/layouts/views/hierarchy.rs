use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::{
    build_view_template_node_projection_with_patches, ViewTemplateNodePatch,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::SceneEntries;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const HIERARCHY_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/hierarchy.zui";
const HIERARCHY_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.zui";
const HIERARCHY_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.zui";
const HIERARCHY_HEADER_PANEL: &str = "HierarchyHeaderPanel";
const HIERARCHY_LIST_PANEL: &str = "HierarchyListPanel";
const HIERARCHY_EMPTY_STATE_MESSAGE: &str = "HierarchyEmptyStateMessage";

pub(crate) fn hierarchy_pane_nodes(
    entries: &SceneEntries,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    let active_entry = entries
        .iter()
        .find(|entry| entries.is_selected(entry.entity));
    text_overrides.insert(HIERARCHY_HEADER_PANEL.to_string(), "Hierarchy".to_string());
    text_overrides.insert(
        HIERARCHY_EMPTY_STATE_MESSAGE.to_string(),
        if entries.is_empty() {
            "No scene nodes".to_string()
        } else {
            String::new()
        },
    );

    let node_patches = hierarchy_visual_state_patches(active_entry.is_some());
    let Ok(projection) = build_view_template_node_projection_with_patches(
        "hierarchy.template_projection",
        HIERARCHY_LAYOUT_ASSET_PATH,
        &[(HIERARCHY_STYLE_ASSET_ID, HIERARCHY_STYLE_ASSET_PATH)],
        size,
        &text_overrides,
        &node_patches,
    ) else {
        return ModelRc::default();
    };
    projection.into_model()
}

fn hierarchy_visual_state_patches(has_selection: bool) -> BTreeMap<String, ViewTemplateNodePatch> {
    BTreeMap::from([
        (
            HIERARCHY_LIST_PANEL.to_string(),
            ViewTemplateNodePatch::visual_state(
                has_selection,
                false,
                if has_selection { "panel" } else { "inset" },
                if has_selection { "default" } else { "muted" },
            ),
        ),
        (
            HIERARCHY_HEADER_PANEL.to_string(),
            ViewTemplateNodePatch::default()
                .focused(false)
                .surface_variant("inset")
                .text_tone("default"),
        ),
        (
            HIERARCHY_EMPTY_STATE_MESSAGE.to_string(),
            ViewTemplateNodePatch::default()
                .focused(false)
                .text_tone("muted"),
        ),
    ])
}
