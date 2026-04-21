use std::collections::BTreeSet;

use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use crate::core::math::Vec3;
use crate::graphics::types::{
    hybrid_gi_voxel_clipmap_bounds_cell_ranges, hybrid_gi_voxel_clipmap_cell_bit_index,
    HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap, HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
};

use super::representation::HybridGiCardDescriptor;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HybridGiVoxelClipmapDescriptor {
    clipmap_id: u32,
    center: Vec3,
    half_extent: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HybridGiVoxelSceneState {
    resident_clipmap_ids: Vec<u32>,
    dirty_clipmap_ids: Vec<u32>,
    invalidated_clipmap_ids: Vec<u32>,
    clipmap_descriptors: Vec<HybridGiVoxelClipmapDescriptor>,
    voxel_cells: Vec<HybridGiPrepareVoxelCell>,
}

impl HybridGiVoxelSceneState {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn synchronize(
        &mut self,
        cards: &[HybridGiCardDescriptor],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
        clipmap_budget: usize,
        scene_changed: bool,
    ) {
        let (resident_clipmap_ids, clipmap_descriptors) =
            build_clipmap_descriptors(cards, clipmap_budget);
        let voxel_cells = build_voxel_cells(
            cards,
            directional_lights,
            point_lights,
            spot_lights,
            &clipmap_descriptors,
        );
        let resident_clipmap_set = resident_clipmap_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let clipmaps_changed = self.resident_clipmap_ids != resident_clipmap_ids
            || self.clipmap_descriptors != clipmap_descriptors;

        self.invalidated_clipmap_ids = self
            .resident_clipmap_ids
            .iter()
            .copied()
            .filter(|clipmap_id| !resident_clipmap_set.contains(clipmap_id))
            .collect();
        self.dirty_clipmap_ids = if scene_changed || clipmaps_changed {
            resident_clipmap_ids.clone()
        } else {
            Vec::new()
        };
        self.resident_clipmap_ids = resident_clipmap_ids;
        self.clipmap_descriptors = clipmap_descriptors;
        self.voxel_cells = voxel_cells;
    }

    pub(crate) fn resident_clipmap_count(&self) -> usize {
        self.resident_clipmap_ids.len()
    }

    #[cfg(test)]
    pub(crate) fn resident_clipmap_ids(&self) -> Vec<u32> {
        self.resident_clipmap_ids.clone()
    }

    pub(crate) fn dirty_clipmap_count(&self) -> usize {
        self.dirty_clipmap_ids.len()
    }

    #[cfg(test)]
    pub(crate) fn dirty_clipmap_ids(&self) -> Vec<u32> {
        self.dirty_clipmap_ids.clone()
    }

    pub(crate) fn invalidated_clipmap_count(&self) -> usize {
        self.invalidated_clipmap_ids.len()
    }

    pub(crate) fn clipmap_descriptors_snapshot(&self) -> Vec<(u32, Vec3, f32)> {
        self.clipmap_descriptors
            .iter()
            .map(|descriptor| {
                (
                    descriptor.clipmap_id,
                    descriptor.center,
                    descriptor.half_extent,
                )
            })
            .collect()
    }

    pub(crate) fn voxel_cells_snapshot(&self) -> Vec<HybridGiPrepareVoxelCell> {
        self.voxel_cells.clone()
    }

    #[cfg(test)]
    pub(crate) fn invalidated_clipmap_ids(&self) -> Vec<u32> {
        self.invalidated_clipmap_ids.clone()
    }

    #[cfg(test)]
    pub(crate) fn clipmap_descriptors(&self) -> Vec<(u32, [f32; 3], f32)> {
        self.clipmap_descriptors
            .iter()
            .map(|descriptor| {
                (
                    descriptor.clipmap_id,
                    [
                        descriptor.center.x,
                        descriptor.center.y,
                        descriptor.center.z,
                    ],
                    descriptor.half_extent,
                )
            })
            .collect()
    }
}

fn build_clipmap_descriptors(
    cards: &[HybridGiCardDescriptor],
    clipmap_budget: usize,
) -> (Vec<u32>, Vec<HybridGiVoxelClipmapDescriptor>) {
    if cards.is_empty() || clipmap_budget == 0 {
        return (Vec::new(), Vec::new());
    }

    let (scene_bounds_min, scene_bounds_max) = scene_bounds(cards);
    let scene_center = (scene_bounds_min + scene_bounds_max) * 0.5;
    let base_half_extent = (((scene_bounds_max - scene_bounds_min).max_element()) * 0.5)
        .max(1.0)
        .ceil();
    let resident_clipmap_ids = (0..clipmap_budget)
        .map(|clipmap_id| clipmap_id as u32)
        .collect::<Vec<_>>();
    let clipmap_descriptors = resident_clipmap_ids
        .iter()
        .map(|&clipmap_id| HybridGiVoxelClipmapDescriptor {
            clipmap_id,
            center: scene_center,
            half_extent: base_half_extent * 2.0_f32.powi(clipmap_id as i32),
        })
        .collect();

    (resident_clipmap_ids, clipmap_descriptors)
}

fn build_voxel_cells(
    cards: &[HybridGiCardDescriptor],
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
    clipmap_descriptors: &[HybridGiVoxelClipmapDescriptor],
) -> Vec<HybridGiPrepareVoxelCell> {
    clipmap_descriptors
        .iter()
        .flat_map(|descriptor| {
            let clipmap = HybridGiPrepareVoxelClipmap {
                clipmap_id: descriptor.clipmap_id,
                center: descriptor.center,
                half_extent: descriptor.half_extent,
            };
            let mut occupancy_counts = [0_u32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
            let mut dominant_card_ids = [0_u32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
            let mut dominant_strengths = [0.0_f32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
            let mut dominant_radiance_present = [false; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
            let mut dominant_radiance_rgb = [[0_u8; 3]; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
            for card in cards {
                let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
                    hybrid_gi_voxel_clipmap_bounds_cell_ranges(
                        &clipmap,
                        card.bounds_center,
                        card.bounds_radius,
                    )
                else {
                    continue;
                };
                let radiance_rgb =
                    card_voxel_radiance_rgb(card, directional_lights, point_lights, spot_lights);
                let radiance_strength = voxel_radiance_strength(radiance_rgb);

                for z in z_start..=z_end {
                    for y in y_start..=y_end {
                        for x in x_start..=x_end {
                            let cell_index = hybrid_gi_voxel_clipmap_cell_bit_index(x, y, z);
                            occupancy_counts[cell_index] =
                                occupancy_counts[cell_index].saturating_add(1);
                            let should_replace = dominant_card_ids[cell_index] == 0
                                || radiance_strength > dominant_strengths[cell_index]
                                || (radiance_strength == dominant_strengths[cell_index]
                                    && card.card_id > dominant_card_ids[cell_index]);
                            if should_replace {
                                dominant_card_ids[cell_index] = card.card_id;
                                dominant_strengths[cell_index] = radiance_strength;
                                dominant_radiance_present[cell_index] = true;
                                dominant_radiance_rgb[cell_index] = radiance_rgb;
                            }
                        }
                    }
                }
            }

            (0..HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT).map(move |cell_index| {
                HybridGiPrepareVoxelCell {
                    clipmap_id: descriptor.clipmap_id,
                    cell_index: cell_index as u32,
                    occupancy_count: occupancy_counts[cell_index],
                    dominant_card_id: dominant_card_ids[cell_index],
                    radiance_present: dominant_radiance_present[cell_index],
                    radiance_rgb: dominant_radiance_rgb[cell_index],
                }
            })
        })
        .collect()
}

fn card_voxel_radiance_rgb(
    card: &HybridGiCardDescriptor,
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
) -> [u8; 3] {
    let tint = saturate_vec3(Vec3::new(
        card.mesh.tint.x,
        card.mesh.tint.y,
        card.mesh.tint.z,
    ));
    let card_normal = card_normal(card);
    let direct_light = directional_lights.iter().fold(Vec3::ZERO, |acc, light| {
        acc + directional_light_contribution(card_normal, light)
    }) + point_lights.iter().fold(Vec3::ZERO, |acc, light| {
        acc + point_light_contribution(card.bounds_center, card_normal, light)
    }) + spot_lights.iter().fold(Vec3::ZERO, |acc, light| {
        acc + spot_light_contribution(card.bounds_center, card_normal, light)
    });
    let radiance = tint * 0.45 + component_mul(tint, direct_light * 0.9);
    quantize_voxel_radiance(radiance.max(Vec3::ZERO))
}

fn card_normal(card: &HybridGiCardDescriptor) -> Vec3 {
    let normal = card.mesh.transform.forward();
    if normal == Vec3::ZERO {
        -Vec3::Z
    } else {
        normal
    }
}

fn saturate_vec3(value: Vec3) -> Vec3 {
    Vec3::new(
        value.x.clamp(0.0, 1.0),
        value.y.clamp(0.0, 1.0),
        value.z.clamp(0.0, 1.0),
    )
}

fn component_mul(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

fn quantize_voxel_radiance(radiance: Vec3) -> [u8; 3] {
    let mapped = Vec3::new(
        radiance.x / (1.0 + radiance.x),
        radiance.y / (1.0 + radiance.y),
        radiance.z / (1.0 + radiance.z),
    );
    [
        quantize_voxel_radiance_channel(mapped.x),
        quantize_voxel_radiance_channel(mapped.y),
        quantize_voxel_radiance_channel(mapped.z),
    ]
}

fn quantize_voxel_radiance_channel(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn double_sided_orientation(card_normal: Vec3, light_direction: Vec3) -> f32 {
    let direction = light_direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        return 0.0;
    }

    card_normal.dot(direction).abs().max(0.2)
}

fn directional_light_contribution(
    card_normal: Vec3,
    light: &RenderDirectionalLightSnapshot,
) -> Vec3 {
    let incoming = (-light.direction).normalize_or_zero();
    if incoming == Vec3::ZERO {
        return Vec3::ZERO;
    }

    let strength = light.intensity.max(0.0) * double_sided_orientation(card_normal, incoming);
    saturate_vec3(light.color) * strength
}

fn point_light_contribution(
    card_center: Vec3,
    card_normal: Vec3,
    light: &RenderPointLightSnapshot,
) -> Vec3 {
    if light.range <= 0.0 {
        return Vec3::ZERO;
    }

    let to_light = light.position - card_center;
    let distance = to_light.length();
    if distance >= light.range {
        return Vec3::ZERO;
    }

    let attenuation = (1.0 - (distance / light.range)).powi(2);
    let strength = light.intensity.max(0.0)
        * attenuation
        * double_sided_orientation(card_normal, to_light.normalize_or_zero());
    saturate_vec3(light.color) * strength
}

fn spot_light_contribution(
    card_center: Vec3,
    card_normal: Vec3,
    light: &RenderSpotLightSnapshot,
) -> Vec3 {
    if light.range <= 0.0 {
        return Vec3::ZERO;
    }

    let to_light = light.position - card_center;
    let distance = to_light.length();
    if distance >= light.range {
        return Vec3::ZERO;
    }

    let attenuation = (1.0 - (distance / light.range)).powi(2);
    let cone_weight = spot_cone_weight(
        light.direction,
        card_center - light.position,
        light.inner_angle_radians,
        light.outer_angle_radians,
    );
    if cone_weight <= 0.0 {
        return Vec3::ZERO;
    }

    let strength = light.intensity.max(0.0)
        * attenuation
        * cone_weight
        * double_sided_orientation(card_normal, to_light.normalize_or_zero());
    saturate_vec3(light.color) * strength
}

fn spot_cone_weight(
    light_direction: Vec3,
    to_card: Vec3,
    inner_angle: f32,
    outer_angle: f32,
) -> f32 {
    let light_direction = light_direction.normalize_or_zero();
    let to_card = to_card.normalize_or_zero();
    if light_direction == Vec3::ZERO || to_card == Vec3::ZERO {
        return 0.0;
    }

    let inner_cos = inner_angle.max(0.0).cos();
    let outer_cos = outer_angle.max(0.0).cos();
    let (start, end) = if inner_cos >= outer_cos {
        (outer_cos, inner_cos)
    } else {
        (inner_cos, outer_cos)
    };
    let alignment = light_direction.dot(to_card);
    if alignment <= start {
        0.0
    } else if alignment >= end {
        1.0
    } else {
        (alignment - start) / (end - start).max(f32::EPSILON)
    }
}

fn voxel_radiance_strength(radiance_rgb: [u8; 3]) -> f32 {
    (radiance_rgb[0] as f32 + radiance_rgb[1] as f32 + radiance_rgb[2] as f32) / 255.0
}

fn scene_bounds(cards: &[HybridGiCardDescriptor]) -> (Vec3, Vec3) {
    cards.iter().fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(min_bounds, max_bounds), card| {
            let radius = Vec3::splat(card.bounds_radius.max(0.0));
            (
                min_bounds.min(card.bounds_center - radius),
                max_bounds.max(card.bounds_center + radius),
            )
        },
    )
}
