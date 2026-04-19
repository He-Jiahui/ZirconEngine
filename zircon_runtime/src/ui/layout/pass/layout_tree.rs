use crate::ui::tree::UiTreeError;
use crate::ui::{layout::UiFrame, layout::UiSize, tree::UiTree};

use super::arrange::arrange_node;
use super::measure::measure_node;

pub fn compute_layout_tree(tree: &mut UiTree, root_size: UiSize) -> Result<(), UiTreeError> {
    let roots = tree.roots.clone();
    for root_id in &roots {
        let _ = measure_node(tree, *root_id)?;
    }

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
        )?;
    }

    Ok(())
}
