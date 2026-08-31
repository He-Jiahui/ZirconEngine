use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::{
    build_view_template_node_projection_with_patches, ViewTemplateNodePatch,
};
use crate::ui::retained_host::hierarchy_pointer::hierarchy_paint_metadata;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::SceneEntries;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const HIERARCHY_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/hierarchy.zui";
const HIERARCHY_HEADER_PANEL: &str = "HierarchyHeaderPanel";
const HIERARCHY_LIST_PANEL: &str = "HierarchyListPanel";
const HIERARCHY_EMPTY_STATE_MESSAGE: &str = "HierarchyEmptyStateMessage";

pub(crate) fn hierarchy_pane_nodes(
    entries: &SceneEntries,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        HIERARCHY_EMPTY_STATE_MESSAGE.to_string(),
        if entries.is_empty() {
            "No scene nodes".to_string()
        } else {
            String::new()
        },
    );

    let node_patches = hierarchy_visual_state_patches();
    let Ok(projection) = build_view_template_node_projection_with_patches(
        "hierarchy.template_projection",
        HIERARCHY_LAYOUT_ASSET_PATH,
        &[],
        size,
        &text_overrides,
        &node_patches,
    ) else {
        return ModelRc::default();
    };
    let metadata = hierarchy_paint_metadata(projection.iter().map(|node| node.control_id.as_str()));
    projection.into_model().replacing_metadata(metadata)
}

fn hierarchy_visual_state_patches() -> BTreeMap<String, ViewTemplateNodePatch> {
    BTreeMap::from([
        (
            HIERARCHY_LIST_PANEL.to_string(),
            ViewTemplateNodePatch::visual_state(false, false, "transparent", "muted"),
        ),
        (
            HIERARCHY_HEADER_PANEL.to_string(),
            ViewTemplateNodePatch::default()
                .focused(false)
                .surface_variant("transparent")
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
