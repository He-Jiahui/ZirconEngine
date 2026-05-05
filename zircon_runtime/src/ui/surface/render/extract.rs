use crate::ui::surface::{build_arranged_tree, is_arranged_render_visible};
use zircon_runtime_interface::ui::surface::UiArrangedTree;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderExtract, UiRenderList};
use zircon_runtime_interface::ui::tree::UiTree;

use super::node_visual_data::UiNodeVisualData;
use super::resolve::resolve_command_kind;
use crate::ui::text::layout_text;

pub fn extract_ui_render_tree(tree: &UiTree) -> UiRenderExtract {
    let arranged_tree = build_arranged_tree(tree);
    extract_ui_render_tree_from_arranged(tree, &arranged_tree)
}

pub fn extract_ui_render_tree_from_arranged(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
) -> UiRenderExtract {
    let commands = arranged_tree
        .draw_order
        .iter()
        .copied()
        .into_iter()
        .filter_map(|node_id| {
            let node = tree.nodes.get(&node_id)?;
            let arranged_node = arranged_tree.get(node_id)?;
            let visual = UiNodeVisualData::resolve(node.template_metadata.as_ref());
            is_arranged_render_visible(arranged_tree, node_id)
                .ok()
                .filter(|visible| *visible)
                .map(|_| {
                    let text_layout = visual.text.as_deref().map(|text| {
                        layout_text(
                            text,
                            &visual.style,
                            arranged_node.frame,
                            Some(arranged_node.clip_frame),
                        )
                    });
                    UiRenderCommand {
                        node_id,
                        kind: resolve_command_kind(
                            &visual.style,
                            visual.text.as_ref(),
                            visual.image.as_ref(),
                        ),
                        frame: arranged_node.frame,
                        clip_frame: Some(arranged_node.clip_frame),
                        z_index: arranged_node.z_index,
                        style: visual.style,
                        text_layout,
                        text: visual.text,
                        image: visual.image,
                        opacity: visual.opacity,
                    }
                })
        })
        .collect();

    UiRenderExtract {
        tree_id: tree.tree_id.clone(),
        list: UiRenderList { commands },
    }
}
