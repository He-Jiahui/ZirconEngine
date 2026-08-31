use zircon_runtime_interface::ui::{
    surface::{UiArrangedTree, UiRenderExtract},
    tree::UiTree,
};

use crate::ui::{surface::build_arranged_tree, text::UiTextMeasureCache};

use super::extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache;

/// Standalone tree extraction owns one short-lived session for the complete operation.
/// Product surfaces use their retained `UiTextMeasureCache` through the owner-aware entrypoint.
pub fn extract_ui_render_tree(tree: &UiTree) -> UiRenderExtract {
    let arranged_tree = build_arranged_tree(tree);
    extract_ui_render_tree_from_arranged(tree, &arranged_tree)
}

/// Extracts one standalone arranged tree with a process-default compatibility text cache.
/// Product surfaces retain and pass the Core-owned cache through the owner-aware entrypoint.
pub fn extract_ui_render_tree_from_arranged(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
) -> UiRenderExtract {
    let mut text_measure_cache = UiTextMeasureCache::default();
    text_measure_cache.begin_frame();
    let extract = extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
        tree,
        arranged_tree,
        None,
        &mut text_measure_cache,
    );
    text_measure_cache.finish_frame();
    extract
}
