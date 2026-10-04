use std::collections::HashSet;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiTree, UiTreeError},
};

use crate::ui::{surface::resolve_inline_widget_layout_with_cache, text::UiTextMeasureCache};

use super::{
    arrange::{arrange_node, hide_subtree_layout},
    child_frame::inset_frame,
    engine::UiLayoutPassEngineContext,
    slot::UiLayoutSlotIndex,
};

pub(super) fn arrange_inline_widget_children(
    tree: &mut UiTree,
    roots: &[UiNodeId],
    text_measure_cache: &mut UiTextMeasureCache,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let candidates = tree_preorder(tree, roots);
    let mut owner_count = 0_usize;
    let mut binding_count = 0_usize;
    let mut arranged_count = 0_usize;
    let mut rejected_count = 0_usize;
    for parent_id in candidates {
        let resolved = {
            let Some(parent) = tree.node(parent_id) else {
                continue;
            };
            if !parent.effective_visibility().occupies_layout()
                || parent.children.is_empty()
                || parent.layout_cache.frame.width <= 0.0
                || parent.layout_cache.frame.height <= 0.0
            {
                continue;
            }
            resolve_inline_widget_layout_with_cache(
                parent.template_metadata.as_ref(),
                inset_frame(parent.layout_cache.frame, parent.layout_padding),
                parent.layout_cache.clip_frame,
                text_measure_cache,
            )
        };
        let Some(resolved) = resolved else {
            continue;
        };
        owner_count = owner_count.saturating_add(1);
        binding_count = binding_count.saturating_add(resolved.bindings().len());
        let (direct_children, inherited_clip) = {
            let parent = tree
                .node(parent_id)
                .ok_or(UiTreeError::MissingNode(parent_id))?;
            let mut direct_children = HashSet::with_capacity(parent.children.len());
            direct_children.extend(parent.children.iter().copied());
            (direct_children, parent.layout_cache.clip_frame)
        };
        let mut managed_children = HashSet::with_capacity(resolved.bindings().len());
        for binding in resolved.bindings() {
            let node_id = UiNodeId::new(binding.slot.value());
            if !direct_children.contains(&node_id) {
                rejected_count = rejected_count.saturating_add(1);
                continue;
            }
            managed_children.insert(node_id);
            if binding.valid {
                if let Some(frame) = binding.frame {
                    arrange_node(
                        tree,
                        node_id,
                        frame,
                        inherited_clip,
                        slot_index,
                        engine_context,
                    )?;
                    arranged_count = arranged_count.saturating_add(1);
                    continue;
                }
            }
            rejected_count = rejected_count.saturating_add(1);
            hide_subtree_layout(tree, node_id, slot_index, engine_context)?;
        }
        for child_id in direct_children.difference(&managed_children).copied() {
            rejected_count = rejected_count.saturating_add(1);
            hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
        }
    }
    crate::profile_counter!("runtime", "ui.inline_widget.owner_count", owner_count);
    crate::profile_counter!("runtime", "ui.inline_widget.binding_count", binding_count);
    crate::profile_counter!("runtime", "ui.inline_widget.arranged_count", arranged_count);
    crate::profile_counter!("runtime", "ui.inline_widget.rejected_count", rejected_count);
    Ok(())
}

fn tree_preorder(tree: &UiTree, roots: &[UiNodeId]) -> Vec<UiNodeId> {
    let mut result = Vec::with_capacity(roots.len());
    let mut visited = HashSet::with_capacity(roots.len());
    let mut pending = Vec::with_capacity(roots.len());
    pending.extend(roots.iter().rev().copied());
    while let Some(node_id) = pending.pop() {
        if !visited.insert(node_id) {
            continue;
        }
        let Some(node) = tree.node(node_id) else {
            continue;
        };
        result.push(node_id);
        pending.extend(node.children.iter().rev().copied());
    }
    result
}

#[cfg(test)]
#[path = "tests/inline_widgets_performance_tests.rs"]
mod performance_tests;
