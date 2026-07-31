use crate::ui::retained_host as host_contract;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn project_node_into_mount(
    mut node: host_contract::TemplatePaneNodeData,
    mount_frame: Option<UiFrame>,
) -> host_contract::TemplatePaneNodeData {
    let Some(mount_frame) = mount_frame else {
        return node;
    };
    node.frame.x += mount_frame.x;
    node.frame.y += mount_frame.y;
    if node.has_clip_frame {
        node.clip_frame.x += mount_frame.x;
        node.clip_frame.y += mount_frame.y;
    }
    if node.has_popup_anchor {
        node.popup_anchor_x += mount_frame.x;
        node.popup_anchor_y += mount_frame.y;
    }
    node
}
