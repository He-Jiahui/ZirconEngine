use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::build_view_template_node_projection;
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const ANIMATION_SEQUENCE_LAYOUT_ASSET_PATH: &str =
    "/assets/ui/editor/host/animation_sequence_body.zui";
const ANIMATION_GRAPH_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/host/animation_graph_body.zui";
const ANIMATION_EDITOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.zui";
const ANIMATION_EDITOR_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.zui";

fn build_animation_pane_nodes(
    projection_id: &str,
    layout_asset_path: &str,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    build_view_template_node_projection(
        projection_id,
        layout_asset_path,
        &[(
            ANIMATION_EDITOR_STYLE_ASSET_ID,
            ANIMATION_EDITOR_STYLE_ASSET_PATH,
        )],
        size,
        &BTreeMap::new(),
    )
    .map(|projection| projection.into_model())
    .unwrap_or_default()
}

pub(crate) fn animation_sequence_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    build_animation_pane_nodes(
        "animation_sequence_editor.template_projection",
        ANIMATION_SEQUENCE_LAYOUT_ASSET_PATH,
        size,
    )
}

pub(crate) fn animation_graph_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    build_animation_pane_nodes(
        "animation_graph_editor.template_projection",
        ANIMATION_GRAPH_LAYOUT_ASSET_PATH,
        size,
    )
}
