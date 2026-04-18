use std::sync::{Arc, Mutex};

use zircon_ui::{UiFrame, UiPoint, UiPointerDispatchEffect};

use crate::ui::slint_host::tab_drag::WorkbenchDragTargetGroup;
use crate::DockEdge;

use super::drag_frames::DragTargetFrames;

const MAX_DOCUMENT_EDGE_EXTENT: f32 = 96.0;

pub(super) fn side_target_effect(
    side: WorkbenchDragTargetGroup,
    frames: &Arc<Mutex<DragTargetFrames>>,
    point: UiPoint,
) -> UiPointerDispatchEffect {
    let frames = *frames
        .lock()
        .expect("shell pointer drag frames lock poisoned");
    let (side_frame, side_distance) = match side {
        WorkbenchDragTargetGroup::Left => (frames.left, point.x - frames.left.x),
        WorkbenchDragTargetGroup::Right => (frames.right, frames.right.right() - point.x),
        WorkbenchDragTargetGroup::Bottom | WorkbenchDragTargetGroup::Document => {
            return UiPointerDispatchEffect::Unhandled;
        }
    };
    if !side_frame.contains_point(point) {
        return UiPointerDispatchEffect::Unhandled;
    }
    if !frames.bottom.contains_point(point) {
        return UiPointerDispatchEffect::handled();
    }

    let bottom_distance = frames.bottom.bottom() - point.y;
    if side_distance <= bottom_distance {
        UiPointerDispatchEffect::handled()
    } else {
        UiPointerDispatchEffect::passthrough()
    }
}

pub(super) fn document_edge_effect(
    edge: DockEdge,
    frames: &Arc<Mutex<DragTargetFrames>>,
    point: UiPoint,
) -> UiPointerDispatchEffect {
    let frames = *frames
        .lock()
        .expect("shell pointer drag frames lock poisoned");
    edge_effect_in_frame(frames.document, edge, point)
}

pub(super) fn edge_effect_in_frame(
    frame: UiFrame,
    edge: DockEdge,
    point: UiPoint,
) -> UiPointerDispatchEffect {
    if !frame.contains_point(point) {
        return UiPointerDispatchEffect::Unhandled;
    }

    let Some((nearest_edge, distance)) = nearest_edge(frame, point) else {
        return UiPointerDispatchEffect::Unhandled;
    };
    if nearest_edge != edge {
        return UiPointerDispatchEffect::passthrough();
    }

    let extent = edge_extent(frame, edge).min(MAX_DOCUMENT_EDGE_EXTENT);
    if distance <= extent {
        UiPointerDispatchEffect::handled()
    } else {
        UiPointerDispatchEffect::passthrough()
    }
}

fn nearest_edge(frame: UiFrame, point: UiPoint) -> Option<(DockEdge, f32)> {
    if !frame.contains_point(point) {
        return None;
    }

    [
        (DockEdge::Left, point.x - frame.x),
        (DockEdge::Right, frame.right() - point.x),
        (DockEdge::Top, point.y - frame.y),
        (DockEdge::Bottom, frame.bottom() - point.y),
    ]
    .into_iter()
    .min_by(|(_, left), (_, right)| left.total_cmp(right))
}

fn edge_extent(frame: UiFrame, edge: DockEdge) -> f32 {
    match edge {
        DockEdge::Left | DockEdge::Right => frame.width * 0.25,
        DockEdge::Top | DockEdge::Bottom => frame.height * 0.25,
    }
}
