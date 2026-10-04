use super::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

const UTILITY_TAB_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;
const UTILITY_TAB_SELECTED_FONT_WEIGHT: i32 = 600;
const UTILITY_TAB_IDLE_FONT_WEIGHT: i32 = 400;
const UTILITY_TAB_IDS: &[&str] = &[
    "AssetBrowserPreviewTabButton",
    "AssetBrowserReferencesTabButton",
    "AssetBrowserMetadataTabButton",
    "AssetBrowserPluginsTabButton",
];

pub(super) fn apply_asset_browser_utility_tab_typography(nodes: &mut [ViewTemplateNodeData]) {
    for node in nodes
        .iter_mut()
        .filter(|node| UTILITY_TAB_IDS.contains(&node.control_id.as_str()))
    {
        node.font_size = UTILITY_TAB_FONT_SIZE;
        node.font_weight = if node.selected {
            UTILITY_TAB_SELECTED_FONT_WEIGHT
        } else {
            UTILITY_TAB_IDLE_FONT_WEIGHT
        };
        assign_shared_string_if_changed(&mut node.overflow, "elide");
    }
}

fn assign_shared_string_if_changed(target: &mut SharedString, value: &str) {
    if target.as_str() != value {
        *target = value.into();
    }
}

#[cfg(test)]
#[path = "tests/utility_tabs.rs"]
mod tests;
