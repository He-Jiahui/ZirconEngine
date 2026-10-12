use std::sync::Arc;

use arc_swap::ArcSwap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect,
    event_ui::{UiNodePath, UiRouteId, UiTreeId},
    layout::UiFrame,
    surface::UiPointerEventKind,
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};
use crate::ui::retained_host::tab_drag::HostDragTargetGroup;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use crate::ui::workbench::autolayout::{ShellFrame, ShellSizePx};
use crate::ui::workbench::layout::DockEdge;
use crate::ui::workbench::model::FloatingWindowModel;

use super::common::{base_target_state, clamp_frame_to_root, frame_if_visible, update_target_node};
use super::drag_frames::DragHitGeometry;
use super::effects::{document_edge_effect, edge_effect_in_frame, side_target_effect};
use super::node_ids::{
    floating_window_attach_node_id, floating_window_edge_node_id, DOCUMENT_EDGE_BOTTOM_NODE_ID,
    DOCUMENT_EDGE_LEFT_NODE_ID, DOCUMENT_EDGE_RIGHT_NODE_ID, DOCUMENT_EDGE_TOP_NODE_ID,
    DRAG_POINTER_ROOT_NODE_ID, DRAG_TARGET_BOTTOM_NODE_ID, DRAG_TARGET_DOCUMENT_NODE_ID,
    DRAG_TARGET_LEFT_NODE_ID, DRAG_TARGET_RIGHT_NODE_ID,
};
use super::route::HostShellPointerRoute;

const MIN_SIDE_DROP_EXTENT: f32 = 92.0;
const MIN_BOTTOM_DROP_EXTENT: f32 = 92.0;
const DRAG_ROUTE_ID_BASE: u64 = 50_000;

pub(super) fn build_drag_surface(
    root_size: ShellSizePx,
    drawers_visible: bool,
    floating_windows: &[FloatingWindowModel],
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    floating_window_projection_bundle: Option<&FloatingWindowProjectionBundle>,
) -> (
    UiSurface,
    UiPointerDispatcher,
    EditorRouteIntentMap,
    Arc<ArcSwap<DragHitGeometry>>,
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

    let mut route_intents = EditorRouteIntentMap::default();

    for (node_id, path, z_index, route_id, route) in [
        (
            DRAG_TARGET_DOCUMENT_NODE_ID,
            "editor.workbench.shell_pointer/drag/document",
            10,
            drag_route_id(1),
            HostShellPointerRoute::DragTarget(HostDragTargetGroup::Document),
        ),
        (
            DRAG_TARGET_BOTTOM_NODE_ID,
            "editor.workbench.shell_pointer/drag/bottom",
            20,
            drag_route_id(2),
            HostShellPointerRoute::DragTarget(HostDragTargetGroup::Bottom),
        ),
        (
            DOCUMENT_EDGE_LEFT_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_left",
            30,
            drag_route_id(3),
            HostShellPointerRoute::DocumentEdge(DockEdge::Left),
        ),
        (
            DOCUMENT_EDGE_RIGHT_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_right",
            31,
            drag_route_id(4),
            HostShellPointerRoute::DocumentEdge(DockEdge::Right),
        ),
        (
            DOCUMENT_EDGE_TOP_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_top",
            32,
            drag_route_id(5),
            HostShellPointerRoute::DocumentEdge(DockEdge::Top),
        ),
        (
            DOCUMENT_EDGE_BOTTOM_NODE_ID,
            "editor.workbench.shell_pointer/drag/document_edge_bottom",
            33,
            drag_route_id(6),
            HostShellPointerRoute::DocumentEdge(DockEdge::Bottom),
        ),
        (
            DRAG_TARGET_LEFT_NODE_ID,
            "editor.workbench.shell_pointer/drag/left",
            40,
            drag_route_id(7),
            HostShellPointerRoute::DragTarget(HostDragTargetGroup::Left),
        ),
        (
            DRAG_TARGET_RIGHT_NODE_ID,
            "editor.workbench.shell_pointer/drag/right",
            50,
            drag_route_id(8),
            HostShellPointerRoute::DragTarget(HostDragTargetGroup::Right),
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
        route_intents.bind_node(node_id, route_id, EditorRouteIntent::ShellPointer(route));
    }

    let (root_frame, resolved_geometry) = resolve_drag_hit_geometry(
        root_size,
        drawers_visible,
        floating_windows,
        componentized_workbench_layout_frames,
        floating_window_projection_bundle,
    );

    if let Some(root) = surface.tree.node_mut(DRAG_POINTER_ROOT_NODE_ID) {
        root.layout_cache.frame = root_frame;
        root.layout_cache.clip_frame = None;
        root.state_flags = base_target_state(false);
    }

    update_target_node(
        &mut surface,
        DRAG_TARGET_DOCUMENT_NODE_ID,
        resolved_geometry.frame(DRAG_TARGET_DOCUMENT_NODE_ID),
    );
    update_target_node(
        &mut surface,
        DRAG_TARGET_LEFT_NODE_ID,
        resolved_geometry.frame(DRAG_TARGET_LEFT_NODE_ID),
    );
    update_target_node(
        &mut surface,
        DRAG_TARGET_RIGHT_NODE_ID,
        resolved_geometry.frame(DRAG_TARGET_RIGHT_NODE_ID),
    );
    update_target_node(
        &mut surface,
        DRAG_TARGET_BOTTOM_NODE_ID,
        resolved_geometry.frame(DRAG_TARGET_BOTTOM_NODE_ID),
    );
    update_target_node(
        &mut surface,
        DOCUMENT_EDGE_LEFT_NODE_ID,
        resolved_geometry.frame(DOCUMENT_EDGE_LEFT_NODE_ID),
    );
    update_target_node(
        &mut surface,
        DOCUMENT_EDGE_RIGHT_NODE_ID,
        resolved_geometry.frame(DOCUMENT_EDGE_RIGHT_NODE_ID),
    );
    update_target_node(
        &mut surface,
        DOCUMENT_EDGE_TOP_NODE_ID,
        resolved_geometry.frame(DOCUMENT_EDGE_TOP_NODE_ID),
    );
    update_target_node(
        &mut surface,
        DOCUMENT_EDGE_BOTTOM_NODE_ID,
        resolved_geometry.frame(DOCUMENT_EDGE_BOTTOM_NODE_ID),
    );

    let hit_geometry = Arc::new(ArcSwap::from_pointee(resolved_geometry));
    let mut drag_dispatcher = UiPointerDispatcher::default();

    let left_geometry = Arc::clone(&hit_geometry);
    drag_dispatcher.register(
        DRAG_TARGET_LEFT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            let geometry = left_geometry.load();
            side_target_effect(
                HostDragTargetGroup::Left,
                &geometry.targets,
                context.route.point,
            )
        },
    );

    let right_geometry = Arc::clone(&hit_geometry);
    drag_dispatcher.register(
        DRAG_TARGET_RIGHT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            let geometry = right_geometry.load();
            side_target_effect(
                HostDragTargetGroup::Right,
                &geometry.targets,
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

    let document_edge_geometry = Arc::clone(&hit_geometry);
    drag_dispatcher.register(
        DOCUMENT_EDGE_LEFT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            let geometry = document_edge_geometry.load();
            document_edge_effect(DockEdge::Left, &geometry.targets, context.route.point)
        },
    );
    let document_edge_geometry = Arc::clone(&hit_geometry);
    drag_dispatcher.register(
        DOCUMENT_EDGE_RIGHT_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            let geometry = document_edge_geometry.load();
            document_edge_effect(DockEdge::Right, &geometry.targets, context.route.point)
        },
    );
    let document_edge_geometry = Arc::clone(&hit_geometry);
    drag_dispatcher.register(
        DOCUMENT_EDGE_TOP_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            let geometry = document_edge_geometry.load();
            document_edge_effect(DockEdge::Top, &geometry.targets, context.route.point)
        },
    );
    let document_edge_geometry = Arc::clone(&hit_geometry);
    drag_dispatcher.register(
        DOCUMENT_EDGE_BOTTOM_NODE_ID,
        UiPointerEventKind::Move,
        move |context| {
            let geometry = document_edge_geometry.load();
            document_edge_effect(DockEdge::Bottom, &geometry.targets, context.route.point)
        },
    );

    for (index, window) in floating_windows.iter().enumerate() {
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
            update_target_node(&mut surface, node_id, hit_geometry.load().frame(node_id));
        }

        route_intents.bind_node(
            attach_id,
            drag_route_id(100 + index as u64 * 10),
            EditorRouteIntent::ShellPointer(HostShellPointerRoute::FloatingWindow(
                window.window_id.clone(),
            )),
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
            route_intents.bind_node(
                node_id,
                floating_window_edge_route_id(index, edge),
                EditorRouteIntent::ShellPointer(HostShellPointerRoute::FloatingWindowEdge {
                    window_id: window.window_id.clone(),
                    edge,
                }),
            );
            let floating_geometry = Arc::clone(&hit_geometry);
            drag_dispatcher.register(node_id, UiPointerEventKind::Move, move |context| {
                let geometry = floating_geometry.load();
                let Some(frame) = geometry.frame(context.node_id) else {
                    return UiPointerDispatchEffect::Unhandled;
                };
                edge_effect_in_frame(frame, edge, context.route.point)
            });
        }
    }

    surface.rebuild();
    (surface, drag_dispatcher, route_intents, hit_geometry)
}

pub(super) fn patch_drag_surface(
    surface: &mut UiSurface,
    hit_geometry: &Arc<ArcSwap<DragHitGeometry>>,
    root_size: ShellSizePx,
    drawers_visible: bool,
    floating_windows: &[FloatingWindowModel],
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    floating_window_projection_bundle: Option<&FloatingWindowProjectionBundle>,
) -> Option<bool> {
    zircon_runtime::profile_counter!("editor", "ui.shell_drag.geometry_resolve_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.shell_drag.floating_frame_candidate_count",
        floating_windows.len()
    );
    let (root_frame, next_geometry) = resolve_drag_hit_geometry(
        root_size,
        drawers_visible,
        floating_windows,
        componentized_workbench_layout_frames,
        floating_window_projection_bundle,
    );
    let base_node_ids = [
        DRAG_POINTER_ROOT_NODE_ID,
        DRAG_TARGET_DOCUMENT_NODE_ID,
        DRAG_TARGET_LEFT_NODE_ID,
        DRAG_TARGET_RIGHT_NODE_ID,
        DRAG_TARGET_BOTTOM_NODE_ID,
        DOCUMENT_EDGE_LEFT_NODE_ID,
        DOCUMENT_EDGE_RIGHT_NODE_ID,
        DOCUMENT_EDGE_TOP_NODE_ID,
        DOCUMENT_EDGE_BOTTOM_NODE_ID,
    ];
    zircon_runtime::profile_counter!(
        "editor",
        "ui.shell_drag.node_candidate_count",
        base_node_ids
            .len()
            .saturating_add(floating_windows.len().saturating_mul(5))
    );
    let base_nodes_missing = base_node_ids
        .iter()
        .any(|node_id| surface.tree.node(*node_id).is_none());
    let floating_nodes_missing = (0..floating_windows.len()).any(|index| {
        [
            floating_window_attach_node_id(index),
            floating_window_edge_node_id(index, DockEdge::Left),
            floating_window_edge_node_id(index, DockEdge::Right),
            floating_window_edge_node_id(index, DockEdge::Top),
            floating_window_edge_node_id(index, DockEdge::Bottom),
        ]
        .into_iter()
        .any(|node_id| surface.tree.node(node_id).is_none())
    });
    if base_nodes_missing || floating_nodes_missing {
        zircon_runtime::profile_counter!("editor", "ui.shell_drag.topology_miss_count", 1);
        return None;
    }

    let root_changed = surface
        .tree
        .node(DRAG_POINTER_ROOT_NODE_ID)
        .is_some_and(|root| {
            root.layout_cache.frame != root_frame
                || root.layout_cache.clip_frame.is_some()
                || root.state_flags != base_target_state(false)
        });
    if root_changed {
        let root = surface
            .tree
            .node_mut(DRAG_POINTER_ROOT_NODE_ID)
            .expect("validated drag pointer root must exist");
        root.layout_cache.frame = root_frame;
        root.layout_cache.clip_frame = None;
        root.state_flags = base_target_state(false);
    }

    let mut node_patch_count = if root_changed { 1 } else { 0 };
    for node_id in base_node_ids.into_iter().skip(1) {
        if update_target_node(surface, node_id, next_geometry.frame(node_id)) {
            node_patch_count += 1;
        }
    }
    for index in 0..floating_windows.len() {
        for node_id in [
            floating_window_attach_node_id(index),
            floating_window_edge_node_id(index, DockEdge::Left),
            floating_window_edge_node_id(index, DockEdge::Right),
            floating_window_edge_node_id(index, DockEdge::Top),
            floating_window_edge_node_id(index, DockEdge::Bottom),
        ] {
            if update_target_node(surface, node_id, next_geometry.frame(node_id)) {
                node_patch_count += 1;
            }
        }
    }
    let changed = node_patch_count > 0;
    if !changed {
        zircon_runtime::profile_counter!("editor", "ui.shell_drag.geometry_reuse_count", 1);
    }
    if changed {
        surface.rebuild();
        hit_geometry.store(Arc::new(next_geometry));
        record_current_ui_perf_counter(UiPerfCounter::ShellDragGeometryPatchCount, 1.0);
        record_current_ui_perf_counter(
            UiPerfCounter::ShellDragNodePatchCount,
            node_patch_count as f64,
        );
    }
    Some(changed)
}

fn resolve_drag_hit_geometry(
    root_size: ShellSizePx,
    drawers_visible: bool,
    floating_windows: &[FloatingWindowModel],
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    floating_window_projection_bundle: Option<&FloatingWindowProjectionBundle>,
) -> (UiFrame, DragHitGeometry) {
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        root_size.width.max(0.0),
        root_size.height.max(0.0),
    );
    let resolved_center_band_frame = componentized_workbench_layout_frames
        .center_band_frame
        .and_then(frame_if_visible)
        .map(shell_frame)
        .unwrap_or_default();
    let resolved_status_bar_frame = componentized_workbench_layout_frames
        .status_bar_frame
        .and_then(frame_if_visible)
        .map(shell_frame)
        .unwrap_or_default();
    let resolved_document_region_frame = componentized_workbench_layout_frames
        .document_region_frame
        .and_then(frame_if_visible)
        .map(shell_frame)
        .unwrap_or_default();
    let resolved_left_region_frame = componentized_workbench_layout_frames
        .left_region_frame
        .and_then(frame_if_visible)
        .map(shell_frame);
    let resolved_right_region_frame = componentized_workbench_layout_frames
        .right_region_frame
        .and_then(frame_if_visible)
        .map(shell_frame);
    let resolved_bottom_region_frame = componentized_workbench_layout_frames
        .bottom_region_frame
        .and_then(frame_if_visible)
        .map(shell_frame);
    let root_projection_visible =
        frame_is_visible(resolved_center_band_frame) && frame_is_visible(resolved_status_bar_frame);
    let overlay_top = resolved_center_band_frame.y.max(0.0);
    let overlay_bottom = resolved_status_bar_frame
        .y
        .min(root_frame.height)
        .max(overlay_top);
    let overlay_height = (overlay_bottom - overlay_top).max(0.0);

    let left_drag_frame = (drawers_visible && root_projection_visible)
        .then(|| resolved_left_region_frame)
        .flatten()
        .and_then(|frame| {
            let left_width = frame.width.max(MIN_SIDE_DROP_EXTENT);
            frame_if_visible(clamp_frame_to_root(
                UiFrame::new(frame.x, overlay_top, left_width, overlay_height),
                root_frame,
            ))
        });
    let right_drag_frame = (drawers_visible && root_projection_visible)
        .then(|| resolved_right_region_frame)
        .flatten()
        .and_then(|frame| {
            let right_width = frame.width.max(MIN_SIDE_DROP_EXTENT);
            frame_if_visible(clamp_frame_to_root(
                UiFrame::new(frame.x, overlay_top, right_width, overlay_height),
                root_frame,
            ))
        });
    let bottom_drag_frame = (drawers_visible && root_projection_visible)
        .then(|| resolved_bottom_region_frame)
        .flatten()
        .and_then(|frame| {
            let bottom_height = frame.height.max(MIN_BOTTOM_DROP_EXTENT);
            frame_if_visible(clamp_frame_to_root(
                UiFrame::new(frame.x, frame.y, frame.width, bottom_height),
                root_frame,
            ))
        });
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

    let floating_frames = floating_windows
        .iter()
        .map(|window| {
            floating_window_projection_bundle
                .and_then(|bundle| bundle.outer_frame(&window.window_id))
                .and_then(|frame| frame_if_visible(clamp_frame_to_root(frame, root_frame)))
        })
        .collect();
    let geometry = DragHitGeometry::new(
        document_drag_frame,
        left_drag_frame,
        right_drag_frame,
        bottom_drag_frame,
        document_edge_frame,
        floating_frames,
    );
    (root_frame, geometry)
}

const fn drag_route_id(offset: u64) -> UiRouteId {
    UiRouteId::new(DRAG_ROUTE_ID_BASE + offset)
}

const fn floating_window_edge_route_id(index: usize, edge: DockEdge) -> UiRouteId {
    let offset = match edge {
        DockEdge::Left => 101,
        DockEdge::Right => 102,
        DockEdge::Top => 103,
        DockEdge::Bottom => 104,
    };
    drag_route_id(offset + index as u64 * 10)
}

fn shell_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}

fn frame_is_visible(frame: ShellFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}
