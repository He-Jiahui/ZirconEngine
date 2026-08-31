use crate::core::framework::render::{
    ProbeInfluenceShape, ReflectionProbeData, RenderFrameExtract,
};
use crate::core::math::{Quat, Vec3};

use super::offline_bake_output::OfflineBakeOutput;
use super::offline_bake_settings::OfflineBakeSettings;

pub fn offline_bake_frame(
    extract: &RenderFrameExtract,
    settings: &OfflineBakeSettings,
) -> OfflineBakeOutput {
    if settings.max_reflection_probes == 0 || extract.geometry.meshes.is_empty() {
        return OfflineBakeOutput {
            reflection_probes: Vec::new(),
        };
    }
    let total_intensity = extract
        .lighting
        .directional_lights
        .iter()
        .map(|light| light.intensity)
        .sum();

    let probe_count = eligible_reflection_probe_count(
        extract.geometry.meshes.len(),
        settings.max_reflection_probes,
        total_intensity,
    );
    if probe_count == 0 {
        return OfflineBakeOutput {
            reflection_probes: Vec::new(),
        };
    }

    let mut reflection_probes = Vec::with_capacity(probe_count);
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
        let Ok(probe) =
            probe.try_with_intensity(total_intensity * settings.reflection_probe_scale.max(0.0))
        else {
            continue;
        };
        reflection_probes.push(probe);
    }

    OfflineBakeOutput { reflection_probes }
}

fn eligible_reflection_probe_count(
    mesh_count: usize,
    max_reflection_probes: usize,
    total_intensity: f32,
) -> usize {
    if mesh_count == 0 || max_reflection_probes == 0 || !(total_intensity > f32::EPSILON) {
        return 0;
    }
    mesh_count.min(max_reflection_probes)
}

#[cfg(test)]
mod tests {
    use super::eligible_reflection_probe_count;

    #[test]
    fn eligible_probe_count_short_circuits_empty_and_clamps_budget() {
        assert_eq!(eligible_reflection_probe_count(0, 4, 1.0), 0);
        assert_eq!(eligible_reflection_probe_count(8, 0, 1.0), 0);
        assert_eq!(eligible_reflection_probe_count(8, 4, 0.0), 0);
        assert_eq!(eligible_reflection_probe_count(8, 4, f32::NAN), 0);
        assert_eq!(eligible_reflection_probe_count(8, 4, 1.0), 4);
        assert_eq!(eligible_reflection_probe_count(2, 4, 1.0), 2);
    }
}
