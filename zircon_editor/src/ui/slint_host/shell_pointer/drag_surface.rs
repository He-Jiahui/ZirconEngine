use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerDispatcher},
    event_ui::UiNodeId,
    event_ui::UiNodePath,
    event_ui::UiTreeId,
    layout::UiFrame,
    surface::UiPointerEventKind,
    surface::UiSurface,
    tree::UiInputPolicy,
    tree::UiTreeNode,
};

use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_center_band_frame,
    resolve_root_document_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame, resolve_root_status_bar_frame,
};
use crate::ui::slint_host::tab_drag::WorkbenchDragTargetGroup;
use crate::{DockEdge, FloatingWindowModel, ShellSizePx, WorkbenchShellGeometry};

use super::common::{base_target_state, clamp_frame_to_root, frame_if_visible, update_target_node};
use super::drag_frames::DragTargetFrames;
use super::effects::{document_edge_effect, edge_effect_in_frame, side_target_effect};
use super::node_ids::{
    floating_window_attach_node_id, floating_window_edge_node_id,
    floating_window_projection_exclusion_node_id, DOCUMENT_EDGE_BOTTOM_NODE_ID,
    DOCUMENT_EDGE_LEFT_NODE_ID, DOCUMENT_EDGE_RIGHT_NODE_ID, DOCUMENT_EDGE_TOP_NODE_ID,
    DRAG_POINTER_ROOT_NODE_ID, DRAG_TARGET_BOTTOM_NODE_ID, DRAG_TARGET_DOCUMENT_NODE_ID,
    DRAG_TARGET_LEFT_NODE_ID, DRAG_TARGET_RIGHT_NODE_ID,
};
use super::route::WorkbenchShellPointerRoute;

const MIN_SIDE_DROP_EXTENT: f32 = 92.0;
const MIN_BOTTOM_DROP_EXTENT: f32 = 92.0;

pub(super) fn build_drag_surface(
    root_size: ShellSizePx,
    geometry: &WorkbenchShellGeometry,
    drawers_visible: bool,
    floating_windows: &[FloatingWindowModel],
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    floating_window_projection_bundle: Option<&FloatingWindowProjectionBundle>,
) -> (
    UiSurface,
    UiPointerDispatcher,
    BTreeMap<UiNodeId, WorkbenchShellPointerRoute>,
) {
    let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.workbench.shell_pointer.drag"));
    surface.tree.insert_root(
        UiTreeNode::new(
            DRAG_POINTER_ROOT_NODE_ID,
            UiNodePath::new("editor.workbench.shell_pointer.drag"),
        )
        .with_state_flags(base_target_state(false))
        .with_frame(UiFrame::new(0.0, 0.0, 1.0, 1.0)),
    );

    for (node_id, path, z_index) in [
        (
            DRAG_TARGET_DOCUMENT_NODE_ID,
            "editor.workbench.shell_pointer/drag/document",
            10,
        ),
        (
            DRAG_TARGET_BOTTOM_NODE_ID,
            "editor.workbench.shell_pointer/drag/bottom",
            20,
        ),
        (
            DOCUMENT_EDGE_LEFT_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_left",
            30,
        ),
        (
            DOCUMENT_EDGE_RIGHT_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_right",
            31,
        ),
        (
            DOCUMENT_EDGE_TOP_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_top",
            32,
        ),
        (
            DOCUMENT_EDGE_BOTTOM_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_bottom",
            33,
        ),
        (
            DRAG_TARGET_LEFT_NODE_ID,
            "editor.workbench.shell_pointer/drag/left",
            40,
        ),
        (
            DRAG_TARGET_RIGHT_NODE_ID,
            "editor.workbench.shell_pointer/drag/right",
            50,
        ),
    ] {
        surface
            .tree
            .insert_child(
                DRAG_POINTER_ROOT_NODE_ID,
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_z_index(z_index)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_target_state(true)),
            )
            .expect("drag pointer root must exist");
    }

    let root_frame = UiFrame::new(
        0.0,
        0.0,
        root_size.width.max(0.0),
        root_size.height.max(0.0),
    );
    let resolved_center_band_frame = resolve_root_center_band_frame(geometry, shared_root_frames);
    let resolved_status_bar_frame = resolve_root_status_bar_frame(geometry, shared_root_frames);
    let resolved_document_region_frame =
        resolve_root_document_region_frame(geometry, shared_root_frames);
    let resolved_left_region_frame = resolve_root_left_region_frame(geometry, shared_root_frames);
    let resolved_right_region_frame = resolve_root_right_region_frame(geometry, shared_root_frames);
    let resolved_bottom_region_frame =
        resolve_root_bottom_region_frame(geometry, shared_root_frames);
    let overlay_top = resolved_center_band_frame.y.max(0.0);
    let overlay_bottom = resolved_status_bar_frame
        .y
        .min(root_frame.height)
        .max(overlay_top);
    let overlay_height = (overlay_bottom - overlay_top).max(0.0);

    let left_width = resolved_left_region_frame.width.max(MIN_SIDE_DROP_EXTENT);
    let right_width = resolved_right_region_frame.width.max(MIN_SIDE_DROP_EXTENT);
    let bottom_height = resolved_bottom_region_frame
        .height
        .max(MIN_BOTTOM_DROP_EXTENT);

    let left_drag_frame = drawers_visible
        .then(|| {
            frame_if_visible(clamp_frame_to_root(
                UiFrame::new(0.0, overlay_top, left_width, overlay_height),
                root_frame,
            ))
        })
        .flatten();
    let right_drag_frame = drawers_visible
        .then(|| {
            frame_if_visible(clamp_frame_to_root(
                UiFrame::new(
                    (root_frame.width - right_width).max(0.0),
                    overlay_top,
                    right_width,
                    overlay_height,
                ),
                root_frame,
            ))
        })
        .flatten();
    let bottom_drag_frame = drawers_visible
        .then(|| {
            frame_if_visible(clamp_frame_to_root(
                UiFrame::new(
                    0.0,
                    (overlay_bottom - bottom_height).max(overlay_top),
                    root_frame.width,
                    bottom_height,
                ),
                root_frame,
            ))
        })
        .flatten();
    let document_drag_frame = frame_if_visible(clamp_frame_to_root(
        UiFrame::new(
            resolved_document_region_frame.x.max(0.0),
            overlay_top,
            resolved_document_region_frame.width.max(0.0),
            overlay_height,
        ),
        root_frame,
    ));
    let document_edge_frame = frame_if_visible(clamp_frame_to_root(
        resolved_document_region_frame,
        root_frame,
    ));

    if let Some(root) = surface.tree.node_mut(DRAG_POINTER_ROOT_NODE_ID) {
        root.layout_cache.frame = root_frame;
        root.layout_cache.clip_frame = None;
        root.state_flags = base_target_state(false);
    }

    update_target_node(
        &mut surface,
        DRAG_TARGET_DOCUMENT_NODE_ID,
        document_drag_frame,
    );
    update_target_node(&mut surface, DRAG_TARGET_LEFT_NODE_ID, left_drag_frame);
    update_target_node(&mut surface, DRAG_TARGET_RIGHT_NODE_ID, right_drag_frame);
    update_target_node(&mut surface, DRAG_TARGET_BOTTOM_NODE_ID, bottom_drag_frame);
    update_target_node(
        &mut surface,
        DOCUMENT_EDGE_LEFT_NODE_ID,
        document_edge_frame,
    );
    update_target_node(
        &mut surface,
        DOCUMENT_EDGE_RIGHT_NODE_ID,
        document_edge_frame,
    );
    update_target_node(&mut surface, DOCUMENT_EDGE_TOP_NODE_ID, document_edge_frame);
    update_target_node(
        &mut surface,
        DOCUMENT_EDGE_BOTTOM_NODE_ID,
        document_edge_frame,
    );

    let drag_frames = Arc::new(Mutex::new(DragTargetFrames {
        left: left_drag_frame.unwrap_or_default(),
        right: right_drag_frame.unwrap_or_default(),
        bottom: bottom_drag_frame.unwrap_or_default(),
        document: document_edge_frame.unwrap_or_default(),
    }));
    let mut drag_dispatcher = UiPointerDispatcher::default();
    let mut drag_routes = BTreeMap::new();

    let left_frames = Arc::clone(&drag_frames);
    drag_dispatcher.register(
        DRAG_TARGET_LEFT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            side_target_effect(
                WorkbenchDragTargetGroup::Left,
                &left_frames,
                context.route.point,
            )
        },
    );

    let right_frames = Arc::clone(&drag_frames);
    drag_dispatcher.register(
        DRAG_TARGET_RIGHT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            side_target_effect(
                WorkbenchDragTargetGroup::Right,
                &right_frames,
                context.route.point,
            )
        },
    );

    drag_dispatcher.register(
        DRAG_TARGET_BOTTOM_NODE_ID,
        UiPointerEventKind::Move,
        |_context| UiPointerDispatchEffect::handled(),
    );
    drag_dispatcher.register(
        DRAG_TARGET_DOCUMENT_NODE_ID,
        UiPointerEventKind::Move,
        |_context| UiPointerDispatchEffect::handled(),
    );

    let document_edge_frames = Arc::clone(&drag_frames);
    drag_dispatcher.register(
        DOCUMENT_EDGE_LEFT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            document_edge_effect(DockEdge::Left, &document_edge_frames, context.route.point)
        },
    );
    let document_edge_frames = Arc::clone(&drag_frames);
    drag_dispatcher.register(
        DOCUMENT_EDGE_RIGHT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            document_edge_effect(DockEdge::Right, &document_edge_frames, context.route.point)
        },
    );
    let document_edge_frames = Arc::clone(&drag_frames);
    drag_dispatcher.register(
        DOCUMENT_EDGE_TOP_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            document_edge_effect(DockEdge::Top, &document_edge_frames, context.route.point)
        },
    );
    let document_edge_frames = Arc::clone(&drag_frames);
    drag_dispatcher.register(
        DOCUMENT_EDGE_BOTTOM_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            document_edge_effect(DockEdge::Bottom, &document_edge_frames, context.route.point)
        },
    );

    for (index, window) in floating_windows.iter().enumerate() {
        let projected_frame = floating_window_projection_bundle
            .and_then(|bundle| bundle.outer_frame(&window.window_id))
            .or_else(|| {
                floating_window_projection_bundle
                    .is_none()
                    .then_some(geometry.floating_window_frame(&window.window_id))
            });
        let frame = projected_frame
            .and_then(|frame| frame_if_visible(clamp_frame_to_root(frame, root_frame)));
        let exclusion_frame = if floating_window_projection_bundle.is_some() && frame.is_none() {
            frame_if_visible(clamp_frame_to_root(
                geometry.floating_window_frame(&window.window_id),
                root_frame,
            ))
        } else {
            None
        };

        if let Some(exclusion_frame) = exclusion_frame {
            let exclusion_id = floating_window_projection_exclusion_node_id(index);
            surface
                .tree
                .insert_child(
                    DRAG_POINTER_ROOT_NODE_ID,
                    UiTreeNode::new(
                        exclusion_id,
                        UiNodePath::new(format!(
                            "editor.workbench.shell_pointer/floating/{}/exclusion",
                            window.window_id.0
                        )),
                    )
                    .with_z_index(99 + index as i32 * 10)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_target_state(true)),
                )
                .expect("drag pointer root must exist");
            update_target_node(&mut surface, exclusion_id, Some(exclusion_frame));
            drag_dispatcher.register(exclusion_id, UiPointerEventKind::Move, |_context| {
                UiPointerDispatchEffect::handled()
            });
        }

        let Some(frame) = frame else {
            continue;
        };

        let attach_id = floating_window_attach_node_id(index);
        let left_edge_id = floating_window_edge_node_id(index, DockEdge::Left);
        let right_edge_id = floating_window_edge_node_id(index, DockEdge::Right);
        let top_edge_id = floating_window_edge_node_id(index, DockEdge::Top);
        let bottom_edge_id = floating_window_edge_node_id(index, DockEdge::Bottom);

        for (node_id, path_suffix, z_index) in [
            (attach_id, "attach", 100 + index as i32 * 10),
            (left_edge_id, "edge_left", 101 + index as i32 * 10),
            (right_edge_id, "edge_right", 102 + index as i32 * 10),
            (top_edge_id, "edge_top", 103 + index as i32 * 10),
            (bottom_edge_id, "edge_bottom", 104 + index as i32 * 10),
        ] {
            surface
                .tree
                .insert_child(
                    DRAG_POINTER_ROOT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!(
                            "editor.workbench.shell_pointer/floating/{}/{}",
                            window.window_id.0, path_suffix
                        )),
                    )
                    .with_z_index(z_index)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_target_state(true)),
                )
                .expect("drag pointer root must exist");
            update_target_node(&mut surface, node_id, Some(frame));
        }

        drag_routes.insert(
            attach_id,
            WorkbenchShellPointerRoute::FloatingWindow(window.window_id.clone()),
        );
        drag_dispatcher.register(attach_id, UiPointerEventKind::Move, |_context| {
            UiPointerDispatchEffect::handled()
        });

        for (node_id, edge) in [
            (left_edge_id, DockEdge::Left),
            (right_edge_id, DockEdge::Right),
            (top_edge_id, DockEdge::Top),
            (bottom_edge_id, DockEdge::Bottom),
        ] {
            drag_routes.insert(
                node_id,
                WorkbenchShellPointerRoute::FloatingWindowEdge {
                    window_id: window.window_id.clone(),
                    edge,
                },
            );
            drag_dispatcher.register(node_id, UiPointerEventKind::Move, move |context| {
                edge_effect_in_frame(frame, edge, context.route.point)
            });
        }
    }

    surface.rebuild();
    (surface, drag_dispatcher, drag_routes)
}
