use zircon_ui::{event_ui::UiNodeId, event_ui::UiStateFlags, UiFrame, UiInputPolicy, UiSurface};

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
) {
    if let Some(node) = surface.tree.node_mut(node_id) {
        let interactive = frame.is_some();
        node.layout_cache.frame = frame.unwrap_or_default();
        node.layout_cache.clip_frame = None;
        node.input_policy = if interactive {
            UiInputPolicy::Receive
        } else {
            UiInputPolicy::Ignore
        };
        node.state_flags = base_target_state(interactive);
    }
}

pub(super) fn frame_if_visible(frame: UiFrame) -> Option<UiFrame> {
    (frame.width > 0.0 && frame.height > 0.0).then_some(frame)
}

pub(super) fn clamp_frame_to_root(frame: UiFrame, root: UiFrame) -> UiFrame {
    frame.intersection(root).unwrap_or_default()
}
