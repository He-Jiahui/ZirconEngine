use std::collections::{HashMap, HashSet};

use crate::core::resource::ResourceId;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPipelineVariantId;

use super::material_pipeline_publication::{
    MaterialPipelineRequirement, ResolvedMaterialPipelineRequirement,
};

#[derive(Default)]
struct MaterialPipelineGenerationAdmission {
    requirements: HashSet<MaterialPipelineRequirement>,
    resolved_pipelines: HashSet<ResolvedMaterialPipelineRequirement>,
}

#[derive(Default)]
/// 记住材质各代已全部准入的需求；每代为解析后的管线持有一份 pin，淘汰代际时逐份释放。
pub(super) struct MaterialPipelineGenerationAdmissionLedger {
    materials: HashMap<ResourceId, HashMap<u64, MaterialPipelineGenerationAdmission>>,
    resolved_pipeline_pins: HashMap<ResolvedMaterialPipelineRequirement, usize>,
}

impl MaterialPipelineGenerationAdmissionLedger {
    pub(super) fn contains_all<'a>(
        &self,
        material_id: ResourceId,
        generation: u64,
        requirements: impl IntoIterator<Item = &'a MaterialPipelineRequirement>,
    ) -> bool {
        let Some(ready) = self
            .materials
            .get(&material_id)
            .and_then(|generations| generations.get(&generation))
        else {
            return false;
        };
        requirements
            .into_iter()
            .all(|requirement| ready.requirements.contains(requirement))
    }

    pub(super) fn record_ready<'a, R>(
        &mut self,
        material_id: ResourceId,
        generation: u64,
        requirements: impl IntoIterator<Item = &'a MaterialPipelineRequirement>,
        resolved_pipelines: R,
    ) where
        R: IntoIterator<Item = ResolvedMaterialPipelineRequirement>,
    {
        let admission = self
            .materials
            .entry(material_id)
            .or_default()
            .entry(generation)
            .or_default();
        admission
            .requirements
            .extend(requirements.into_iter().cloned());
        for resolved in resolved_pipelines {
            if !admission.resolved_pipelines.insert(resolved) {
                continue;
            }
            let pin_count = self.resolved_pipeline_pins.entry(resolved).or_default();
            *pin_count = pin_count
                .checked_add(1)
                .expect("material generation pipeline pin count overflow");
        }
    }

    pub(super) fn retain_live_generations(
        &mut self,
        material_id: ResourceId,
        live_generations: [Option<u64>; 3],
    ) {
        let Some(generations) = self.materials.get_mut(&material_id) else {
            return;
        };
        let mut retired_pipelines = Vec::new();
        generations.retain(|generation, admission| {
            let retained = live_generations
                .iter()
                .flatten()
                .any(|live| live == generation);
            if !retained {
                retired_pipelines.extend(admission.resolved_pipelines.iter().copied());
            }
            retained
        });
        let remove_material = generations.is_empty();
        for resolved in retired_pipelines {
            self.unpin_resolved_pipeline(resolved);
        }
        if remove_material {
            self.materials.remove(&material_id);
        }
    }

    fn unpin_resolved_pipeline(&mut self, resolved: ResolvedMaterialPipelineRequirement) {
        let remove = {
            let pin_count = self
                .resolved_pipeline_pins
                .get_mut(&resolved)
                .expect("retired material generation must own a resolved pipeline pin");
            *pin_count = pin_count
                .checked_sub(1)
                .expect("material generation pipeline pin count underflow");
            *pin_count == 0
        };
        if remove {
            self.resolved_pipeline_pins.remove(&resolved);
        }
    }

    pub(super) fn resolved_pipeline_pin_count(
        &self,
        target: super::PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) -> usize {
        self.resolved_pipeline_pins
            .get(&ResolvedMaterialPipelineRequirement::new(
                target, variant_id,
            ))
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn pinned_resolved_pipeline_count(&self) -> usize {
        self.resolved_pipeline_pins.len()
    }

    pub(super) fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub(super) fn generation_count(&self, material_id: ResourceId) -> usize {
        self.materials.get(&material_id).map_or(0, HashMap::len)
    }

    pub(super) fn requirement_count(&self, material_id: ResourceId) -> usize {
        self.materials
            .get(&material_id)
            .map(|generations| {
                generations
                    .values()
                    .map(|admission| admission.requirements.len())
                    .sum()
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "tests/material_pipeline_generation_admission.rs"]
mod tests;
