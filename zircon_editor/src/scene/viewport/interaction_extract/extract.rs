use std::sync::Arc;

use crate::scene::viewport::{HandleOverlayExtract, RenderMeshSnapshot, SceneGizmoOverlayExtract};

#[derive(Clone, Debug)]
pub(in crate::scene::viewport) struct ViewportInteractionExtract {
    handles: Arc<[HandleOverlayExtract]>,
    scene_gizmos: Arc<[SceneGizmoOverlayExtract]>,
    render_meshes: Arc<[RenderMeshSnapshot]>,
}

impl ViewportInteractionExtract {
    pub(super) fn new(
        handles: Vec<HandleOverlayExtract>,
        scene_gizmos: Vec<SceneGizmoOverlayExtract>,
        render_meshes: &[RenderMeshSnapshot],
    ) -> Self {
        Self {
            handles: handles.into(),
            scene_gizmos: scene_gizmos.into(),
            render_meshes: cloned_arc_slice(render_meshes),
        }
    }

    pub(in crate::scene::viewport) fn handles(&self) -> Arc<[HandleOverlayExtract]> {
        Arc::clone(&self.handles)
    }

    pub(in crate::scene::viewport) fn scene_gizmos(&self) -> Arc<[SceneGizmoOverlayExtract]> {
        Arc::clone(&self.scene_gizmos)
    }

    pub(in crate::scene::viewport) fn render_meshes(&self) -> &[RenderMeshSnapshot] {
        &self.render_meshes
    }
}

fn cloned_arc_slice<T: Clone>(source: &[T]) -> Arc<[T]> {
    Arc::from(source)
}

#[cfg(test)]
#[path = "extract/direct_arc_slice_tests.rs"]
mod direct_arc_slice_tests;
