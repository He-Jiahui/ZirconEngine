use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::build_view_template_node_projection;
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const ANIMATION_EDITOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/animation_editor.zui";
const ANIMATION_EDITOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.zui";
const ANIMATION_EDITOR_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.zui";

pub(crate) fn animation_editor_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    build_view_template_node_projection(
        "animation_editor.template_projection",
        ANIMATION_EDITOR_LAYOUT_ASSET_PATH,
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
