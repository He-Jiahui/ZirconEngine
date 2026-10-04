use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

use super::super::{RenderPipelinePhase, RenderViewFamilyPipeline};
use super::{
    PostProcessChainSlot, PostProcessEffectKind, PostProcessGraphValidationError,
    PostProcessPassNode, PostProcessStackDescriptor,
};

const RENDER_PIPELINE_PHASE_COUNT: usize = RenderPipelinePhase::Present.order() as usize + 1;

/// 后处理图保存启用节点的拓扑顺序、跳过节点、固定骨架槽位及输出传输节点；校验阶段检查输入可用性、重复输出、循环和视图族阶段。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostProcessPassGraph {
    pub nodes: Vec<PostProcessPassNode>,
    pub skipped_nodes: Vec<PostProcessPassNode>,
    pub planned_backbone_slots: Vec<PostProcessChainSlot>,
    pub active_chain_slots: Vec<PostProcessChainSlot>,
    pub output_transfer_node: Option<String>,
}

impl PostProcessPassGraph {
    pub fn from_ordered_nodes(
        nodes: Vec<PostProcessPassNode>,
        skipped_nodes: Vec<PostProcessPassNode>,
        output_transfer_node: Option<String>,
    ) -> Self {
        let active_chain_slots = nodes.iter().map(|node| node.chain_slot).collect::<Vec<_>>();
        Self {
            nodes,
            skipped_nodes,
            planned_backbone_slots: PostProcessChainSlot::fixed_backbone().to_vec(),
            active_chain_slots,
            output_transfer_node,
        }
    }

    pub fn validate_stack(
        stack: &PostProcessStackDescriptor,
    ) -> Result<Self, PostProcessGraphValidationError> {
        let enabled_nodes = stack
            .effects
            .iter()
            .filter(|effect| effect.enabled)
            .map(PostProcessPassNode::from_settings)
            .collect::<Vec<_>>();
        let skipped_nodes = stack
            .effects
            .iter()
            .filter(|effect| !effect.enabled)
            .map(PostProcessPassNode::from_settings)
            .collect::<Vec<_>>();
        let order = ordered_node_indices(&enabled_nodes)?;
        let mut available = stack
            .initial_resources
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let initial_resources = available.clone();
        let mut produced = HashSet::with_capacity(
            enabled_nodes
                .iter()
                .map(|node| node.produced_outputs.len())
                .sum(),
        );
        let mut ordered_nodes = Vec::with_capacity(enabled_nodes.len());

        for index in order {
            let node = enabled_nodes[index].clone();
            for resource in &node.required_inputs {
                if !available.contains(resource) {
                    return Err(PostProcessGraphValidationError::MissingRequiredInput {
                        node: node.name.clone(),
                        resource: resource.clone(),
                    });
                }
            }
            for resource in &node.produced_outputs {
                if initial_resources.contains(resource) {
                    return Err(PostProcessGraphValidationError::DuplicateOutputResource {
                        node: node.name.clone(),
                        resource: resource.clone(),
                    });
                }
                if !produced.insert(resource.clone()) {
                    return Err(PostProcessGraphValidationError::DuplicateOutputResource {
                        node: node.name.clone(),
                        resource: resource.clone(),
                    });
                }
                available.insert(resource.clone());
            }
            ordered_nodes.push(node);
        }

        let output_transfer_node = ordered_nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .map(|node| node.name.clone());
        Ok(Self::from_ordered_nodes(
            ordered_nodes,
            skipped_nodes,
            output_transfer_node,
        ))
    }

    /// Validates that every active post-process node belongs to a phase enabled by the resolved
    /// view family. Callers should use this at graph compilation time, after choosing the
    /// temporal or spatial reconstruction policy.
    pub fn validate_stack_for_view_family(
        stack: &PostProcessStackDescriptor,
        pipeline: &RenderViewFamilyPipeline,
    ) -> Result<Self, PostProcessGraphValidationError> {
        let graph = Self::validate_stack(stack)?;
        graph.validate_view_family_phases(pipeline)?;
        Ok(graph)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn skipped_node_count(&self) -> usize {
        self.skipped_nodes.len()
    }

    fn validate_view_family_phases(
        &self,
        pipeline: &RenderViewFamilyPipeline,
    ) -> Result<(), PostProcessGraphValidationError> {
        let enabled_phase_mask = pipeline
            .phases()
            .iter()
            .fold(0_usize, |mask, phase| mask | (1_usize << phase.order()));
        let mut observed_phase_mask = 0_usize;
        for node in &self.nodes {
            let phase = node.chain_slot.pipeline_phase();
            let phase_bit = 1_usize << phase.order();
            if enabled_phase_mask & phase_bit == 0 {
                return Err(
                    PostProcessGraphValidationError::UnavailableViewFamilyPhase {
                        node: node.name.clone(),
                        phase,
                    },
                );
            }
            observed_phase_mask |= phase_bit;
        }
        for phase in required_post_process_phases(pipeline) {
            if observed_phase_mask & (1_usize << phase.order()) == 0 {
                return Err(
                    PostProcessGraphValidationError::MissingRequiredViewFamilyPhase { phase },
                );
            }
        }
        Ok(())
    }
}

fn required_post_process_phases(
    pipeline: &RenderViewFamilyPipeline,
) -> impl Iterator<Item = RenderPipelinePhase> + '_ {
    pipeline.phases().iter().copied().filter(|phase| {
        matches!(
            phase,
            RenderPipelinePhase::TemporalReconstruction
                | RenderPipelinePhase::PrimarySpatialUpscale
                | RenderPipelinePhase::SecondarySpatialUpscale
        )
    })
}

fn ordered_node_indices(
    nodes: &[PostProcessPassNode],
) -> Result<Vec<usize>, PostProcessGraphValidationError> {
    let indices_by_kind = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.kind, index))
        .collect::<BTreeMap<_, _>>();
    let mut dependencies = vec![BTreeSet::<usize>::new(); nodes.len()];
    let mut dependents = vec![Vec::<usize>::new(); nodes.len()];
    let mut phase_buckets: [Vec<usize>; RENDER_PIPELINE_PHASE_COUNT] =
        std::array::from_fn(|_| Vec::new());
    let mut has_reverse_phase_dependency = false;

    for (index, node) in nodes.iter().enumerate() {
        phase_buckets[node.chain_slot.pipeline_phase().order() as usize].push(index);
    }

    for (index, node) in nodes.iter().enumerate() {
        let node_phase = node.chain_slot.pipeline_phase().order();
        for dependency in &node.after {
            let Some(dependency_index) = indices_by_kind.get(dependency).copied() else {
                return Err(PostProcessGraphValidationError::MissingDependency {
                    node: node.name.clone(),
                    dependency: *dependency,
                });
            };
            let dependency_phase = nodes[dependency_index].chain_slot.pipeline_phase().order();
            if dependency_phase == node_phase {
                if dependencies[index].insert(dependency_index) {
                    dependents[dependency_index].push(index);
                }
            } else if dependency_phase > node_phase {
                has_reverse_phase_dependency = true;
            }
        }
    }

    if has_reverse_phase_dependency {
        return Err(PostProcessGraphValidationError::CycleDetected);
    }

    let mut ordered = Vec::with_capacity(nodes.len());
    for phase_bucket in phase_buckets {
        let phase_node_count = phase_bucket.len();
        let mut ready = phase_bucket
            .into_iter()
            .filter(|index| dependencies[*index].is_empty())
            .collect::<VecDeque<_>>();
        let ordered_before_phase = ordered.len();

        while let Some(index) = ready.pop_front() {
            ordered.push(index);
            for dependent in &dependents[index] {
                dependencies[*dependent].remove(&index);
                if dependencies[*dependent].is_empty() {
                    ready.push_back(*dependent);
                }
            }
        }

        if ordered.len() - ordered_before_phase != phase_node_count {
            return Err(PostProcessGraphValidationError::CycleDetected);
        }
    }

    Ok(ordered)
}

#[cfg(test)]
#[path = "pass_graph/tests/optimization_batch_iu_runtime631_tests.rs"]
mod optimization_batch_iu_runtime631_tests;

#[cfg(test)]
#[path = "tests/pass_graph.rs"]
mod tests;
