use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiTreeId;
use crate::ui::tree::UiTree;

use super::node_visual_data::UiNodeVisualData;
use super::resolve::resolve_command_kind;
use super::{layout_text, UiRenderCommand, UiRenderList};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderExtract {
    pub tree_id: UiTreeId,
    pub list: UiRenderList,
}

impl UiRenderExtract {
    pub fn from_tree(tree: &UiTree) -> Self {
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

        Self {
            tree_id: tree.tree_id.clone(),
            list: UiRenderList { commands },
        }
    }
}
