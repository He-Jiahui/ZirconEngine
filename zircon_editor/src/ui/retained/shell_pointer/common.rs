use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    tree::UiInputPolicy,
};

pub(super) fn base_target_state(interactive: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

pub(super) fn update_target_node(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: Option<UiFrame>,
) -> bool {
    let interactive = frame.is_some();
    let next_frame = frame.unwrap_or_default();
    let next_input_policy = if interactive {
        UiInputPolicy::Receive
    } else {
        UiInputPolicy::Ignore
    };
    let next_state = base_target_state(interactive);
    let changed = surface.tree.node(node_id).is_some_and(|node| {
        node.layout_cache.frame != next_frame
            || node.layout_cache.clip_frame.is_some()
            || node.input_policy != next_input_policy
            || node.state_flags != next_state
    });
    if !changed {
        return false;
    }

    if let Some(node) = surface.tree.node_mut(node_id) {
        node.layout_cache.frame = next_frame;
        node.layout_cache.clip_frame = None;
        node.input_policy = next_input_policy;
        node.state_flags = next_state;
        return true;
    }
    false
}

pub(super) fn frame_if_visible(frame: UiFrame) -> Option<UiFrame> {
    (frame.width > 0.0 && frame.height > 0.0).then_some(frame)
}

pub(super) fn clamp_frame_to_root(frame: UiFrame, root: UiFrame) -> UiFrame {
    frame.intersection(root).unwrap_or_default()
}
