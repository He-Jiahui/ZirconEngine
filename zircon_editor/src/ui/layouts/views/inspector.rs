use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use slint::ModelRc;
use zircon_runtime::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const INSPECTOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/inspector.ui.toml";
const INSPECTOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";
const INSPECTOR_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.ui.toml";

pub(crate) fn inspector_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    model_rc(
        build_view_template_nodes(
            "inspector.template_projection",
            INSPECTOR_LAYOUT_ASSET_PATH,
            &[(INSPECTOR_STYLE_ASSET_ID, INSPECTOR_STYLE_ASSET_PATH)],
            size,
            &BTreeMap::new(),
        )
        .unwrap_or_default(),
    )
}
