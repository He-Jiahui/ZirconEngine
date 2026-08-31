use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::build_view_template_node_projection;
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const WELCOME_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/welcome.zui";

pub(crate) fn welcome_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    build_view_template_node_projection(
        "welcome.template_projection",
        WELCOME_LAYOUT_ASSET_PATH,
        &[],
        size,
        &BTreeMap::new(),
    )
    .map(|projection| projection.into_model())
    .unwrap_or_default()
}
