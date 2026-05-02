use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::layouts::windows::workbench_host_window::AssetsActivityPaneViewData;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSETS_ACTIVITY_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/assets_activity.ui.toml";
const ASSETS_ACTIVITY_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";
const ASSETS_ACTIVITY_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.ui.toml";

pub(crate) fn assets_activity_pane_data(size: UiSize) -> AssetsActivityPaneViewData {
    AssetsActivityPaneViewData {
        nodes: model_rc(
            build_view_template_nodes(
                "assets_activity.template_projection",
                ASSETS_ACTIVITY_LAYOUT_ASSET_PATH,
                &[(
                    ASSETS_ACTIVITY_STYLE_ASSET_ID,
                    ASSETS_ACTIVITY_STYLE_ASSET_PATH,
                )],
                size,
                &BTreeMap::new(),
            )
            .unwrap_or_default(),
        ),
    }
}
