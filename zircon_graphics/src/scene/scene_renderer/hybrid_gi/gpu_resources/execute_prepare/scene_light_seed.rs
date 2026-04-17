use zircon_math::Vec3;
use zircon_scene::RenderDirectionalLightSnapshot;

pub(super) struct SceneLightSeed {
    pub(super) packed_rgb: u32,
    pub(super) strength_q: u32,
}

pub(super) fn scene_light_seed(
    directional_lights: &[RenderDirectionalLightSnapshot],
) -> SceneLightSeed {
    let mut accumulated = Vec3::ZERO;
    for light in directional_lights {
        let intensity = light.intensity.max(0.0);
        accumulated += light.color.max(Vec3::ZERO) * intensity;
    }

    let max_component = accumulated.x.max(accumulated.y).max(accumulated.z);
    let (rgb, strength_q) = if max_component <= f32::EPSILON {
        ([255_u8, 255_u8, 255_u8], 255_u32)
    } else {
        (
            [
                ((accumulated.x / max_component).clamp(0.0, 1.0) * 255.0).round() as u8,
                ((accumulated.y / max_component).clamp(0.0, 1.0) * 255.0).round() as u8,
                ((accumulated.z / max_component).clamp(0.0, 1.0) * 255.0).round() as u8,
            ],
            (max_component.clamp(0.0, 1.0) * 255.0).round() as u32,
        )
    };

    SceneLightSeed {
        packed_rgb: u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16),
        strength_q,
    }
}
