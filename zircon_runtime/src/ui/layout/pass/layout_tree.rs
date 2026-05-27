use zircon_runtime_interface::ui::layout::{UiFrame, UiLayoutEngineSelectionReport, UiSize};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

use super::arrange::arrange_node;
use super::engine::UiLayoutPassEngineContext;
use super::measure::measure_node;
use super::responsive_mui::apply_mui_responsive_layout;

pub fn compute_layout_tree(
    tree: &mut UiTree,
    root_size: UiSize,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    apply_mui_responsive_layout(tree, root_size)?;
    let roots = tree.roots.clone();
    for root_id in &roots {
        let _ = measure_node(tree, *root_id)?;
    }

    let mut engine_context = UiLayoutPassEngineContext::default();
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

    Ok(engine_context.finish())
}
