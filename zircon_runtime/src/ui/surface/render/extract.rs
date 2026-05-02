use crate::ui::tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeRenderOrderExt};
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderExtract, UiRenderList};
use zircon_runtime_interface::ui::tree::UiTree;

use super::layout_text;
use super::node_visual_data::UiNodeVisualData;
use super::resolve::resolve_command_kind;

pub fn extract_ui_render_tree(tree: &UiTree) -> UiRenderExtract {
    let commands = tree
        .draw_order()
        .into_iter()
        .filter_map(|node_id| {
            let node = tree.node(node_id)?;
            let visual = UiNodeVisualData::resolve(node.template_metadata.as_ref());
            tree.is_visible_in_tree(node_id)
                .ok()
                .filter(|visible| *visible)
                .map(|_| {
                    let text_layout = visual.text.as_deref().map(|text| {
                        layout_text(
                            text,
                            &visual.style,
                            node.layout_cache.frame,
                            node.layout_cache.clip_frame,
                        )
                    });
                    UiRenderCommand {
                        node_id,
                        kind: resolve_command_kind(
                            &visual.style,
                            visual.text.as_ref(),
                            visual.image.as_ref(),
                        ),
                        frame: node.layout_cache.frame,
                        clip_frame: node.layout_cache.clip_frame,
                        z_index: node.z_index,
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
