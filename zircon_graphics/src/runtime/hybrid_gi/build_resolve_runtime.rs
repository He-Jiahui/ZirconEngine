use std::collections::{BTreeMap, BTreeSet};

use crate::types::HybridGiResolveRuntime;

use super::HybridGiRuntimeState;

const CHILD_SPECIFICITY_BOOST: f32 = 0.3;
const RESIDENT_CHILD_ATTENUATION: f32 = 0.78;
const FARTHER_ANCESTOR_BUDGET_FALLOFF: f32 = 0.72;
const FARTHER_ANCESTOR_BUDGET_SCALE: f32 = 0.6;
const FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF: f32 = 0.72;
const IRRADIANCE_INHERITANCE_WEIGHT_SCALE: f32 = 0.5;
const ANCESTOR_TRACE_INHERITANCE_FALLOFF: f32 = 0.72;
const TRACE_INHERITANCE_WEIGHT_SCALE: f32 = 0.45;
const SCENE_TRACE_SUPPORT_FALLOFF: f32 = 0.78;
const SCENE_TRACE_RESOLVE_SCALE: f32 = 0.35;
const SCENE_TRACE_RT_SCALE: f32 = 0.25;

impl HybridGiRuntimeState {
    pub(crate) fn build_resolve_runtime(&self) -> HybridGiResolveRuntime {
        let resident_probe_ids = self.resident_slots.keys().copied().collect::<Vec<_>>();

        HybridGiResolveRuntime {
            probe_rt_lighting_rgb: self
                .probe_rt_lighting_rgb
                .iter()
                .filter_map(|(&probe_id, &rt_lighting_rgb)| {
                    self.resident_slots
                        .contains_key(&probe_id)
                        .then_some((probe_id, rt_lighting_rgb))
                })
                .collect::<BTreeMap<_, _>>(),
            probe_hierarchy_resolve_weight_q8: resident_probe_ids
                .iter()
                .map(|&probe_id| {
                    (
                        probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(
                            self.runtime_hierarchy_resolve_weight(probe_id),
                        ),
                    )
                })
                .collect(),
            probe_hierarchy_irradiance_rgb_and_weight: resident_probe_ids
                .iter()
                .filter_map(|&probe_id| {
                    self.runtime_hierarchy_irradiance(probe_id)
                        .map(|encoded| (probe_id, encoded))
                })
                .collect(),
            probe_hierarchy_rt_lighting_rgb_and_weight: resident_probe_ids
                .iter()
                .filter_map(|&probe_id| {
                    self.runtime_hierarchy_rt_lighting(probe_id)
                        .map(|encoded| (probe_id, encoded))
                })
                .collect(),
        }
    }

    fn runtime_hierarchy_resolve_weight(&self, probe_id: u32) -> f32 {
        let resident_child_count = self.resident_descendant_count(probe_id);
        let resident_parent_depth = self.resident_parent_depth(probe_id);
        let farther_ancestor_budget_support =
            self.farther_resident_ancestor_budget_support(probe_id);
        let scene_trace_support = self.scene_trace_lineage_support(probe_id);

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

    fn runtime_hierarchy_irradiance(&self, probe_id: u32) -> Option<[u8; 4]> {
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut current_probe_id = probe_id;
        let mut resident_ancestor_count = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.resident_slots.contains_key(&parent_probe_id) {
                resident_ancestor_count += 1;
                if resident_ancestor_count > 1 {
                    let Some(ancestor_irradiance_rgb) =
                        self.probe_irradiance_rgb.get(&parent_probe_id).copied()
                    else {
                        current_probe_id = parent_probe_id;
                        continue;
                    };
                    let farther_ancestor_depth = resident_ancestor_count - 2;
                    let support = FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF
                        .powi(farther_ancestor_depth as i32)
                        * budget_weight(
                            self.probe_ray_budgets
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
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * IRRADIANCE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn runtime_hierarchy_rt_lighting(&self, probe_id: u32) -> Option<[u8; 4]> {
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut current_probe_id = probe_id;
        let mut ancestor_depth = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        let scene_trace_weight =
            1.0 + self.scene_trace_lineage_support(probe_id) * SCENE_TRACE_RT_SCALE;

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.resident_slots.contains_key(&parent_probe_id) {
                let Some(ancestor_rt_lighting_rgb) =
                    self.probe_rt_lighting_rgb.get(&parent_probe_id).copied()
                else {
                    current_probe_id = parent_probe_id;
                    continue;
                };
                let resident_budget_weight = budget_weight(
                    self.probe_ray_budgets
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
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * TRACE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn resident_descendant_count(&self, probe_id: u32) -> usize {
        let mut count = 0usize;
        let mut stack = self
            .probe_parent_probes
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
            if self.resident_slots.contains_key(&candidate_probe_id) {
                count += 1;
            }
            stack.extend(self.probe_parent_probes.iter().filter_map(
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

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if self.resident_slots.contains_key(&parent_probe_id) {
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

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.resident_slots.contains_key(&parent_probe_id) {
                resident_ancestor_count += 1;
                if resident_ancestor_count > 1 {
                    let farther_ancestor_depth = resident_ancestor_count - 2;
                    total_support += FARTHER_ANCESTOR_BUDGET_FALLOFF
                        .powi(farther_ancestor_depth as i32)
                        * budget_weight(
                            self.probe_ray_budgets
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

    fn scene_trace_lineage_support(&self, probe_id: u32) -> f32 {
        let mut total_support = 0.0_f32;
        let mut lineage_weight = 1.0_f32;
        let mut current_probe_id = probe_id;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        loop {
            total_support +=
                self.single_probe_scene_trace_support(current_probe_id) * lineage_weight;
            let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied()
            else {
                break;
            };
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            lineage_weight *= SCENE_TRACE_SUPPORT_FALLOFF;
            current_probe_id = parent_probe_id;
        }

        total_support.clamp(0.0, 1.5)
    }

    fn single_probe_scene_trace_support(&self, probe_id: u32) -> f32 {
        let Some(probe) = self.probe_scene_data.get(&probe_id) else {
            return 0.0;
        };

        self.scheduled_trace_regions
            .iter()
            .filter_map(|region_id| self.trace_region_scene_data.get(region_id))
            .map(|region| {
                let reach = probe.radius_q.saturating_add(region.radius_q).max(1) as f32;
                let max_distance = (reach * 3.0).max(1.0);
                let distance_to_region = probe.position_x_q.abs_diff(region.center_x_q) as f32
                    + probe.position_y_q.abs_diff(region.center_y_q) as f32
                    + probe.position_z_q.abs_diff(region.center_z_q) as f32;
                if distance_to_region >= max_distance {
                    return 0.0;
                }

                let proximity = 1.0 - distance_to_region / max_distance;
                proximity * proximity * (region.coverage_q.min(255) as f32 / 128.0)
            })
            .sum()
    }
}

fn budget_weight(ray_budget: u32) -> f32 {
    (ray_budget.min(256) as f32 / 256.0).clamp(0.0, 1.0)
}

fn runtime_rgb_intensity(rgb: [u8; 3]) -> f32 {
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]) as f32 / 255.0;
    max_component.clamp(0.0, 1.0)
}
