use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use slint::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const HIERARCHY_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/hierarchy.ui.toml";
const HIERARCHY_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";
const HIERARCHY_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.ui.toml";

pub(crate) fn hierarchy_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    model_rc(
        build_view_template_nodes(
            "hierarchy.template_projection",
            HIERARCHY_LAYOUT_ASSET_PATH,
            &[(HIERARCHY_STYLE_ASSET_ID, HIERARCHY_STYLE_ASSET_PATH)],
            size,
            &BTreeMap::new(),
        )
        .unwrap_or_default(),
    )
}
