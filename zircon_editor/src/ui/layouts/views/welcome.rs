use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use slint::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const WELCOME_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/welcome.ui.toml";
const WELCOME_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";
const WELCOME_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.ui.toml";

pub(crate) fn welcome_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    model_rc(
        build_view_template_nodes(
            "welcome.template_projection",
            WELCOME_LAYOUT_ASSET_PATH,
            &[(WELCOME_STYLE_ASSET_ID, WELCOME_STYLE_ASSET_PATH)],
            size,
            &BTreeMap::new(),
        )
        .unwrap_or_default(),
    )
}
