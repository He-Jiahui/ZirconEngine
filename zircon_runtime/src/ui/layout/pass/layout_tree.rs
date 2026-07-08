use zircon_runtime_interface::ui::layout::{UiFrame, UiLayoutEngineSelectionReport, UiSize};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

use crate::ui::text::UiTextMeasureCache;

use super::arrange::arrange_node;
use super::engine::UiLayoutPassEngineContext;
use super::measure::measure_node;
use super::pipeline::{assert_layout_pass_stage, UiLayoutPassStage};
use super::responsive_mui::apply_mui_responsive_layout;

pub fn compute_layout_tree(
    tree: &mut UiTree,
    root_size: UiSize,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    compute_layout_tree_with_text_measure_cache(tree, root_size, None)
}

pub(crate) fn compute_layout_tree_with_text_measure_cache(
    tree: &mut UiTree,
    root_size: UiSize,
    mut text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    assert_layout_pass_stage(UiLayoutPassStage::ResponsiveStyleResolution, 0);
    apply_mui_responsive_layout(tree, root_size)?;

    let roots = tree.roots.clone();
    assert_layout_pass_stage(UiLayoutPassStage::Measurement, 1);
    for root_id in &roots {
        let _ = measure_node(tree, *root_id, text_measure_cache.as_deref_mut())?;
    }

    assert_layout_pass_stage(UiLayoutPassStage::BackendSelection, 2);
    let mut engine_context = UiLayoutPassEngineContext::default();
    assert_layout_pass_stage(UiLayoutPassStage::TaffyBridgeArrangement, 3);
    assert_layout_pass_stage(UiLayoutPassStage::ZirconFallbackArrangement, 4);
    assert_layout_pass_stage(UiLayoutPassStage::ClipAndVirtualWindowPropagation, 5);
    for root_id in roots {
        arrange_node(
            tree,
            root_id,
            UiFrame::new(
                0.0,
                0.0,
                root_size.width.max(0.0),
                root_size.height.max(0.0),
            ),
            None,
            &mut engine_context,
        )?;
    }

    assert_layout_pass_stage(UiLayoutPassStage::SelectionReport, 6);
    Ok(engine_context.finish())
}
