use std::collections::BTreeMap;

use crate::ui::asset_editor::{
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
};
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes_with_imports;
use slint::ModelRc;
use zircon_runtime::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const UI_ASSET_EDITOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/ui_asset_editor.ui.toml";
const UI_ASSET_EDITOR_WIDGET_ASSET_PATH: &str = "/assets/ui/editor/editor_widgets.ui.toml";
const UI_ASSET_EDITOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";

pub(crate) fn ui_asset_editor_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    model_rc(
        build_view_template_nodes_with_imports(
            "ui_asset_editor.template_projection",
            UI_ASSET_EDITOR_LAYOUT_ASSET_PATH,
            &[
                (
                    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
                    UI_ASSET_EDITOR_WIDGET_ASSET_PATH,
                ),
                (
                    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
                    UI_ASSET_EDITOR_WIDGET_ASSET_PATH,
                ),
            ],
            &[(
                UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
                UI_ASSET_EDITOR_STYLE_ASSET_PATH,
            )],
            size,
            &BTreeMap::new(),
        )
        .unwrap_or_default(),
    )
}
