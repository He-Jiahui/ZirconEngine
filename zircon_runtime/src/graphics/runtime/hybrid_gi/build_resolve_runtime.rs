use std::collections::{BTreeMap, BTreeSet};

use crate::core::math::Vec3;
use crate::graphics::types::{
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareVoxelClipmap,
    HybridGiResolveProbeSceneData, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT, HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};

use super::HybridGiRuntimeState;

const CHILD_SPECIFICITY_BOOST: f32 = 0.3;
const RESIDENT_CHILD_ATTENUATION: f32 = 0.78;
const FARTHER_ANCESTOR_BUDGET_FALLOFF: f32 = 0.72;
const FARTHER_ANCESTOR_BUDGET_SCALE: f32 = 0.6;
const FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF: f32 = 0.72;
const IRRADIANCE_INHERITANCE_WEIGHT_SCALE: f32 = 0.5;
const DIRECT_LINEAGE_IRRADIANCE_WEIGHT_SCALE: f32 = 0.45;
const DESCENDANT_LINEAGE_IRRADIANCE_FALLOFF: f32 = 0.84;
const ANCESTOR_TRACE_INHERITANCE_FALLOFF: f32 = 0.72;
const TRACE_INHERITANCE_WEIGHT_SCALE: f32 = 0.45;
const DIRECT_LINEAGE_TRACE_WEIGHT_SCALE: f32 = 0.45;
const DESCENDANT_LINEAGE_TRACE_FALLOFF: f32 = 0.84;
const STANDALONE_DIRECT_TRACE_WEIGHT_SCALE: f32 = 0.35;
const SCENE_TRACE_RESOLVE_SCALE: f32 = 0.35;
const SCENE_TRACE_RT_SCALE: f32 = 0.25;
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
const POSITIVE_RADIUS_SCALE: f32 = 96.0;

impl HybridGiRuntimeState {
    pub(crate) fn build_resolve_runtime(&self) -> HybridGiResolveRuntime {
        let tracked_probe_ids = self.tracked_runtime_probe_ids();
        let mut probe_hierarchy_irradiance_rgb_and_weight = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_irradiance_ids = BTreeSet::new();
        let mut probe_scene_driven_hierarchy_irradiance_quality_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_irradiance_freshness_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_irradiance_revision = BTreeMap::new();
        let mut probe_hierarchy_rt_lighting_rgb_and_weight = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_ids = BTreeSet::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_quality_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_freshness_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_revision = BTreeMap::new();

        for &probe_id in &tracked_probe_ids {
            let Some((
                encoded,
                scene_driven,
                scene_truth_quality,
                scene_truth_freshness,
                scene_truth_revision,
            )) = self.runtime_hierarchy_irradiance_entry(probe_id)
            else {
                continue;
            };
            if scene_driven {
                probe_scene_driven_hierarchy_irradiance_ids.insert(probe_id);
                probe_scene_driven_hierarchy_irradiance_quality_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_quality_q8(scene_truth_quality),
                );
                probe_scene_driven_hierarchy_irradiance_freshness_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_freshness_q8(scene_truth_freshness),
                );
                probe_scene_driven_hierarchy_irradiance_revision
                    .insert(probe_id, scene_truth_revision);
            }
            probe_hierarchy_irradiance_rgb_and_weight.insert(probe_id, encoded);
        }
        for &probe_id in &tracked_probe_ids {
            let Some((
                encoded,
                scene_driven,
                scene_truth_quality,
                scene_truth_freshness,
                scene_truth_revision,
            )) = self.runtime_hierarchy_rt_lighting_entry(probe_id)
            else {
                continue;
            };
            if scene_driven {
                probe_scene_driven_hierarchy_rt_lighting_ids.insert(probe_id);
                probe_scene_driven_hierarchy_rt_lighting_quality_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_quality_q8(scene_truth_quality),
                );
                probe_scene_driven_hierarchy_rt_lighting_freshness_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_freshness_q8(scene_truth_freshness),
                );
                probe_scene_driven_hierarchy_rt_lighting_revision
                    .insert(probe_id, scene_truth_revision);
            }
            probe_hierarchy_rt_lighting_rgb_and_weight.insert(probe_id, encoded);
        }

        let probe_scene_data = self
            .probe_scene_data()
            .iter()
            .filter_map(|(&probe_id, scene_data)| {
                tracked_probe_ids.contains(&probe_id).then_some((
                    probe_id,
                    HybridGiResolveProbeSceneData::new(
                        scene_data.position_x_q(),
                        scene_data.position_y_q(),
                        scene_data.position_z_q(),
                        scene_data.radius_q(),
                    ),
                ))
            })
            .collect();
        let trace_region_scene_data = self
            .scheduled_trace_region_ids()
            .iter()
            .filter_map(|&region_id| {
                self.trace_region_scene_data()
                    .get(&region_id)
                    .map(|scene_data| {
                        (
                            region_id,
                            HybridGiResolveTraceRegionSceneData::new(
                                scene_data.center_x_q(),
                                scene_data.center_y_q(),
                                scene_data.center_z_q(),
                                scene_data.radius_q(),
                                scene_data.coverage_q(),
                                scene_data.rt_lighting_rgb(),
                            ),
                        )
                    })
            })
            .collect();
        let probe_parent_probes = self
            .probe_parent_probes()
            .iter()
            .filter_map(|(&probe_id, &parent_probe_id)| {
                (tracked_probe_ids.contains(&probe_id)
                    && tracked_probe_ids.contains(&parent_probe_id))
                .then_some((probe_id, parent_probe_id))
            })
            .collect();
        let probe_rt_lighting_rgb = self
            .probe_rt_lighting_rgb()
            .iter()
            .filter_map(|(&probe_id, &rt_lighting_rgb)| {
                tracked_probe_ids
                    .contains(&probe_id)
                    .then_some((probe_id, rt_lighting_rgb))
            })
            .collect::<BTreeMap<_, _>>();
        let probe_hierarchy_resolve_weight_q8 = tracked_probe_ids
            .iter()
            .map(|&probe_id| {
                (
                    probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(
                        self.runtime_hierarchy_resolve_weight(probe_id),
                    ),
                )
            })
            .collect();

        HybridGiResolveRuntime::new(
            probe_scene_data,
            trace_region_scene_data,
            probe_parent_probes,
            probe_rt_lighting_rgb,
            probe_hierarchy_resolve_weight_q8,
            probe_hierarchy_irradiance_rgb_and_weight,
            probe_hierarchy_rt_lighting_rgb_and_weight,
            probe_scene_driven_hierarchy_irradiance_ids,
            probe_scene_driven_hierarchy_rt_lighting_ids,
            probe_scene_driven_hierarchy_irradiance_quality_q8,
            probe_scene_driven_hierarchy_irradiance_freshness_q8,
            probe_scene_driven_hierarchy_irradiance_revision,
            probe_scene_driven_hierarchy_rt_lighting_quality_q8,
            probe_scene_driven_hierarchy_rt_lighting_freshness_q8,
            probe_scene_driven_hierarchy_rt_lighting_revision,
        )
    }

    fn tracked_runtime_probe_ids(&self) -> Vec<u32> {
        let seed_probe_ids = self
            .resident_probe_ids()
            .chain(self.pending_probe_ids())
            .chain(
                self.pending_update_requests()
                    .iter()
                    .map(|update| update.probe_id()),
            )
            .chain(self.current_requested_probe_ids().iter().copied())
            .chain(
                self.scene_representation_owns_runtime()
                    .then_some(self.probe_scene_data().keys().copied())
                    .into_iter()
                    .flatten(),
            )
            .collect::<BTreeSet<_>>();
        let mut tracked_probe_ids = seed_probe_ids.clone();
        for probe_id in seed_probe_ids {
            let mut current_probe_id = probe_id;
            let mut visited_probe_ids = BTreeSet::from([probe_id]);
            while let Some(parent_probe_id) =
                self.probe_parent_probes().get(&current_probe_id).copied()
            {
                if !visited_probe_ids.insert(parent_probe_id) {
                    break;
                }
                tracked_probe_ids.insert(parent_probe_id);
                current_probe_id = parent_probe_id;
            }
        }
        tracked_probe_ids.into_iter().collect()
    }

    fn runtime_hierarchy_resolve_weight(&self, probe_id: u32) -> f32 {
        let resident_child_count = self.resident_descendant_count(probe_id);
        let resident_parent_depth = self.resident_parent_depth(probe_id);
        let farther_ancestor_budget_support =
            self.farther_resident_ancestor_budget_support(probe_id);
        let scene_trace_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);

        let specificity_weight = 1.0 + resident_parent_depth as f32 * CHILD_SPECIFICITY_BOOST;
        let attenuation_weight = if resident_child_count == 0 {
            1.0
        } else {
            RESIDENT_CHILD_ATTENUATION.powi(resident_child_count as i32)
        };
        let lineage_budget_weight =
            1.0 + farther_ancestor_budget_support * FARTHER_ANCESTOR_BUDGET_SCALE;
        let scene_trace_weight = 1.0 + scene_trace_support * SCENE_TRACE_RESOLVE_SCALE;
        (specificity_weight * attenuation_weight * lineage_budget_weight * scene_trace_weight)
            .clamp(0.25, 2.75)
    }

    fn scene_surface_cache_scene_truth_revision(&self) -> u32 {
        self.scene_representation().surface_cache().scene_revision()
    }

    fn scene_voxel_scene_truth_revision(&self) -> u32 {
        self.scene_representation().voxel_scene().scene_revision()
    }

    fn runtime_hierarchy_irradiance_entry(
        &self,
        probe_id: u32,
    ) -> Option<([u8; 4], bool, f32, f32, u32)> {
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut current_probe_id = probe_id;
        let mut resident_ancestor_count = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some(parent_probe_id) = self.probe_parent_probes().get(&current_probe_id).copied()
        {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.has_resident_probe(parent_probe_id) {
                resident_ancestor_count += 1;
                if resident_ancestor_count > 1 {
                    let Some(ancestor_irradiance_rgb) =
                        self.probe_irradiance_rgb().get(&parent_probe_id).copied()
                    else {
                        current_probe_id = parent_probe_id;
                        continue;
                    };
                    let farther_ancestor_depth = resident_ancestor_count - 2;
                    let support = FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF
                        .powi(farther_ancestor_depth as i32)
                        * budget_weight(
                            self.probe_ray_budgets()
                                .get(&parent_probe_id)
                                .copied()
                                .unwrap_or_default(),
                        );
                    if support > 0.0 {
                        weighted_rgb[0] += ancestor_irradiance_rgb[0] as f32 / 255.0 * support;
                        weighted_rgb[1] += ancestor_irradiance_rgb[1] as f32 / 255.0 * support;
                        weighted_rgb[2] += ancestor_irradiance_rgb[2] as f32 / 255.0 * support;
                        total_support += support;
                    }
                }
            }
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            if let Some(encoded) = self
                .direct_lineage_irradiance_fallback(probe_id)
                .or_else(|| self.descendant_lineage_irradiance_fallback(probe_id))
            {
                return Some((encoded, false, 1.0, 1.0, 0));
            }

            if !self.has_current_lineage_trace_support(probe_id) {
                return self.scene_surface_cache_irradiance_fallback(probe_id).map(
                    |(encoded, scene_truth_quality, scene_truth_freshness)| {
                        (
                            encoded,
                            true,
                            scene_truth_quality,
                            scene_truth_freshness,
                            self.scene_surface_cache_scene_truth_revision(),
                        )
                    },
                );
            }

            return None;
        }

        Some((
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    weighted_rgb[0] / total_support,
                    weighted_rgb[1] / total_support,
                    weighted_rgb[2] / total_support,
                ],
                (total_support * IRRADIANCE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
            ),
            false,
            1.0,
            1.0,
            0,
        ))
    }

    fn runtime_hierarchy_rt_lighting_entry(
        &self,
        probe_id: u32,
    ) -> Option<([u8; 4], bool, f32, f32, u32)> {
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut current_probe_id = probe_id;
        let mut ancestor_depth = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        let scene_trace_weight = 1.0
            + self
                .effective_lineage_trace_support_score(probe_id)
                .clamp(0.0, 1.5)
                * SCENE_TRACE_RT_SCALE;

        while let Some(parent_probe_id) = self.probe_parent_probes().get(&current_probe_id).copied()
        {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.has_resident_probe(parent_probe_id) {
                let Some(ancestor_rt_lighting_rgb) =
                    self.probe_rt_lighting_rgb().get(&parent_probe_id).copied()
                else {
                    current_probe_id = parent_probe_id;
                    continue;
                };
                let resident_budget_weight = budget_weight(
                    self.probe_ray_budgets()
                        .get(&parent_probe_id)
                        .copied()
                        .unwrap_or_default(),
                );
                if resident_budget_weight > f32::EPSILON {
                    let hierarchy_weight =
                        ANCESTOR_TRACE_INHERITANCE_FALLOFF.powi(ancestor_depth as i32);
                    let intensity_weight = runtime_rgb_intensity(ancestor_rt_lighting_rgb);
                    let support = hierarchy_weight
                        * resident_budget_weight
                        * intensity_weight
                        * scene_trace_weight;
                    if support > 0.0 {
                        weighted_rgb[0] += ancestor_rt_lighting_rgb[0] as f32 / 255.0 * support;
                        weighted_rgb[1] += ancestor_rt_lighting_rgb[1] as f32 / 255.0 * support;
                        weighted_rgb[2] += ancestor_rt_lighting_rgb[2] as f32 / 255.0 * support;
                        total_support += support;
                    }
                }
                ancestor_depth += 1;
            }
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            if let Some(encoded) = self
                .direct_lineage_rt_lighting_fallback(probe_id)
                .or_else(|| self.descendant_lineage_rt_lighting_fallback(probe_id))
            {
                return Some((encoded, false, 1.0, 1.0, 0));
            }

            if !self.has_current_lineage_trace_support(probe_id) {
                return self.scene_voxel_rt_lighting_fallback(probe_id).map(
                    |(
                        encoded,
                        scene_truth_quality,
                        scene_truth_freshness,
                        scene_truth_revision,
                    )| {
                        (
                            encoded,
                            true,
                            scene_truth_quality,
                            scene_truth_freshness,
                            scene_truth_revision,
                        )
                    },
                );
            }

            return None;
        }

        Some((
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    weighted_rgb[0] / total_support,
                    weighted_rgb[1] / total_support,
                    weighted_rgb[2] / total_support,
                ],
                (total_support * TRACE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
            ),
            false,
            1.0,
            1.0,
            0,
        ))
    }

    fn direct_lineage_irradiance_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        if self.has_resident_probe(probe_id) {
            return None;
        }

        let direct_lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if direct_lineage_support <= f32::EPSILON {
            return self.standalone_direct_rt_lighting_fallback(probe_id);
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;

        if let Some(direct_irradiance_rgb) = self.probe_irradiance_rgb().get(&probe_id).copied() {
            weighted_rgb[0] += direct_irradiance_rgb[0] as f32 / 255.0 * direct_lineage_support;
            weighted_rgb[1] += direct_irradiance_rgb[1] as f32 / 255.0 * direct_lineage_support;
            weighted_rgb[2] += direct_irradiance_rgb[2] as f32 / 255.0 * direct_lineage_support;
            total_support += direct_lineage_support;
        }

        let mut current_probe_id = probe_id;
        let mut ancestor_depth = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        while let Some(parent_probe_id) = self.probe_parent_probes().get(&current_probe_id).copied()
        {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if let Some(ancestor_irradiance_rgb) =
                self.probe_irradiance_rgb().get(&parent_probe_id).copied()
            {
                let ancestor_support = direct_lineage_support
                    * FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF.powi(ancestor_depth as i32)
                    * budget_weight(
                        self.probe_ray_budgets()
                            .get(&parent_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    );
                if ancestor_support > f32::EPSILON {
                    weighted_rgb[0] += ancestor_irradiance_rgb[0] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[1] += ancestor_irradiance_rgb[1] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[2] += ancestor_irradiance_rgb[2] as f32 / 255.0 * ancestor_support;
                    total_support += ancestor_support;
                }
            }
            ancestor_depth += 1;
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_IRRADIANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn direct_lineage_rt_lighting_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        if self.has_resident_probe(probe_id) {
            return None;
        }

        let direct_lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if direct_lineage_support <= f32::EPSILON {
            return None;
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;

        if let Some(direct_rt_lighting_rgb) = self.probe_rt_lighting_rgb().get(&probe_id).copied() {
            let direct_support = direct_lineage_support
                * budget_weight(
                    self.probe_ray_budgets()
                        .get(&probe_id)
                        .copied()
                        .unwrap_or_default(),
                )
                * runtime_rgb_intensity(direct_rt_lighting_rgb);
            if direct_support > f32::EPSILON {
                weighted_rgb[0] += direct_rt_lighting_rgb[0] as f32 / 255.0 * direct_support;
                weighted_rgb[1] += direct_rt_lighting_rgb[1] as f32 / 255.0 * direct_support;
                weighted_rgb[2] += direct_rt_lighting_rgb[2] as f32 / 255.0 * direct_support;
                total_support += direct_support;
            }
        }

        let mut current_probe_id = probe_id;
        let mut ancestor_depth = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        while let Some(parent_probe_id) = self.probe_parent_probes().get(&current_probe_id).copied()
        {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if let Some(ancestor_rt_lighting_rgb) =
                self.probe_rt_lighting_rgb().get(&parent_probe_id).copied()
            {
                let ancestor_support = direct_lineage_support
                    * ANCESTOR_TRACE_INHERITANCE_FALLOFF.powi(ancestor_depth as i32)
                    * budget_weight(
                        self.probe_ray_budgets()
                            .get(&parent_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    )
                    * runtime_rgb_intensity(ancestor_rt_lighting_rgb);
                if ancestor_support > f32::EPSILON {
                    weighted_rgb[0] +=
                        ancestor_rt_lighting_rgb[0] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[1] +=
                        ancestor_rt_lighting_rgb[1] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[2] +=
                        ancestor_rt_lighting_rgb[2] as f32 / 255.0 * ancestor_support;
                    total_support += ancestor_support;
                }
            }
            ancestor_depth += 1;
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_TRACE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn standalone_direct_rt_lighting_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        if self.has_resident_probe(probe_id) || self.probe_parent_probes().contains_key(&probe_id) {
            return None;
        }

        let direct_rt_lighting_rgb = self.probe_rt_lighting_rgb().get(&probe_id).copied()?;
        let support = budget_weight(
            self.probe_ray_budgets()
                .get(&probe_id)
                .copied()
                .unwrap_or_default(),
        ) * runtime_rgb_intensity(direct_rt_lighting_rgb);
        if support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                direct_rt_lighting_rgb[0] as f32 / 255.0,
                direct_rt_lighting_rgb[1] as f32 / 255.0,
                direct_rt_lighting_rgb[2] as f32 / 255.0,
            ],
            (support * STANDALONE_DIRECT_TRACE_WEIGHT_SCALE).clamp(0.05, 0.45),
        ))
    }

    fn descendant_lineage_irradiance_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        let lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if lineage_support <= f32::EPSILON {
            return None;
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut stack = self
            .probe_parent_probes()
            .iter()
            .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
                (parent_probe_id == probe_id).then_some((candidate_probe_id, 1usize))
            })
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::new();

        while let Some((candidate_probe_id, depth)) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }

            if let Some(descendant_irradiance_rgb) = self
                .probe_irradiance_rgb()
                .get(&candidate_probe_id)
                .copied()
            {
                let support = lineage_support
                    * DESCENDANT_LINEAGE_IRRADIANCE_FALLOFF.powi((depth - 1) as i32)
                    * budget_weight(
                        self.probe_ray_budgets()
                            .get(&candidate_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    );
                if support > f32::EPSILON {
                    weighted_rgb[0] += descendant_irradiance_rgb[0] as f32 / 255.0 * support;
                    weighted_rgb[1] += descendant_irradiance_rgb[1] as f32 / 255.0 * support;
                    weighted_rgb[2] += descendant_irradiance_rgb[2] as f32 / 255.0 * support;
                    total_support += support;
                }
            }

            stack.extend(self.probe_parent_probes().iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id)
                        .then_some((grandchild_probe_id, depth + 1))
                },
            ));
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_IRRADIANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn descendant_lineage_rt_lighting_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        let lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if lineage_support <= f32::EPSILON {
            return None;
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut stack = self
            .probe_parent_probes()
            .iter()
            .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
                (parent_probe_id == probe_id).then_some((candidate_probe_id, 1usize))
            })
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::new();

        while let Some((candidate_probe_id, depth)) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }

            if let Some(descendant_rt_lighting_rgb) = self
                .probe_rt_lighting_rgb()
                .get(&candidate_probe_id)
                .copied()
            {
                let support = lineage_support
                    * DESCENDANT_LINEAGE_TRACE_FALLOFF.powi((depth - 1) as i32)
                    * budget_weight(
                        self.probe_ray_budgets()
                            .get(&candidate_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    )
                    * runtime_rgb_intensity(descendant_rt_lighting_rgb);
                if support > f32::EPSILON {
                    weighted_rgb[0] += descendant_rt_lighting_rgb[0] as f32 / 255.0 * support;
                    weighted_rgb[1] += descendant_rt_lighting_rgb[1] as f32 / 255.0 * support;
                    weighted_rgb[2] += descendant_rt_lighting_rgb[2] as f32 / 255.0 * support;
                    total_support += support;
                }
            }

            stack.extend(self.probe_parent_probes().iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id)
                        .then_some((grandchild_probe_id, depth + 1))
                },
            ));
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_TRACE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn resident_descendant_count(&self, probe_id: u32) -> usize {
        let mut count = 0usize;
        let mut stack = self
            .probe_parent_probes()
            .iter()
            .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
                (parent_probe_id == probe_id).then_some(candidate_probe_id)
            })
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::new();

        while let Some(candidate_probe_id) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }
            if self.has_resident_probe(candidate_probe_id) {
                count += 1;
            }
            stack.extend(self.probe_parent_probes().iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id).then_some(grandchild_probe_id)
                },
            ));
        }

        count
    }

    fn resident_parent_depth(&self, probe_id: u32) -> usize {
        let mut depth = 0usize;
        let mut current_probe_id = probe_id;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some(parent_probe_id) = self.probe_parent_probes().get(&current_probe_id).copied()
        {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.has_resident_probe(parent_probe_id) {
                depth += 1;
            }
            current_probe_id = parent_probe_id;
        }

        depth
    }

    fn farther_resident_ancestor_budget_support(&self, probe_id: u32) -> f32 {
        let mut current_probe_id = probe_id;
        let mut resident_ancestor_count = 0usize;
        let mut total_support = 0.0_f32;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some(parent_probe_id) = self.probe_parent_probes().get(&current_probe_id).copied()
        {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.has_resident_probe(parent_probe_id) {
                resident_ancestor_count += 1;
                if resident_ancestor_count > 1 {
                    let farther_ancestor_depth = resident_ancestor_count - 2;
                    total_support += FARTHER_ANCESTOR_BUDGET_FALLOFF
                        .powi(farther_ancestor_depth as i32)
                        * budget_weight(
                            self.probe_ray_budgets()
                                .get(&parent_probe_id)
                                .copied()
                                .unwrap_or_default(),
                        );
                }
            }
            current_probe_id = parent_probe_id;
        }

        total_support.clamp(0.0, 1.5)
    }

    fn scene_surface_cache_irradiance_fallback(
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

    fn scene_voxel_rt_lighting_fallback(&self, probe_id: u32) -> Option<([u8; 4], f32, f32, u32)> {
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

fn budget_weight(ray_budget: u32) -> f32 {
    (ray_budget.min(256) as f32 / 256.0).clamp(0.0, 1.0)
}

fn runtime_rgb_intensity(rgb: [u8; 3]) -> f32 {
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]) as f32 / 255.0;
    max_component.clamp(0.0, 1.0)
}

fn dequantize_probe_position(
    probe_scene_data: &super::declarations::HybridGiRuntimeProbeSceneData,
) -> Vec3 {
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
