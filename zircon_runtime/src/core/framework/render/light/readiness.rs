use super::snapshots::{RenderAmbientLightSnapshot, RenderRectLightSnapshot};

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
    /// Mirrors the renderer's current light consumption path rather than the authored scene data.
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
                directional_light_count,
            ),
            point: RenderLightFamilyReadiness::new(point_light_count, point_light_count),
            spot: RenderLightFamilyReadiness::new(spot_light_count, spot_light_count),
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
#[path = "tests/readiness.rs"]
mod tests;
