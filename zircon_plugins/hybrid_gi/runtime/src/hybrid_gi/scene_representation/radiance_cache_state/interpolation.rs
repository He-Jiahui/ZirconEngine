use zircon_runtime::core::math::Vec3;

use super::super::screen_probe_state::HybridGiScreenProbeDescriptor;

pub(super) const RADIANCE_CACHE_CLIPMAP_RESOLUTION: u32 = 48;
const RADIANCE_CACHE_CLIPMAP_LEVEL_COUNT: u32 = 4;
const RADIANCE_CACHE_BASE_CELL_SIZE: f32 = 1.0;
const RADIANCE_CACHE_CLIPMAP_LEVEL_SCALE: f32 = 2.0;
const RADIANCE_CACHE_PROBE_CENTER_OFFSET: f32 = 0.5;
const RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT: usize = 8;
const RADIANCE_CACHE_INTERPOLATION_WEIGHT_SCALE: u64 = 65_535;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HybridGiRadianceCacheClipmapDescriptor {
    pub(super) level: u32,
    pub(super) anchor: Vec3,
    pub(super) anchor_cell: [i32; 3],
    pub(super) cell_size: f32,
    pub(super) resolution: u32,
}

impl HybridGiRadianceCacheClipmapDescriptor {
    fn interpolation_coordinates(self, world_position: Vec3) -> Option<([i32; 3], [u64; 3])> {
        if !world_position.is_finite()
            || !self.anchor.is_finite()
            || !self.cell_size.is_finite()
            || self.cell_size <= 0.0
        {
            return None;
        }

        let relative_position = world_position - self.anchor;
        if !relative_position.is_finite() {
            return None;
        }
        let probe_coord = relative_position / self.cell_size
            + Vec3::splat(self.resolution as f32 * RADIANCE_CACHE_PROBE_CENTER_OFFSET);
        if !probe_coord.is_finite() {
            return None;
        }

        let lower_bound = RADIANCE_CACHE_PROBE_CENTER_OFFSET;
        let upper_bound = self.resolution as f32 - RADIANCE_CACHE_PROBE_CENTER_OFFSET;
        if ![probe_coord.x, probe_coord.y, probe_coord.z]
            .into_iter()
            .all(|component| component > lower_bound && component < upper_bound)
        {
            return None;
        }

        let interpolation_position = probe_coord - Vec3::splat(RADIANCE_CACHE_PROBE_CENTER_OFFSET);
        let bottom = [
            interpolation_position.x.floor() as i32,
            interpolation_position.y.floor() as i32,
            interpolation_position.z.floor() as i32,
        ];
        let fractional = [
            quantize_interpolation_fraction(interpolation_position.x - bottom[0] as f32),
            quantize_interpolation_fraction(interpolation_position.y - bottom[1] as f32),
            quantize_interpolation_fraction(interpolation_position.z - bottom[2] as f32),
        ];
        Some((bottom, fractional))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct HybridGiRadianceProbeDemand {
    pub(super) clipmap_level: u32,
    pub(super) probe_coord: [i32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HybridGiRadianceProbeInterpolationCorner {
    pub(super) demand: HybridGiRadianceProbeDemand,
    pub(super) weight_q16: u64,
}

pub(super) fn rebuild_radiance_cache_clipmaps(
    clipmaps: &mut Vec<HybridGiRadianceCacheClipmapDescriptor>,
    probes: &[HybridGiScreenProbeDescriptor],
    camera_position: Option<Vec3>,
) {
    clipmaps.clear();
    // Product submission provides the camera. The probe fallback keeps headless fixtures deterministic.
    let Some(anchor_position) = camera_position
        .filter(|position| position.is_finite())
        .or_else(|| {
            probes
                .iter()
                .map(HybridGiScreenProbeDescriptor::bounds_center)
                .find(|center| center.is_finite())
        })
    else {
        return;
    };

    for level in 0..RADIANCE_CACHE_CLIPMAP_LEVEL_COUNT {
        let cell_size =
            RADIANCE_CACHE_BASE_CELL_SIZE * RADIANCE_CACHE_CLIPMAP_LEVEL_SCALE.powi(level as i32);
        if let Some((anchor, anchor_cell)) = snap_clipmap_anchor(anchor_position, cell_size) {
            clipmaps.push(HybridGiRadianceCacheClipmapDescriptor {
                level,
                anchor,
                anchor_cell,
                cell_size,
                resolution: RADIANCE_CACHE_CLIPMAP_RESOLUTION,
            });
        }
    }
}

pub(super) fn radiance_probe_demands_for_position(
    world_position: Vec3,
    clipmaps: &[HybridGiRadianceCacheClipmapDescriptor],
) -> Vec<HybridGiRadianceProbeDemand> {
    radiance_probe_interpolation_corners(world_position, clipmaps)
        .into_iter()
        .map(|corner| corner.demand)
        .collect()
}

pub(super) fn radiance_probe_interpolation_corners(
    world_position: Vec3,
    clipmaps: &[HybridGiRadianceCacheClipmapDescriptor],
) -> Vec<HybridGiRadianceProbeInterpolationCorner> {
    let Some((clipmap, (bottom, fractional))) = clipmaps.iter().copied().find_map(|clipmap| {
        clipmap
            .interpolation_coordinates(world_position)
            .map(|coordinates| (clipmap, coordinates))
    }) else {
        return Vec::new();
    };

    let mut corners = Vec::with_capacity(RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT);
    for x_offset in 0..=1 {
        for y_offset in 0..=1 {
            for z_offset in 0..=1 {
                let x_weight = interpolation_axis_weight(fractional[0], x_offset);
                let y_weight = interpolation_axis_weight(fractional[1], y_offset);
                let z_weight = interpolation_axis_weight(fractional[2], z_offset);
                corners.push(HybridGiRadianceProbeInterpolationCorner {
                    demand: HybridGiRadianceProbeDemand {
                        clipmap_level: clipmap.level,
                        probe_coord: [
                            bottom[0] + x_offset,
                            bottom[1] + y_offset,
                            bottom[2] + z_offset,
                        ],
                    },
                    weight_q16: trilinear_weight_q16(x_weight, y_weight, z_weight),
                });
            }
        }
    }
    corners
}

fn quantize_interpolation_fraction(fraction: f32) -> u64 {
    (fraction * RADIANCE_CACHE_INTERPOLATION_WEIGHT_SCALE as f32)
        .round()
        .clamp(0.0, RADIANCE_CACHE_INTERPOLATION_WEIGHT_SCALE as f32) as u64
}

fn snap_clipmap_anchor(world_position: Vec3, cell_size: f32) -> Option<(Vec3, [i32; 3])> {
    if !world_position.is_finite() || !cell_size.is_finite() || cell_size <= 0.0 {
        return None;
    }

    let cell_component = |component: f32| {
        let value = (component / cell_size).floor();
        (value >= i32::MIN as f32 && value <= i32::MAX as f32).then_some(value as i32)
    };
    let anchor_cell = [
        cell_component(world_position.x)?,
        cell_component(world_position.y)?,
        cell_component(world_position.z)?,
    ];
    Some((
        Vec3::new(
            anchor_cell[0] as f32 * cell_size,
            anchor_cell[1] as f32 * cell_size,
            anchor_cell[2] as f32 * cell_size,
        ),
        anchor_cell,
    ))
}

fn interpolation_axis_weight(fractional: u64, offset: i32) -> u64 {
    if offset == 0 {
        RADIANCE_CACHE_INTERPOLATION_WEIGHT_SCALE.saturating_sub(fractional)
    } else {
        fractional
    }
}

fn trilinear_weight_q16(x_weight: u64, y_weight: u64, z_weight: u64) -> u64 {
    let xy_weight = rounded_divide(
        x_weight.saturating_mul(y_weight),
        RADIANCE_CACHE_INTERPOLATION_WEIGHT_SCALE,
    );
    rounded_divide(
        xy_weight.saturating_mul(z_weight),
        RADIANCE_CACHE_INTERPOLATION_WEIGHT_SCALE,
    )
}

fn rounded_divide(numerator: u64, denominator: u64) -> u64 {
    numerator.saturating_add(denominator / 2) / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radiance_cache_midpoint_marks_all_eight_weighted_corners() {
        let clipmap = HybridGiRadianceCacheClipmapDescriptor {
            level: 0,
            anchor: Vec3::ZERO,
            anchor_cell: [0, 0, 0],
            cell_size: 1.0,
            resolution: RADIANCE_CACHE_CLIPMAP_RESOLUTION,
        };

        let corners = radiance_probe_interpolation_corners(Vec3::ZERO, &[clipmap]);

        assert_eq!(corners.len(), RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT);
        assert!(corners.iter().all(|corner| corner.weight_q16 > 0));
        assert!(corners
            .iter()
            .any(|corner| corner.demand.probe_coord == [23, 23, 23]));
        assert!(corners
            .iter()
            .any(|corner| corner.demand.probe_coord == [24, 24, 24]));
    }
}
