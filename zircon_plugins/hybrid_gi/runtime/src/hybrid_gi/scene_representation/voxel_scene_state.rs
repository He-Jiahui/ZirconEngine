use std::collections::{BTreeMap, BTreeSet};

use crate::hybrid_gi::{
    hybrid_gi_voxel_clipmap_bounds_cell_ranges, hybrid_gi_voxel_clipmap_cell_bit_index,
    HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap, HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
};
use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use zircon_runtime::core::math::Vec3;

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
    scene_prepare_voxel_cell_overrides: Vec<HybridGiPrepareVoxelCell>,
    scene_revision: u32,
}

impl HybridGiVoxelSceneState {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::hybrid_gi::scene_representation) fn synchronize(
        &mut self,
        cards: &[HybridGiCardDescriptor],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
        surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
        dirty_page_ids: &[u32],
        clipmap_budget: usize,
        scene_changed: bool,
    ) {
        let previous_resident_clipmap_ids = self.resident_clipmap_ids.clone();
        let previous_dirty_clipmap_ids = self.dirty_clipmap_ids.clone();
        let previous_invalidated_clipmap_ids = self.invalidated_clipmap_ids.clone();
        let previous_clipmap_descriptors = self.clipmap_descriptors.clone();
        let previous_voxel_cells = self.voxel_cells.clone();
        let previous_scene_prepare_voxel_cell_overrides =
            self.scene_prepare_voxel_cell_overrides.clone();
        let (resident_clipmap_ids, clipmap_descriptors) =
            build_clipmap_descriptors(cards, clipmap_budget);
        let mut voxel_cells = build_voxel_cells(
            cards,
            directional_lights,
            point_lights,
            spot_lights,
            &clipmap_descriptors,
        );
        apply_surface_cache_page_contents_to_voxel_cells(
            &mut voxel_cells,
            surface_cache_page_contents,
            dirty_page_ids,
        );
        let resident_clipmap_set = resident_clipmap_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let clipmaps_changed = self.resident_clipmap_ids != resident_clipmap_ids
            || self.clipmap_descriptors != clipmap_descriptors;
        if scene_changed || clipmaps_changed {
            self.scene_prepare_voxel_cell_overrides.clear();
        }
        merge_scene_prepare_voxel_cells(&mut voxel_cells, &self.scene_prepare_voxel_cell_overrides);

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
        self.bump_scene_revision_if(
            scene_changed
                || self.resident_clipmap_ids != previous_resident_clipmap_ids
                || self.dirty_clipmap_ids != previous_dirty_clipmap_ids
                || self.invalidated_clipmap_ids != previous_invalidated_clipmap_ids
                || self.clipmap_descriptors != previous_clipmap_descriptors
                || self.voxel_cells != previous_voxel_cells
                || self.scene_prepare_voxel_cell_overrides
                    != previous_scene_prepare_voxel_cell_overrides,
        );
    }

    pub(crate) fn apply_surface_cache_page_contents(
        &mut self,
        surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
    ) {
        let previous_voxel_cells = self.voxel_cells.clone();
        apply_surface_cache_page_contents_to_voxel_cells(
            &mut self.voxel_cells,
            surface_cache_page_contents,
            &[],
        );
        self.bump_scene_revision_if(self.voxel_cells != previous_voxel_cells);
    }

    pub(crate) fn apply_scene_prepare_voxel_cells(
        &mut self,
        readback_voxel_cells: &[HybridGiPrepareVoxelCell],
    ) {
        if readback_voxel_cells.is_empty() {
            return;
        }

        let previous_voxel_cells = self.voxel_cells.clone();
        let previous_scene_prepare_voxel_cell_overrides =
            self.scene_prepare_voxel_cell_overrides.clone();
        self.scene_prepare_voxel_cell_overrides = readback_voxel_cells.to_vec();
        merge_scene_prepare_voxel_cells(&mut self.voxel_cells, readback_voxel_cells);
        self.bump_scene_revision_if(
            self.voxel_cells != previous_voxel_cells
                || self.scene_prepare_voxel_cell_overrides
                    != previous_scene_prepare_voxel_cell_overrides,
        );
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

    pub(crate) fn dirty_clipmap_ids_snapshot(&self) -> Vec<u32> {
        self.dirty_clipmap_ids.clone()
    }

    #[cfg(test)]
    pub(crate) fn dirty_clipmap_ids(&self) -> Vec<u32> {
        self.dirty_clipmap_ids_snapshot()
    }

    pub(crate) fn invalidated_clipmap_count(&self) -> usize {
        self.invalidated_clipmap_ids.len()
    }

    #[cfg(test)]
    pub(crate) fn invalidated_clipmap_ids_snapshot(&self) -> Vec<u32> {
        self.invalidated_clipmap_ids.clone()
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

    pub(crate) fn scene_revision(&self) -> u32 {
        self.scene_revision
    }

    #[cfg(test)]
    pub(crate) fn invalidated_clipmap_ids(&self) -> Vec<u32> {
        self.invalidated_clipmap_ids_snapshot()
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

    fn bump_scene_revision_if(&mut self, changed: bool) {
        if changed {
            self.scene_revision = self.scene_revision.wrapping_add(1);
        }
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
                        card.bounds_center(),
                        card.bounds_radius(),
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
                                    && card.card_id() > dominant_card_ids[cell_index]);
                            if should_replace {
                                dominant_card_ids[cell_index] = card.card_id();
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

fn apply_surface_cache_page_contents_to_voxel_cells(
    voxel_cells: &mut [HybridGiPrepareVoxelCell],
    surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
    excluded_page_ids: &[u32],
) {
    let excluded_page_ids = excluded_page_ids.iter().copied().collect::<BTreeSet<_>>();
    let surface_cache_rgb_by_owner_card_id = surface_cache_page_contents
        .iter()
        .filter(|(page_id, _, _, _, _, _)| !excluded_page_ids.contains(page_id))
        .filter_map(
            |(
                _page_id,
                owner_card_id,
                _atlas_slot_id,
                _capture_slot_id,
                atlas_sample_rgba,
                capture_sample_rgba,
            )| {
                let capture_present = capture_sample_rgba[3] > 0;
                let atlas_present = atlas_sample_rgba[3] > 0;
                if !capture_present && !atlas_present {
                    return None;
                }

                let preferred_sample_rgba = if capture_present {
                    *capture_sample_rgba
                } else {
                    *atlas_sample_rgba
                };

                Some((
                    *owner_card_id,
                    [
                        preferred_sample_rgba[0],
                        preferred_sample_rgba[1],
                        preferred_sample_rgba[2],
                    ],
                ))
            },
        )
        .collect::<BTreeMap<_, _>>();

    for cell in voxel_cells {
        if cell.occupancy_count == 0 || cell.dominant_card_id == 0 {
            continue;
        }
        let Some(surface_cache_rgb) = surface_cache_rgb_by_owner_card_id
            .get(&cell.dominant_card_id)
            .copied()
        else {
            continue;
        };
        cell.radiance_present = true;
        cell.radiance_rgb = surface_cache_rgb;
    }
}

fn merge_scene_prepare_voxel_cells(
    voxel_cells: &mut Vec<HybridGiPrepareVoxelCell>,
    readback_voxel_cells: &[HybridGiPrepareVoxelCell],
) {
    if readback_voxel_cells.is_empty() {
        return;
    }

    let mut cells_by_key = voxel_cells
        .iter()
        .copied()
        .map(|cell| ((cell.clipmap_id, cell.cell_index), cell))
        .collect::<BTreeMap<_, _>>();
    for cell in readback_voxel_cells {
        cells_by_key.insert((cell.clipmap_id, cell.cell_index), *cell);
    }
    *voxel_cells = cells_by_key.into_values().collect();
}

fn card_voxel_radiance_rgb(
    card: &HybridGiCardDescriptor,
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
) -> [u8; 3] {
    let tint = saturate_vec3(Vec3::new(
        card.mesh().tint.x,
        card.mesh().tint.y,
        card.mesh().tint.z,
    ));
    let card_normal = card_normal(card);
    let direct_light = directional_lights.iter().fold(Vec3::ZERO, |acc, light| {
        acc + directional_light_contribution(card_normal, light)
    }) + point_lights.iter().fold(Vec3::ZERO, |acc, light| {
        acc + point_light_contribution(card.bounds_center(), card_normal, light)
    }) + spot_lights.iter().fold(Vec3::ZERO, |acc, light| {
        acc + spot_light_contribution(card.bounds_center(), card_normal, light)
    });
    let radiance = tint * 0.45 + component_mul(tint, direct_light * 0.9);
    quantize_voxel_radiance(radiance.max(Vec3::ZERO))
}

fn card_normal(card: &HybridGiCardDescriptor) -> Vec3 {
    let normal = card.mesh().transform.forward();
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
            let radius = Vec3::splat(card.bounds_radius().max(0.0));
            (
                min_bounds.min(card.bounds_center() - radius),
                max_bounds.max(card.bounds_center() + radius),
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::framework::render::RenderMeshSnapshot;
    use zircon_runtime::core::framework::scene::Mobility;
    use zircon_runtime::core::math::{Transform, Vec4};
    use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

    #[test]
    fn persisted_surface_cache_page_samples_override_voxel_cells_by_owner_card_id_not_page_id() {
        let mut voxel_cells = vec![HybridGiPrepareVoxelCell {
            clipmap_id: 0,
            cell_index: 7,
            occupancy_count: 1,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        }];

        apply_surface_cache_page_contents_to_voxel_cells(
            &mut voxel_cells,
            &[(21, 11, 0, 0, [10, 20, 30, 255], [40, 50, 60, 255])],
            &[],
        );

        assert_eq!(voxel_cells[0].radiance_rgb, [40, 50, 60]);
        assert!(
            voxel_cells[0].radiance_present,
            "expected voxel radiance reuse to match the persisted owner card id even when the persisted page id differs"
        );
    }

    #[test]
    fn scene_prepare_voxel_cell_readback_merges_into_voxel_cells() {
        let mut state = HybridGiVoxelSceneState::default();
        let readback_cell = HybridGiPrepareVoxelCell {
            clipmap_id: 2,
            cell_index: 5,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: true,
            radiance_rgb: [32, 48, 64],
        };

        state.apply_scene_prepare_voxel_cells(&[readback_cell]);

        assert_eq!(state.voxel_cells_snapshot(), vec![readback_cell]);
        assert_eq!(state.scene_revision(), 1);
    }

    #[test]
    fn scene_prepare_voxel_cell_readback_survives_stable_scene_synchronization() {
        let mut state = HybridGiVoxelSceneState::default();
        let cards = vec![card_descriptor(11, Vec3::ZERO)];
        state.synchronize(&cards, &[], &[], &[], &[], &[], 1, true);
        let base_cell = state
            .voxel_cells_snapshot()
            .into_iter()
            .find(|cell| cell.occupancy_count > 0)
            .expect("card should occupy at least one voxel cell");
        let readback_cell = HybridGiPrepareVoxelCell {
            radiance_present: true,
            radiance_rgb: [96, 48, 24],
            ..base_cell
        };

        state.apply_scene_prepare_voxel_cells(&[readback_cell]);
        state.synchronize(&cards, &[], &[], &[], &[], &[], 1, false);

        let persisted = state
            .voxel_cells_snapshot()
            .into_iter()
            .find(|cell| {
                cell.clipmap_id == readback_cell.clipmap_id
                    && cell.cell_index == readback_cell.cell_index
            })
            .expect("stable scene sync should keep the readback cell key");
        assert_eq!(persisted, readback_cell);
    }

    fn card_descriptor(card_id: u32, center: Vec3) -> HybridGiCardDescriptor {
        HybridGiCardDescriptor {
            card_id,
            mesh: RenderMeshSnapshot {
                node_id: card_id as u64,
                transform: Transform::from_translation(center).with_scale(Vec3::splat(2.0)),
                model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                    "res://models/hgi-card.obj",
                )),
                material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                    "res://materials/hgi-card.mat",
                )),
                tint: Vec4::ONE,
                mobility: Mobility::Static,
                render_layer_mask: u32::MAX,
            },
            bounds_center: center,
            bounds_radius: 1.0,
        }
    }
}
