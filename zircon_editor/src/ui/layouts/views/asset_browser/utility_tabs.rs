use super::ViewTemplateNodeData;

const UTILITY_TAB_FONT_SIZE: f32 = 12.0;
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
    }
}
