use crate::scene::viewport::pointer::{
    precision::PrecisionCandidate, viewport_pointer_layout::ViewportPointerLayout,
};
use crate::scene::viewport::projection::ViewportProjectionContext;

use super::{handle_candidate, renderable_candidate, scene_gizmo_candidate};

pub(in crate::scene::viewport::pointer) fn precision_candidates_from_layout(
    layout: &ViewportPointerLayout,
) -> Vec<PrecisionCandidate> {
    let projection = ViewportProjectionContext::new(&layout.camera, layout.viewport);
    let capacity = layout
        .handles
        .iter()
        .map(|handle| handle.elements.len())
        .sum::<usize>()
        + layout
            .scene_gizmos
            .iter()
            .map(|gizmo| gizmo.pick_shapes.len())
            .sum::<usize>()
        + layout.renderables.len();
    let mut candidates = Vec::with_capacity(capacity);

    for handle in layout.handles.iter() {
        for element in &handle.elements {
            let Some(candidate) = handle_candidate(handle.owner, element, &projection) else {
                continue;
            };
            candidates.push(candidate);
        }
    }

    for gizmo in layout.scene_gizmos.iter() {
        for shape in &gizmo.pick_shapes {
            let Some(candidate) = scene_gizmo_candidate(gizmo.owner, shape, &projection) else {
                continue;
            };
            candidates.push(candidate);
        }
    }

    for renderable in layout.renderables.iter() {
        let Some(candidate) = renderable_candidate(renderable, &projection) else {
            continue;
        };
        candidates.push(candidate);
    }

    candidates
}
