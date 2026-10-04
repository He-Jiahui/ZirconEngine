use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::{
    build_view_template_node_projection_with_patches, ViewTemplateNodePatch,
};
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const CONSOLE_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/console.zui";

pub(crate) fn console_pane_nodes(status_text: &str, size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        "ConsoleTextPanel".to_string(),
        if status_text.is_empty() {
            "Console ready".to_string()
        } else {
            status_text.to_string()
        },
    );
    let node_patches = console_visual_state_patches(!status_text.is_empty());
    let Ok(projection) = build_view_template_node_projection_with_patches(
        "console.template_projection",
        CONSOLE_LAYOUT_ASSET_PATH,
        &[],
        size,
        &text_overrides,
        &node_patches,
    ) else {
        return ModelRc::default();
    };
    projection.into_model()
}

fn console_visual_state_patches(has_status: bool) -> BTreeMap<String, ViewTemplateNodePatch> {
    BTreeMap::from([
        (
            "ConsoleHeader".to_string(),
            ViewTemplateNodePatch::visual_state(false, false, "transparent", "default"),
        ),
        (
            "ConsoleBodySection".to_string(),
            ViewTemplateNodePatch::visual_state(false, false, "transparent", "muted"),
        ),
        (
            "ConsoleTextPanel".to_string(),
            ViewTemplateNodePatch::visual_state(
                false,
                false,
                "transparent",
                if has_status { "default" } else { "muted" },
            ),
        ),
    ])
}

#[cfg(test)]
#[path = "tests/console.rs"]
mod tests;
