use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use zircon_runtime::core::math::Vec3;

const MIN_SPOT_CONE_FOCUS: f32 = 0.25;

pub(super) struct SceneLightSeed {
    pub(super) packed_rgb: u32,
    pub(super) strength_q: u32,
}

pub(super) fn scene_light_seed(
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
) -> SceneLightSeed {
    let mut accumulated = Vec3::ZERO;
    for light in directional_lights {
        let intensity = light.intensity.max(0.0);
        accumulated += light.color.max(Vec3::ZERO) * intensity;
    }
    for light in point_lights {
        let intensity = light.intensity.max(0.0) * local_light_range_weight(light.range);
        accumulated += light.color.max(Vec3::ZERO) * intensity;
    }
    for light in spot_lights {
        let cone_width = (light.outer_angle_radians - light.inner_angle_radians)
            .abs()
            .max(f32::EPSILON);
        let cone_focus = (1.0 / (1.0 + cone_width)).clamp(MIN_SPOT_CONE_FOCUS, 1.0);
        let intensity =
            light.intensity.max(0.0) * local_light_range_weight(light.range) * cone_focus;
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

fn local_light_range_weight(range: f32) -> f32 {
    if range <= f32::EPSILON {
        return 0.0;
    }

    (range / (1.0 + range)).clamp(0.0, 1.0)
}
