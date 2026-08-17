use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::super::{RenderPipelinePhase, RenderViewFamilyPipeline};
use super::{
    PostProcessChainSlot, PostProcessEffectKind, PostProcessGraphValidationError,
    PostProcessPassNode, PostProcessStackDescriptor,
};

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
            .collect::<BTreeSet<_>>();
        let initial_resources = available.clone();
        let mut produced = BTreeSet::new();
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
        for node in &self.nodes {
            let phase = node.chain_slot.pipeline_phase();
            if !pipeline.phases().contains(&phase) {
                return Err(
                    PostProcessGraphValidationError::UnavailableViewFamilyPhase {
                        node: node.name.clone(),
                        phase,
                    },
                );
            }
        }
        for phase in required_post_process_phases(pipeline) {
            if !self
                .nodes
                .iter()
                .any(|node| node.chain_slot.pipeline_phase() == phase)
            {
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
            RenderPipelinePhase::TemporalReconstruction | RenderPipelinePhase::SpatialUpscale
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

    for (index, node) in nodes.iter().enumerate() {
        for dependency in &node.after {
            let Some(dependency_index) = indices_by_kind.get(dependency).copied() else {
                return Err(PostProcessGraphValidationError::MissingDependency {
                    node: node.name.clone(),
                    dependency: *dependency,
                });
            };
            dependencies[index].insert(dependency_index);
            dependents[dependency_index].push(index);
        }
    }

    // A phase edge is an ordering constraint in addition to resource/effect dependencies.
    // This keeps display mapping and output work from becoming ready ahead of temporal or HDR
    // scene-linear work merely because their legacy settings omitted an explicit `after` edge.
    for (later_index, later_node) in nodes.iter().enumerate() {
        let later_phase = later_node.chain_slot.pipeline_phase().order();
        for (earlier_index, earlier_node) in nodes.iter().enumerate() {
            if earlier_node.chain_slot.pipeline_phase().order() < later_phase
                && dependencies[later_index].insert(earlier_index)
            {
                dependents[earlier_index].push(later_index);
            }
        }
    }

    let mut ready = dependencies
        .iter()
        .enumerate()
        .filter_map(|(index, dependencies)| dependencies.is_empty().then_some(index))
        .collect::<VecDeque<_>>();
    let mut ordered = Vec::with_capacity(nodes.len());

    while let Some(index) = ready.pop_front() {
        ordered.push(index);
        for dependent in &dependents[index] {
            dependencies[*dependent].remove(&index);
            if dependencies[*dependent].is_empty() {
                ready.push_back(*dependent);
            }
        }
    }

    if ordered.len() != nodes.len() {
        return Err(PostProcessGraphValidationError::CycleDetected);
    }

    Ok(ordered)
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        PostProcessEffectSettings, RenderPipelinePhase, RenderResolutionPolicy, RenderUpscalerKind,
        RenderViewFamilyPipeline,
    };
    use crate::core::math::UVec2;

    use super::{
        ordered_node_indices, PostProcessEffectKind, PostProcessGraphValidationError,
        PostProcessPassGraph, PostProcessPassNode, PostProcessStackDescriptor,
    };

    #[test]
    fn graph_orders_independent_nodes_by_view_family_phase() {
        let nodes = vec![
            PostProcessPassNode::new("output", PostProcessEffectKind::OutputTransfer),
            PostProcessPassNode::new("taa", PostProcessEffectKind::TaaResolve),
            PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom),
            PostProcessPassNode::new("fxaa", PostProcessEffectKind::Fxaa),
            PostProcessPassNode::new("uber", PostProcessEffectKind::Uber),
            PostProcessPassNode::new("upscale", PostProcessEffectKind::Upscale),
        ];

        assert_eq!(
            ordered_node_indices(&nodes).expect("independent phases form a valid graph"),
            vec![1, 2, 4, 3, 5, 0]
        );
    }

    #[test]
    fn graph_rejects_temporal_nodes_when_the_view_family_is_spatial() {
        let stack = PostProcessStackDescriptor {
            initial_resources: Vec::new(),
            effects: vec![PostProcessEffectSettings::new(
                PostProcessEffectKind::TaaResolve,
            )],
        };
        let spatial_pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::default(),
            RenderUpscalerKind::Spatial,
        );

        assert_eq!(
            PostProcessPassGraph::validate_stack_for_view_family(&stack, &spatial_pipeline),
            Err(
                PostProcessGraphValidationError::UnavailableViewFamilyPhase {
                    node: "taa-resolve".to_string(),
                    phase: RenderPipelinePhase::TemporalReconstruction,
                }
            )
        );
    }

    #[test]
    fn graph_rejects_display_upscale_when_temporal_reconstruction_owns_the_transition() {
        let stack = PostProcessStackDescriptor {
            initial_resources: Vec::new(),
            effects: vec![PostProcessEffectSettings::new(
                PostProcessEffectKind::Upscale,
            )],
        };
        let temporal_pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.5, 1.0),
            RenderUpscalerKind::Temporal,
        );

        assert_eq!(
            PostProcessPassGraph::validate_stack_for_view_family(&stack, &temporal_pipeline),
            Err(
                PostProcessGraphValidationError::UnavailableViewFamilyPhase {
                    node: "upscale".to_string(),
                    phase: RenderPipelinePhase::SpatialUpscale,
                }
            )
        );
    }

    #[test]
    fn graph_requires_spatial_upscale_when_the_resolved_view_family_needs_it() {
        let stack = PostProcessStackDescriptor {
            initial_resources: Vec::new(),
            effects: Vec::new(),
        };
        let spatial_pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.5, 1.0),
            RenderUpscalerKind::Spatial,
        );

        assert_eq!(
            PostProcessPassGraph::validate_stack_for_view_family(&stack, &spatial_pipeline),
            Err(
                PostProcessGraphValidationError::MissingRequiredViewFamilyPhase {
                    phase: RenderPipelinePhase::SpatialUpscale,
                }
            )
        );
    }

    #[test]
    fn graph_requires_temporal_reconstruction_when_the_resolved_view_family_needs_it() {
        let stack = PostProcessStackDescriptor {
            initial_resources: Vec::new(),
            effects: Vec::new(),
        };
        let temporal_pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.5, 1.0),
            RenderUpscalerKind::Temporal,
        );

        assert_eq!(
            PostProcessPassGraph::validate_stack_for_view_family(&stack, &temporal_pipeline),
            Err(
                PostProcessGraphValidationError::MissingRequiredViewFamilyPhase {
                    phase: RenderPipelinePhase::TemporalReconstruction,
                }
            )
        );
    }
}
