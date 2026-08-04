use super::ViewTemplateNodeData;
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
        node.overflow = "elide".into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_tabs_share_workbench_body_typography_and_elide_overflow() {
        let mut nodes = vec![
            ViewTemplateNodeData {
                control_id: "AssetBrowserPreviewTabButton".into(),
                selected: true,
                ..ViewTemplateNodeData::default()
            },
            ViewTemplateNodeData {
                control_id: "AssetBrowserMetadataTabButton".into(),
                ..ViewTemplateNodeData::default()
            },
        ];

        apply_asset_browser_utility_tab_typography(&mut nodes);

        assert_eq!(
            nodes[0].font_size,
            EditorTypographyTokens::WORKBENCH_BODY_SIZE
        );
        assert_eq!(nodes[0].font_weight, UTILITY_TAB_SELECTED_FONT_WEIGHT);
        assert_eq!(nodes[1].font_weight, UTILITY_TAB_IDLE_FONT_WEIGHT);
        assert!(nodes.iter().all(|node| node.overflow == "elide"));
    }
}
