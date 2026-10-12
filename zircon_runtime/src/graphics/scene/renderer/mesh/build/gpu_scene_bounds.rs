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

/// 将局部包围体送入 GPUScene 遮挡剔除契约；无效或时变边界须放行。
/// 调用方将 force_hzb_visible 编入 primitive flags，避免误剔除形变网格。
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
#[path = "tests/gpu_scene_bounds.rs"]
mod tests;
