use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use bytemuck::Zeroable;

use super::super::super::super::constants::MAX_REFLECTION_PROBES;
use super::super::super::super::reflection_probe_gpu::GpuReflectionProbe;
use super::super::camera_matrices::view_projection;
use super::project_reflection_probe::project_reflection_probe;

pub(in super::super) fn encode_reflection_probes(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuReflectionProbe; MAX_REFLECTION_PROBES], u32) {
    let mut probes = [GpuReflectionProbe::zeroed(); MAX_REFLECTION_PROBES];
    if !enabled {
        return (probes, 0);
    }

    let camera = &extract.view.camera;
    let (view, projection) = view_projection(camera, viewport_size);
    let view_proj = projection * view;
    let camera_position = camera.transform.translation;
    let mut count = 0;

    for probe in extract
        .lighting
        .reflection_probes
        .iter()
        .take(MAX_REFLECTION_PROBES)
    {
        let Some(gpu_probe) = project_reflection_probe(probe, view_proj, camera, camera_position)
        else {
            continue;
        };
        probes[count] = gpu_probe;
        count += 1;
    }

    (probes, count as u32)
}
