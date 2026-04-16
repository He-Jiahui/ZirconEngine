use zircon_math::Vec3;
use zircon_scene::{RenderBakedLightingExtract, RenderFrameExtract, RenderReflectionProbeSnapshot};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfflineBakeSettings {
    pub ambient_scale: f32,
    pub reflection_probe_scale: f32,
    pub max_reflection_probes: usize,
}

impl Default for OfflineBakeSettings {
    fn default() -> Self {
        Self {
            ambient_scale: 0.2,
            reflection_probe_scale: 0.75,
            max_reflection_probes: 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OfflineBakeOutput {
    pub baked_lighting: RenderBakedLightingExtract,
    pub reflection_probes: Vec<RenderReflectionProbeSnapshot>,
}

pub fn offline_bake_frame(
    extract: &RenderFrameExtract,
    settings: &OfflineBakeSettings,
) -> OfflineBakeOutput {
    let mut weighted_color = Vec3::ZERO;
    let mut total_intensity = 0.0;
    for light in &extract.lighting.directional_lights {
        weighted_color += light.color * light.intensity;
        total_intensity += light.intensity;
    }

    let average_color = if total_intensity > f32::EPSILON {
        weighted_color / total_intensity
    } else {
        Vec3::splat(0.2)
    };
    let baked_intensity = total_intensity.max(0.0) * settings.ambient_scale.max(0.0);
    let baked_lighting = RenderBakedLightingExtract {
        color: average_color,
        intensity: baked_intensity,
    };

    let probe_count = settings
        .max_reflection_probes
        .max(usize::from(!extract.geometry.meshes.is_empty()))
        .min(extract.geometry.meshes.len().max(1));
    let mut reflection_probes = Vec::new();
    if total_intensity > f32::EPSILON && settings.max_reflection_probes > 0 {
        for mesh in extract.geometry.meshes.iter().take(probe_count) {
            let mesh_scale = mesh.transform.scale.max(Vec3::splat(0.5));
            reflection_probes.push(RenderReflectionProbeSnapshot {
                position: mesh.transform.translation,
                radius: mesh_scale.max_element().max(0.75) * 1.5,
                color: average_color,
                intensity: total_intensity * settings.reflection_probe_scale.max(0.0),
            });
        }
    }

    OfflineBakeOutput {
        baked_lighting,
        reflection_probes,
    }
}
