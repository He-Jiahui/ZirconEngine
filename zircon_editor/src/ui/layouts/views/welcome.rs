use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes_with_imports;
use slint::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const WELCOME_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/welcome.ui.toml";
const WELCOME_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";
const WELCOME_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.ui.toml";
const WELCOME_MATERIAL_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_material.ui.toml";
const WELCOME_MATERIAL_STYLE_ASSET_ID: &str = "res://ui/theme/editor_material.ui.toml";
const WELCOME_MATERIAL_COMPONENT_ASSET_PATH: &str =
    "/assets/ui/editor/material_meta_components.ui.toml";
const MATERIAL_OUTLINED_FIELD_ID: &str =
    "res://ui/editor/material_meta_components.ui.toml#MaterialOutlinedField";
const MATERIAL_BUTTON_ID: &str = "res://ui/editor/material_meta_components.ui.toml#MaterialButton";
const MATERIAL_TEXT_BUTTON_ID: &str =
    "res://ui/editor/material_meta_components.ui.toml#MaterialTextButton";
const MATERIAL_LIST_ITEM_ID: &str =
    "res://ui/editor/material_meta_components.ui.toml#MaterialListItem";

pub(crate) fn welcome_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    model_rc(
        build_view_template_nodes_with_imports(
            "welcome.template_projection",
            WELCOME_LAYOUT_ASSET_PATH,
            &[
                (
                    MATERIAL_OUTLINED_FIELD_ID,
                    WELCOME_MATERIAL_COMPONENT_ASSET_PATH,
                ),
                (MATERIAL_BUTTON_ID, WELCOME_MATERIAL_COMPONENT_ASSET_PATH),
                (
                    MATERIAL_TEXT_BUTTON_ID,
                    WELCOME_MATERIAL_COMPONENT_ASSET_PATH,
                ),
                (MATERIAL_LIST_ITEM_ID, WELCOME_MATERIAL_COMPONENT_ASSET_PATH),
            ],
            &[
                (WELCOME_STYLE_ASSET_ID, WELCOME_STYLE_ASSET_PATH),
                (
                    WELCOME_MATERIAL_STYLE_ASSET_ID,
                    WELCOME_MATERIAL_STYLE_ASSET_PATH,
                ),
            ],
            size,
            &BTreeMap::new(),
        )
        .unwrap_or_default(),
    )
}
