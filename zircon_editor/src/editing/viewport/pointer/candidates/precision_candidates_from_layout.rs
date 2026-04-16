use crate::editing::viewport::pointer::{
    precision::PrecisionCandidate, viewport_pointer_layout::ViewportPointerLayout,
};

use super::{handle_candidate, renderable_candidate, scene_gizmo_candidate};

pub(in crate::editing::viewport::pointer) fn precision_candidates_from_layout(
    layout: &ViewportPointerLayout,
) -> Vec<PrecisionCandidate> {
    let mut candidates = Vec::new();

    for handle in &layout.handles {
        for element in &handle.elements {
            let Some(candidate) =
                handle_candidate(handle.owner, element, &layout.camera, layout.viewport)
            else {
                continue;
            };
            candidates.push(candidate);
        }
    }

    for gizmo in &layout.scene_gizmos {
        for shape in &gizmo.pick_shapes {
            let Some(candidate) =
                scene_gizmo_candidate(gizmo.owner, shape, &layout.camera, layout.viewport)
            else {
                continue;
            };
            candidates.push(candidate);
        }
    }

    for renderable in &layout.renderables {
        let Some(candidate) = renderable_candidate(renderable, &layout.camera, layout.viewport)
        else {
            continue;
        };
        candidates.push(candidate);
    }

    candidates
}
