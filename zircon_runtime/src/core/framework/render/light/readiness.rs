use super::snapshots::{RenderAmbientLightSnapshot, RenderRectLightSnapshot};

pub const BASIC_SCENE_UNIFORM_DIRECTIONAL_LIGHT_LIMIT: usize = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderLightFamilyReadiness {
    pub total_count: usize,
    pub ready_count: usize,
    pub degraded_count: usize,
}

impl RenderLightFamilyReadiness {
    pub fn new(total_count: usize, ready_count: usize) -> Self {
        let ready_count = ready_count.min(total_count);
        Self {
            total_count,
            ready_count,
            degraded_count: total_count.saturating_sub(ready_count),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderLightReadinessReport {
    pub directional: RenderLightFamilyReadiness,
    pub point: RenderLightFamilyReadiness,
    pub spot: RenderLightFamilyReadiness,
    pub ambient: RenderLightFamilyReadiness,
    pub rect: RenderLightFamilyReadiness,
}

impl RenderLightReadinessReport {
    /// Mirrors the renderer's current light consumption limits rather than the authored scene data.
    pub fn from_light_slices(
        directional_light_count: usize,
        point_light_count: usize,
        spot_light_count: usize,
        ambient_lights: &[RenderAmbientLightSnapshot],
        rect_lights: &[RenderRectLightSnapshot],
    ) -> Self {
        Self {
            directional: RenderLightFamilyReadiness::new(
                directional_light_count,
                ready_directional_light_count(directional_light_count),
            ),
            point: RenderLightFamilyReadiness::new(point_light_count, 0),
            spot: RenderLightFamilyReadiness::new(spot_light_count, 0),
            ambient: RenderLightFamilyReadiness::new(
                ambient_lights.len(),
                ready_ambient_light_count(ambient_lights),
            ),
            rect: RenderLightFamilyReadiness::new(
                rect_lights.len(),
                ready_rect_light_count(rect_lights),
            ),
        }
    }
}

fn ready_directional_light_count(total_count: usize) -> usize {
    total_count.min(BASIC_SCENE_UNIFORM_DIRECTIONAL_LIGHT_LIMIT)
}

fn ready_ambient_light_count(lights: &[RenderAmbientLightSnapshot]) -> usize {
    lights
        .iter()
        .filter(|light| !light.renderer_degraded)
        .count()
}

fn ready_rect_light_count(lights: &[RenderRectLightSnapshot]) -> usize {
    lights
        .iter()
        .filter(|light| !light.renderer_degraded)
        .count()
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{RenderAmbientLightSnapshot, RenderRectLightSnapshot};
    use crate::core::math::{Vec2, Vec3};

    use super::RenderLightReadinessReport;

    #[test]
    fn light_status_counts_split_ready_and_degraded_slots() {
        let ambient_lights = vec![
            RenderAmbientLightSnapshot {
                color: Vec3::ONE,
                intensity: 1.0,
                renderer_degraded: false,
                degradation_reason: None,
            },
            RenderAmbientLightSnapshot {
                color: Vec3::ZERO,
                intensity: 0.0,
                renderer_degraded: true,
                degradation_reason: Some("ambient fallback only".to_string()),
            },
        ];
        let rect_lights = vec![
            rect_light(false),
            RenderRectLightSnapshot {
                renderer_degraded: true,
                degradation_reason: Some("area-light shading unavailable".to_string()),
                ..rect_light(false)
            },
        ];

        let report =
            RenderLightReadinessReport::from_light_slices(2, 2, 1, &ambient_lights, &rect_lights);

        assert_eq!(report.directional.total_count, 2);
        assert_eq!(report.directional.ready_count, 1);
        assert_eq!(report.directional.degraded_count, 1);
        assert_eq!(report.point.total_count, 2);
        assert_eq!(report.point.ready_count, 0);
        assert_eq!(report.point.degraded_count, 2);
        assert_eq!(report.spot.total_count, 1);
        assert_eq!(report.spot.ready_count, 0);
        assert_eq!(report.spot.degraded_count, 1);
        assert_eq!(report.ambient.total_count, 2);
        assert_eq!(report.ambient.ready_count, 1);
        assert_eq!(report.ambient.degraded_count, 1);
        assert_eq!(report.rect.total_count, 2);
        assert_eq!(report.rect.ready_count, 1);
        assert_eq!(report.rect.degraded_count, 1);
    }

    fn rect_light(renderer_degraded: bool) -> RenderRectLightSnapshot {
        RenderRectLightSnapshot {
            node_id: 1,
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, 0.0, -1.0),
            color: Vec3::ONE,
            intensity: 1.0,
            range: 1.0,
            size: Vec2::ONE,
            renderer_degraded,
            degradation_reason: renderer_degraded.then(|| "degraded".to_string()),
        }
    }
}
