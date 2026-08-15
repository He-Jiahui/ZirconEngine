use std::collections::{BTreeMap, BTreeSet};

use crate::hybrid_gi::{
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareVoxelClipmap, HybridGiResolveRuntime,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT, HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};
use zircon_runtime::core::math::Vec3;

use super::super::{declarations::HybridGiRuntimeProbeSceneData, HybridGiRuntimeState};

const SCENE_SURFACE_CACHE_IRRADIANCE_WEIGHT_SCALE: f32 = 0.58;
const SCENE_VOXEL_RT_WEIGHT_SCALE: f32 = 0.6;
const SCENE_SURFACE_CACHE_RT_WEIGHT_SCALE: f32 = 0.58;
const SCENE_RUNTIME_VOXEL_RADIANCE_CONFIDENCE_QUALITY: f32 = 1.0;
const SCENE_RUNTIME_SURFACE_CAPTURE_CONFIDENCE_QUALITY: f32 = 0.88;
const SCENE_RUNTIME_SURFACE_ATLAS_CONFIDENCE_QUALITY: f32 = 0.78;
const SCENE_RUNTIME_VOXEL_SPATIAL_CONFIDENCE_QUALITY: f32 = 0.58;
const SCENE_RUNTIME_DIRTY_SURFACE_CACHE_CONFIDENCE_FRESHNESS: f32 = 0.42;
const SCENE_RUNTIME_DIRTY_VOXEL_CLIPMAP_CONFIDENCE_FRESHNESS: f32 = 0.68;
const SCENE_RUNTIME_SURFACE_INVALIDATION_CONFIDENCE_FRESHNESS_FALLOFF: f32 = 0.92;
const SCENE_RUNTIME_VOXEL_INVALIDATION_CONFIDENCE_FRESHNESS_FALLOFF: f32 = 0.9;
const SCENE_RUNTIME_INVALIDATION_CONFIDENCE_FRESHNESS_MIN: f32 = 0.72;
const SIGNED_POSITION_SCALE: f32 = 64.0;
const SIGNED_POSITION_BIAS: i32 = 2048;
pub(super) const POSITIVE_RADIUS_SCALE: f32 = 96.0;

impl HybridGiRuntimeState {
    pub(super) fn scene_surface_cache_scene_truth_revision(&self) -> u32 {
        self.scene_representation().surface_cache().scene_revision()
    }

    pub(super) fn scene_voxel_scene_truth_revision(&self) -> u32 {
        self.scene_representation().voxel_scene().scene_revision()
    }

    pub(super) fn scene_surface_cache_irradiance_fallback(
        &self,
        probe_id: u32,
    ) -> Option<([u8; 4], f32, f32)> {
        let probe_scene_data = self.probe_scene_data().get(&probe_id)?;
        let probe_position = dequantize_probe_position(probe_scene_data);
        let probe_radius = dequantize_positive(probe_scene_data.radius_q(), POSITIVE_RADIUS_SCALE);
        let dirty_page_ids = self
            .scene_representation()
            .surface_cache()
            .dirty_page_ids_snapshot()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let invalidation_freshness = scene_invalidation_confidence_freshness(
            self.scene_representation()
                .surface_cache()
                .invalidated_page_count(),
            SCENE_RUNTIME_SURFACE_INVALIDATION_CONFIDENCE_FRESHNESS_FALLOFF,
        );
        let card_bounds_by_id = self.scene_representation().card_bounds_by_id();
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut weighted_confidence_quality = 0.0_f32;
        let mut weighted_confidence_freshness = 0.0_f32;

        for (
            page_id,
            owner_card_id,
            _atlas_slot_id,
            _capture_slot_id,
            atlas_sample_rgba,
            capture_sample_rgba,
        ) in self
            .scene_representation()
            .surface_cache()
            .page_contents_snapshot()
        {
            let Some((bounds_center, bounds_radius)) =
                card_bounds_by_id.get(&owner_card_id).copied()
            else {
                continue;
            };
            let Some((base_rgb, confidence_quality)) =
                preferred_surface_cache_sample_rgb_and_quality(
                    atlas_sample_rgba,
                    capture_sample_rgba,
                )
            else {
                continue;
            };
            let support = scene_surface_cache_entry_support(
                probe_position,
                probe_radius,
                bounds_center,
                bounds_radius,
            );
            if support <= f32::EPSILON {
                continue;
            }

            weighted_rgb[0] += base_rgb[0] * support;
            weighted_rgb[1] += base_rgb[1] * support;
            weighted_rgb[2] += base_rgb[2] * support;
            total_support += support;
            weighted_confidence_quality += confidence_quality * support;
            weighted_confidence_freshness +=
                surface_cache_page_confidence_freshness(page_id, &dirty_page_ids)
                    * invalidation_freshness
                    * support;
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some((
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    weighted_rgb[0] / total_support,
                    weighted_rgb[1] / total_support,
                    weighted_rgb[2] / total_support,
                ],
                (total_support * SCENE_SURFACE_CACHE_IRRADIANCE_WEIGHT_SCALE).clamp(0.18, 0.62),
            ),
            (weighted_confidence_quality / total_support).clamp(0.0, 1.0),
            (weighted_confidence_freshness / total_support).clamp(0.0, 1.0),
        ))
    }

    pub(super) fn scene_voxel_rt_lighting_fallback(
        &self,
        probe_id: u32,
    ) -> Option<([u8; 4], f32, f32, u32)> {
        let probe_scene_data = self.probe_scene_data().get(&probe_id)?;
        let probe_position = dequantize_probe_position(probe_scene_data);
        let probe_radius = dequantize_positive(probe_scene_data.radius_q(), POSITIVE_RADIUS_SCALE);
        let dirty_page_ids = self
            .scene_representation()
            .surface_cache()
            .dirty_page_ids_snapshot()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let surface_invalidation_freshness = scene_invalidation_confidence_freshness(
            self.scene_representation()
                .surface_cache()
                .invalidated_page_count(),
            SCENE_RUNTIME_SURFACE_INVALIDATION_CONFIDENCE_FRESHNESS_FALLOFF,
        );
        let dirty_clipmap_ids = self
            .scene_representation()
            .voxel_scene()
            .dirty_clipmap_ids_snapshot()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let voxel_invalidation_freshness = scene_invalidation_confidence_freshness(
            self.scene_representation()
                .voxel_scene()
                .invalidated_clipmap_count(),
            SCENE_RUNTIME_VOXEL_INVALIDATION_CONFIDENCE_FRESHNESS_FALLOFF,
        );
        let clipmaps_by_id = self
            .scene_representation()
            .voxel_scene()
            .clipmap_descriptors_snapshot()
            .into_iter()
            .map(|(clipmap_id, center, half_extent)| {
                (
                    clipmap_id,
                    HybridGiPrepareVoxelClipmap {
                        clipmap_id,
                        center,
                        half_extent,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let surface_cache_page_contents = self
            .scene_representation()
            .surface_cache()
            .page_contents_snapshot();
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut weighted_confidence_quality = 0.0_f32;
        let mut weighted_confidence_freshness = 0.0_f32;

        for cell in self
            .scene_representation()
            .voxel_scene()
            .voxel_cells_snapshot()
        {
            if cell.occupancy_count == 0 {
                continue;
            }
            let Some(clipmap) = clipmaps_by_id.get(&cell.clipmap_id) else {
                continue;
            };
            let cell_index = cell.cell_index as usize;
            if cell_index >= HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT {
                continue;
            }
            let cell_x = cell_index % HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION;
            let cell_y = (cell_index / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION)
                % HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION;
            let cell_z = cell_index
                / (HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION
                    * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION);
            let cell_center = hybrid_gi_voxel_clipmap_cell_center(clipmap, cell_x, cell_y, cell_z);
            let cell_half_extent =
                (clipmap.half_extent / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION as f32).max(0.05);
            let support = scene_voxel_cell_support(
                probe_position,
                probe_radius,
                cell_center,
                cell_half_extent,
                cell.occupancy_count,
            );
            if support <= f32::EPSILON {
                continue;
            }

            let clipmap_freshness =
                voxel_clipmap_confidence_freshness(cell.clipmap_id, &dirty_clipmap_ids)
                    * voxel_invalidation_freshness;
            let (base_rgb, confidence_quality, confidence_freshness) = if cell.radiance_present {
                (
                    quantized_rgb_to_unit(cell.radiance_rgb),
                    SCENE_RUNTIME_VOXEL_RADIANCE_CONFIDENCE_QUALITY,
                    clipmap_freshness,
                )
            } else if let Some((owner_rgb, owner_confidence_quality, owner_confidence_freshness)) =
                scene_surface_cache_owner_rgb_quality_and_freshness(
                    &surface_cache_page_contents,
                    &dirty_page_ids,
                    surface_invalidation_freshness,
                    cell.dominant_card_id,
                )
            {
                (
                    owner_rgb,
                    owner_confidence_quality,
                    clipmap_freshness.min(owner_confidence_freshness),
                )
            } else {
                (
                    scene_voxel_cell_spatial_rgb(clipmap, cell_center, cell.occupancy_count),
                    SCENE_RUNTIME_VOXEL_SPATIAL_CONFIDENCE_QUALITY,
                    clipmap_freshness,
                )
            };
            weighted_rgb[0] += base_rgb[0] * support;
            weighted_rgb[1] += base_rgb[1] * support;
            weighted_rgb[2] += base_rgb[2] * support;
            total_support += support;
            weighted_confidence_quality += confidence_quality * support;
            weighted_confidence_freshness += confidence_freshness * support;
        }

        if total_support > f32::EPSILON {
            return Some((
                HybridGiResolveRuntime::pack_rgb_and_weight(
                    [
                        weighted_rgb[0] / total_support,
                        weighted_rgb[1] / total_support,
                        weighted_rgb[2] / total_support,
                    ],
                    (total_support * SCENE_VOXEL_RT_WEIGHT_SCALE).clamp(0.18, 0.7),
                ),
                (weighted_confidence_quality / total_support).clamp(0.0, 1.0),
                (weighted_confidence_freshness / total_support).clamp(0.0, 1.0),
                self.scene_voxel_scene_truth_revision(),
            ));
        }

        self.scene_surface_cache_rt_lighting_fallback(probe_position, probe_radius)
    }

    fn scene_surface_cache_rt_lighting_fallback(
        &self,
        probe_position: Vec3,
        probe_radius: f32,
    ) -> Option<([u8; 4], f32, f32, u32)> {
        let dirty_page_ids = self
            .scene_representation()
            .surface_cache()
            .dirty_page_ids_snapshot()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let invalidation_freshness = scene_invalidation_confidence_freshness(
            self.scene_representation()
                .surface_cache()
                .invalidated_page_count(),
            SCENE_RUNTIME_SURFACE_INVALIDATION_CONFIDENCE_FRESHNESS_FALLOFF,
        );
        let card_bounds_by_id = self.scene_representation().card_bounds_by_id();
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut weighted_confidence_quality = 0.0_f32;
        let mut weighted_confidence_freshness = 0.0_f32;

        for (
            page_id,
            owner_card_id,
            _atlas_slot_id,
            _capture_slot_id,
            atlas_sample_rgba,
            capture_sample_rgba,
        ) in self
            .scene_representation()
            .surface_cache()
            .page_contents_snapshot()
        {
            let Some((bounds_center, bounds_radius)) =
                card_bounds_by_id.get(&owner_card_id).copied()
            else {
                continue;
            };
            let Some((base_rgb, confidence_quality)) =
                preferred_surface_cache_sample_rgb_and_quality(
                    atlas_sample_rgba,
                    capture_sample_rgba,
                )
            else {
                continue;
            };
            let support = scene_surface_cache_entry_support(
                probe_position,
                probe_radius,
                bounds_center,
                bounds_radius,
            );
            if support <= f32::EPSILON {
                continue;
            }

            weighted_rgb[0] += base_rgb[0] * support;
            weighted_rgb[1] += base_rgb[1] * support;
            weighted_rgb[2] += base_rgb[2] * support;
            total_support += support;
            weighted_confidence_quality += confidence_quality * support;
            weighted_confidence_freshness +=
                surface_cache_page_confidence_freshness(page_id, &dirty_page_ids)
                    * invalidation_freshness
                    * support;
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some((
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    weighted_rgb[0] / total_support,
                    weighted_rgb[1] / total_support,
                    weighted_rgb[2] / total_support,
                ],
                (total_support * SCENE_SURFACE_CACHE_RT_WEIGHT_SCALE).clamp(0.18, 0.62),
            ),
            (weighted_confidence_quality / total_support).clamp(0.0, 1.0),
            (weighted_confidence_freshness / total_support).clamp(0.0, 1.0),
            self.scene_surface_cache_scene_truth_revision(),
        ))
    }
}

fn dequantize_probe_position(probe_scene_data: &HybridGiRuntimeProbeSceneData) -> Vec3 {
    Vec3::new(
        dequantize_signed(probe_scene_data.position_x_q()),
        dequantize_signed(probe_scene_data.position_y_q()),
        dequantize_signed(probe_scene_data.position_z_q()),
    )
}

fn dequantize_signed(value: u32) -> f32 {
    (value as i32 - SIGNED_POSITION_BIAS) as f32 / SIGNED_POSITION_SCALE
}

fn dequantize_positive(value: u32, scale: f32) -> f32 {
    value as f32 / scale
}

pub(super) fn quantize_signed(value: f32) -> u32 {
    ((value * SIGNED_POSITION_SCALE).round() as i32).wrapping_add(SIGNED_POSITION_BIAS) as u32
}

pub(super) fn quantize_positive(value: f32, scale: f32) -> u32 {
    (value.max(0.0) * scale).round() as u32
}

fn preferred_surface_cache_sample_rgb_and_quality(
    atlas_sample_rgba: [u8; 4],
    capture_sample_rgba: [u8; 4],
) -> Option<([f32; 3], f32)> {
    let preferred_rgba = if rgba_sample_is_present(capture_sample_rgba) {
        capture_sample_rgba
    } else if rgba_sample_is_present(atlas_sample_rgba) {
        atlas_sample_rgba
    } else {
        return None;
    };

    let confidence_quality = if rgba_sample_is_present(capture_sample_rgba) {
        SCENE_RUNTIME_SURFACE_CAPTURE_CONFIDENCE_QUALITY
    } else {
        SCENE_RUNTIME_SURFACE_ATLAS_CONFIDENCE_QUALITY
    };

    Some((
        [
            preferred_rgba[0] as f32 / 255.0,
            preferred_rgba[1] as f32 / 255.0,
            preferred_rgba[2] as f32 / 255.0,
        ],
        confidence_quality,
    ))
}

fn quantized_rgb_to_unit(rgb: [u8; 3]) -> [f32; 3] {
    [
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
    ]
}

fn scene_surface_cache_owner_rgb_quality_and_freshness(
    surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
    dirty_page_ids: &BTreeSet<u32>,
    invalidation_freshness: f32,
    owner_card_id: u32,
) -> Option<([f32; 3], f32, f32)> {
    surface_cache_page_contents.iter().find_map(
        |(
            page_id,
            candidate_owner_card_id,
            _atlas_slot_id,
            _capture_slot_id,
            atlas_sample_rgba,
            capture_sample_rgba,
        )| {
            (*candidate_owner_card_id == owner_card_id)
                .then(|| {
                    preferred_surface_cache_sample_rgb_and_quality(
                        *atlas_sample_rgba,
                        *capture_sample_rgba,
                    )
                    .map(|(rgb, quality)| {
                        (
                            rgb,
                            quality,
                            surface_cache_page_confidence_freshness(*page_id, dirty_page_ids)
                                * invalidation_freshness,
                        )
                    })
                })
                .flatten()
        },
    )
}

fn surface_cache_page_confidence_freshness(page_id: u32, dirty_page_ids: &BTreeSet<u32>) -> f32 {
    if dirty_page_ids.contains(&page_id) {
        SCENE_RUNTIME_DIRTY_SURFACE_CACHE_CONFIDENCE_FRESHNESS
    } else {
        1.0
    }
}

fn voxel_clipmap_confidence_freshness(clipmap_id: u32, dirty_clipmap_ids: &BTreeSet<u32>) -> f32 {
    if dirty_clipmap_ids.contains(&clipmap_id) {
        SCENE_RUNTIME_DIRTY_VOXEL_CLIPMAP_CONFIDENCE_FRESHNESS
    } else {
        1.0
    }
}

fn scene_invalidation_confidence_freshness(count: usize, falloff: f32) -> f32 {
    if count == 0 {
        return 1.0;
    }

    falloff
        .powi(count.min(4) as i32)
        .clamp(SCENE_RUNTIME_INVALIDATION_CONFIDENCE_FRESHNESS_MIN, 1.0)
}

fn scene_voxel_cell_support(
    probe_position: Vec3,
    probe_radius: f32,
    cell_center: Vec3,
    cell_half_extent: f32,
    occupancy_count: u32,
) -> f32 {
    let reach = (probe_radius.max(0.05) + cell_half_extent * 2.5).max(0.05);
    let falloff = (1.0 - probe_position.distance(cell_center) / reach).max(0.0);
    if falloff <= f32::EPSILON {
        return 0.0;
    }

    let occupancy_support = (occupancy_count.min(8) as f32 / 8.0).max(0.125);
    falloff * (0.18 + occupancy_support * 0.82)
}

fn scene_voxel_cell_spatial_rgb(
    clipmap: &HybridGiPrepareVoxelClipmap,
    cell_center: Vec3,
    occupancy_count: u32,
) -> [f32; 3] {
    let normalized = if clipmap.half_extent > f32::EPSILON {
        (cell_center - clipmap.center) / clipmap.half_extent
    } else {
        Vec3::ZERO
    };
    let warm_bias = (-normalized.x).max(0.0) * 0.55 + (-normalized.z).max(0.0) * 0.45;
    let cool_bias = normalized.x.max(0.0) * 0.55 + normalized.z.max(0.0) * 0.45;
    let vertical_bias = (1.0 - normalized.y.abs()).clamp(0.0, 1.0);
    let occupancy_bias = occupancy_count.min(8) as f32 / 8.0;

    [
        (0.14 + warm_bias * 0.62 + occupancy_bias * 0.14).clamp(0.0, 1.0),
        (0.12 + vertical_bias * 0.28 + occupancy_bias * 0.1).clamp(0.0, 1.0),
        (0.14 + cool_bias * 0.62 + occupancy_bias * 0.14).clamp(0.0, 1.0),
    ]
}

fn scene_surface_cache_entry_support(
    probe_position: Vec3,
    probe_radius: f32,
    bounds_center: Vec3,
    bounds_radius: f32,
) -> f32 {
    let reach = (probe_radius.max(0.05) + bounds_radius.max(0.05) * 2.25).max(0.05);
    let falloff = (1.0 - probe_position.distance(bounds_center) / reach).max(0.0);
    if falloff <= f32::EPSILON {
        return 0.0;
    }

    let bounds_support = (bounds_radius / reach).clamp(0.0, 1.0);
    falloff * (0.28 + bounds_support * 0.72)
}

fn rgba_sample_is_present(rgba: [u8; 4]) -> bool {
    rgba[3] > 0
}
