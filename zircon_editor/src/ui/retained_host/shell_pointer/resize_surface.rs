use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect,
    event_ui::{UiNodePath, UiRouteId, UiTreeId},
    layout::UiFrame,
    surface::UiPointerEventKind,
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::drawer_resize::HostResizeTargetGroup;
use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::autolayout::ShellRegionId;
use crate::ui::workbench::autolayout::ShellSizePx;

use super::common::{base_target_state, clamp_frame_to_root, frame_if_visible, update_target_node};
use super::node_ids::{
    RESIZE_POINTER_ROOT_NODE_ID, RESIZE_TARGET_BOTTOM_NODE_ID, RESIZE_TARGET_LEFT_NODE_ID,
    RESIZE_TARGET_RIGHT_NODE_ID,
};
use super::route::HostShellPointerRoute;

const RESIZE_ROUTE_ID_BASE: u64 = 51_000;

pub(super) fn build_resize_surface() -> (UiSurface, UiPointerDispatcher, EditorRouteIntentMap) {
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

    let mut route_intents = EditorRouteIntentMap::default();

    for (node_id, path, z_index, route_id, group) in [
        (
            RESIZE_TARGET_LEFT_NODE_ID,
            "editor.workbench.shell_pointer/resize/left",
            10,
            resize_route_id(1),
            HostResizeTargetGroup::Left,
        ),
        (
            RESIZE_TARGET_RIGHT_NODE_ID,
            "editor.workbench.shell_pointer/resize/right",
            20,
            resize_route_id(2),
            HostResizeTargetGroup::Right,
        ),
        (
            RESIZE_TARGET_BOTTOM_NODE_ID,
            "editor.workbench.shell_pointer/resize/bottom",
            30,
            resize_route_id(3),
            HostResizeTargetGroup::Bottom,
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
        route_intents.bind_node(
            node_id,
            route_id,
            EditorRouteIntent::ShellPointer(HostShellPointerRoute::Resize(group)),
        );
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
    (surface, resize_dispatcher, route_intents)
}

const fn resize_route_id(offset: u64) -> UiRouteId {
    UiRouteId::new(RESIZE_ROUTE_ID_BASE + offset)
}

pub(super) fn update_resize_surface(
    surface: &mut UiSurface,
    root_size: ShellSizePx,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) {
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        root_size.width.max(0.0),
        root_size.height.max(0.0),
    );

    let mut changed = false;
    if let Some(root) = surface.tree.node_mut(RESIZE_POINTER_ROOT_NODE_ID) {
        let next_state = base_target_state(false);
        if root.layout_cache.frame != root_frame
            || root.layout_cache.clip_frame.is_some()
            || root.state_flags != next_state
        {
            root.layout_cache.frame = root_frame;
            root.layout_cache.clip_frame = None;
            root.state_flags = next_state;
            changed = true;
        }
    }

    for (node_id, region) in [
        (RESIZE_TARGET_LEFT_NODE_ID, ShellRegionId::Left),
        (RESIZE_TARGET_RIGHT_NODE_ID, ShellRegionId::Right),
        (RESIZE_TARGET_BOTTOM_NODE_ID, ShellRegionId::Bottom),
    ] {
        changed |= update_target_node(
            surface,
            node_id,
            frame_if_visible(clamp_frame_to_root(
                resize_splitter_frame(
                    componentized_workbench_layout_frames.resize_splitter_frame(region),
                ),
                root_frame,
            )),
        );
    }

    if changed {
        surface.rebuild();
    }
}

fn resize_splitter_frame(componentized_resize_splitter_frame: Option<UiFrame>) -> ShellFrame {
    componentized_resize_splitter_frame
        .filter(|frame| frame.width > 0.0 && frame.height > 0.0)
        .unwrap_or_default()
}
