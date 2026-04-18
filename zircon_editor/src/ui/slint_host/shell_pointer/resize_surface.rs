use zircon_ui::{
    UiFrame, UiInputPolicy, UiNodePath, UiPointerDispatchEffect, UiPointerDispatcher,
    UiPointerEventKind, UiSurface, UiTreeId, UiTreeNode,
};

use crate::{ShellRegionId, ShellSizePx, WorkbenchShellGeometry};

use super::common::{base_target_state, clamp_frame_to_root, frame_if_visible, update_target_node};
use super::node_ids::{
    RESIZE_POINTER_ROOT_NODE_ID, RESIZE_TARGET_BOTTOM_NODE_ID, RESIZE_TARGET_LEFT_NODE_ID,
    RESIZE_TARGET_RIGHT_NODE_ID,
};

pub(super) fn build_resize_surface() -> (UiSurface, UiPointerDispatcher) {
    let mut surface = UiSurface::new(UiTreeId::new(
        "zircon.editor.workbench.shell_pointer.resize",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(
            RESIZE_POINTER_ROOT_NODE_ID,
            UiNodePath::new("editor.workbench.shell_pointer.resize"),
        )
        .with_state_flags(base_target_state(false))
        .with_frame(UiFrame::new(0.0, 0.0, 1.0, 1.0)),
    );

    for (node_id, path, z_index) in [
        (
            RESIZE_TARGET_LEFT_NODE_ID,
            "editor.workbench.shell_pointer/resize/left",
            10,
        ),
        (
            RESIZE_TARGET_RIGHT_NODE_ID,
            "editor.workbench.shell_pointer/resize/right",
            20,
        ),
        (
            RESIZE_TARGET_BOTTOM_NODE_ID,
            "editor.workbench.shell_pointer/resize/bottom",
            30,
        ),
    ] {
        surface
            .tree
            .insert_child(
                RESIZE_POINTER_ROOT_NODE_ID,
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_z_index(z_index)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_target_state(true)),
            )
            .expect("resize pointer root must exist");
    }

    let mut resize_dispatcher = UiPointerDispatcher::default();
    for node_id in [
        RESIZE_TARGET_LEFT_NODE_ID,
        RESIZE_TARGET_RIGHT_NODE_ID,
        RESIZE_TARGET_BOTTOM_NODE_ID,
    ] {
        resize_dispatcher.register(node_id, UiPointerEventKind::Down, |_context| {
            UiPointerDispatchEffect::capture()
        });
        resize_dispatcher.register(node_id, UiPointerEventKind::Move, |context| {
            if context.route.captured == Some(context.node_id)
                || context.route.target == Some(context.node_id)
            {
                UiPointerDispatchEffect::handled()
            } else {
                UiPointerDispatchEffect::Unhandled
            }
        });
        resize_dispatcher.register(node_id, UiPointerEventKind::Up, |context| {
            if context.route.captured == Some(context.node_id)
                || context.route.target == Some(context.node_id)
            {
                UiPointerDispatchEffect::handled()
            } else {
                UiPointerDispatchEffect::Unhandled
            }
        });
    }

    surface.rebuild();
    (surface, resize_dispatcher)
}

pub(super) fn update_resize_surface(
    surface: &mut UiSurface,
    root_size: ShellSizePx,
    geometry: &WorkbenchShellGeometry,
) {
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        root_size.width.max(0.0),
        root_size.height.max(0.0),
    );

    if let Some(root) = surface.tree.node_mut(RESIZE_POINTER_ROOT_NODE_ID) {
        root.layout_cache.frame = root_frame;
        root.layout_cache.clip_frame = None;
        root.state_flags = base_target_state(false);
    }

    for (node_id, region) in [
        (RESIZE_TARGET_LEFT_NODE_ID, ShellRegionId::Left),
        (RESIZE_TARGET_RIGHT_NODE_ID, ShellRegionId::Right),
        (RESIZE_TARGET_BOTTOM_NODE_ID, ShellRegionId::Bottom),
    ] {
        update_target_node(
            surface,
            node_id,
            frame_if_visible(clamp_frame_to_root(
                geometry.splitter_frame(region),
                root_frame,
            )),
        );
    }

    surface.rebuild();
}
