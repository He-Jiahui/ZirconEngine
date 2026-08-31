use crate::asset::ModelPrimitiveAsset;
use crate::core::framework::render::RenderMeshBounds;
use crate::graphics::scene::resources::GpuMeshResource;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GpuSceneLocalBounds {
    pub(super) center: [f32; 3],
    pub(super) radius: f32,
    pub(super) force_hzb_visible: bool,
}

pub(super) fn local_bounds_for_gpu_mesh(mesh: &GpuMeshResource) -> RenderMeshBounds {
    RenderMeshBounds::from_min_max(mesh.bounds_min.to_array(), mesh.bounds_max.to_array())
}

pub(super) fn local_bounds_for_model_primitive(
    primitive: &ModelPrimitiveAsset,
) -> RenderMeshBounds {
    RenderMeshBounds::from_positions(primitive.vertices.iter().map(|vertex| vertex.position))
}

pub(super) fn project_local_bounds_for_gpu_scene(
    local_bounds: RenderMeshBounds,
    hzb_bounds_are_temporally_stable: bool,
) -> GpuSceneLocalBounds {
    let bounds_are_finite = local_bounds
        .center
        .iter()
        .all(|component| component.is_finite())
        && local_bounds.radius.is_finite()
        && local_bounds.radius >= 0.0;
    GpuSceneLocalBounds {
        center: if bounds_are_finite {
            local_bounds.center
        } else {
            [0.0; 3]
        },
        radius: if bounds_are_finite {
            local_bounds.radius
        } else {
            0.0
        },
        force_hzb_visible: !bounds_are_finite || !hzb_bounds_are_temporally_stable,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderMeshBounds;

    use super::project_local_bounds_for_gpu_scene;

    #[test]
    fn gpu_scene_projection_preserves_off_center_mesh_local_bounds() {
        let local_bounds = RenderMeshBounds::from_min_max([2.0, -1.0, -3.0], [6.0, 3.0, 1.0]);

        let projected = project_local_bounds_for_gpu_scene(local_bounds, true);

        assert_eq!(projected.center, [4.0, 1.0, -1.0]);
        assert!((projected.radius - 12.0_f32.sqrt()).abs() <= 1.0e-6);
        assert!(!projected.force_hzb_visible);
    }

    #[test]
    fn gpu_scene_projection_forces_temporally_varying_bounds_visible() {
        let local_bounds = RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]);

        let projected = project_local_bounds_for_gpu_scene(local_bounds, false);

        assert_eq!(projected.center, [0.0; 3]);
        assert!((projected.radius - 3.0_f32.sqrt()).abs() <= 1.0e-6);
        assert!(projected.force_hzb_visible);
    }

    #[test]
    fn gpu_scene_projection_sanitizes_invalid_bounds_and_fails_open() {
        let local_bounds = RenderMeshBounds {
            center: [f32::NAN, 2.0, 3.0],
            radius: f32::INFINITY,
            ..RenderMeshBounds::default()
        };

        let projected = project_local_bounds_for_gpu_scene(local_bounds, true);

        assert_eq!(projected.center, [0.0; 3]);
        assert_eq!(projected.radius, 0.0);
        assert!(projected.force_hzb_visible);
    }
}
