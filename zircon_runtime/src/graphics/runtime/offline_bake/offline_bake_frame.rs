use crate::core::framework::render::{
    ProbeBakeTiming, ProbeInfluenceShape, ReflectionProbeData, RenderFrameExtract,
};
use crate::core::math::{Quat, Vec3};

use super::offline_bake_output::OfflineBakeOutput;
use super::offline_bake_settings::OfflineBakeSettings;

pub fn offline_bake_frame(
    extract: &RenderFrameExtract,
    settings: &OfflineBakeSettings,
) -> OfflineBakeOutput {
    let mut total_intensity = 0.0;
    for light in &extract.lighting.directional_lights {
        total_intensity += light.intensity;
    }

    let probe_count = settings
        .max_reflection_probes
        .max(usize::from(!extract.geometry.meshes.is_empty()))
        .min(extract.geometry.meshes.len().max(1));
    let mut reflection_probes = Vec::new();
    if total_intensity > f32::EPSILON && settings.max_reflection_probes > 0 {
        for mesh in extract.geometry.meshes.iter().take(probe_count) {
            let mesh_scale = mesh.transform.scale.max(Vec3::splat(0.5));
            let radius = mesh_scale.max_element().max(0.75) * 1.5;
            let Ok(shape) = ProbeInfluenceShape::sphere(radius, radius * 0.25) else {
                continue;
            };
            let Ok(probe) = ReflectionProbeData::try_new(
                mesh.node_id,
                mesh.transform.translation,
                Quat::IDENTITY,
                shape,
                Vec3::splat(radius),
            ) else {
                continue;
            };
            let Ok(probe) = probe
                .try_with_intensity(total_intensity * settings.reflection_probe_scale.max(0.0))
            else {
                continue;
            };
            reflection_probes.push(probe.with_bake_timing(ProbeBakeTiming::EditorManual));
        }
    }

    OfflineBakeOutput { reflection_probes }
}
