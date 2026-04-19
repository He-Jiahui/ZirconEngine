use crate::core::framework::render::{RenderReflectionProbeSnapshot, ViewportCameraSnapshot};
use crate::core::math::{Mat4, Vec3};

use super::super::super::super::reflection_probe_gpu::GpuReflectionProbe;
use super::reflection_probe_radius::reflection_probe_radius;

pub(super) fn project_reflection_probe(
    probe: &RenderReflectionProbeSnapshot,
    view_proj: Mat4,
    camera: &ViewportCameraSnapshot,
    camera_position: Vec3,
) -> Option<GpuReflectionProbe> {
    let clip = view_proj * probe.position.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }

    let uv_x = (0.5 + ndc.x * 0.5).clamp(0.0, 1.0);
    let uv_y = (0.5 - ndc.y * 0.5).clamp(0.0, 1.0);
    let radius = reflection_probe_radius(camera, probe.radius, probe.position, camera_position);

    Some(GpuReflectionProbe {
        screen_uv_and_radius: [uv_x, uv_y, radius, 0.0],
        color_and_intensity: [
            probe.color.x.max(0.0),
            probe.color.y.max(0.0),
            probe.color.z.max(0.0),
            probe.intensity.max(0.0),
        ],
    })
}
