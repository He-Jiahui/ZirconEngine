use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(super) fn nodes(
    surface: &UiSurface,
    ids: &BTreeMap<UiNodeId, String>,
) -> Result<Vec<Value>, String> {
    let mut identities = BTreeSet::new();
    surface
        .arranged_tree
        .nodes
        .iter()
        .map(|node| {
            let node_id = ids
                .get(&node.node_id)
                .ok_or("arranged node has no compiled source identity")?;
            if !identities.insert(node_id) {
                return Err(format!("ambiguous compiled source identity: {node_id}"));
            }
            let metadata = surface
                .tree
                .nodes
                .get(&node.node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .ok_or("arranged node has no component metadata")?;
            let parent = node
                .parent
                .map(|parent| {
                    ids.get(&parent)
                        .ok_or("parent has no compiled source identity")
                })
                .transpose()?;
            let text = surface
                .render_extract
                .list
                .commands
                .iter()
                .filter(|command| command.node_id == node.node_id)
                .filter_map(|command| command.text.as_deref())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(json!({
                "nodeId":node_id,"parentNodeId":parent,"component":metadata.component,
                "visible":node.is_render_visible(),"detached":false,
                "clip":node.clip_to_bounds,"clipBounds":node.clip_frame,
                "bounds":node.frame,"text":text,
            }))
        })
        .collect()
}

pub(super) fn text_nodes(
    surface: &UiSurface,
    ids: &BTreeMap<UiNodeId, String>,
) -> Result<Vec<Value>, String> {
    surface.render_extract.list.commands.iter().filter(|command| command.text.is_some()).map(|command| {
        let node_id = ids.get(&command.node_id).ok_or("text command has no compiled source identity")?;
        Ok(json!({
            "nodeId":node_id,"text":command.text,"layout":command.text_layout,
            "font":{"uri":command.style.font,"family":command.style.font_family,
                "weight":command.style.font_weight,"size":command.style.font_size,"lineHeight":command.style.line_height},
        }))
    }).collect()
}
