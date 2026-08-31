use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::math::{Real, Vec2};

use crate::state_machine::condition_expression::ParameterSlot;
use crate::{BlendSpace1D, BlendSpace2D};

pub(crate) type CompiledGraphSamples<'a> = [Option<(&'a AssetReference, Real)>; 3];

#[derive(Debug)]
pub(crate) struct StateMachineBlendSamplingState {
    triangle_hints: Box<[Option<usize>]>,
}

impl StateMachineBlendSamplingState {
    pub(crate) fn new(state_count: usize) -> Self {
        Self {
            triangle_hints: vec![None; state_count].into_boxed_slice(),
        }
    }

    pub(crate) fn ensure_state_count(&mut self, state_count: usize) {
        if self.triangle_hints.len() != state_count {
            self.triangle_hints = vec![None; state_count].into_boxed_slice();
        }
    }

    pub(crate) fn triangle_hint_mut(&mut self, state_index: usize) -> Option<&mut Option<usize>> {
        self.triangle_hints.get_mut(state_index)
    }

    #[cfg(test)]
    pub(crate) fn triangle_hint(&self, state_index: usize) -> Option<usize> {
        self.triangle_hints.get(state_index).copied().flatten()
    }
}

#[derive(Clone, Debug)]
pub(super) enum CompiledStateKind {
    Clip(AssetReference),
    GraphRef(AssetReference),
    BlendSpace1D {
        parameter: ParameterSlot,
        blend: BlendSpace1D,
        graphs: Box<[AssetReference]>,
    },
    BlendSpace2D {
        parameter: ParameterSlot,
        blend: BlendSpace2D,
        graphs: Box<[AssetReference]>,
    },
    SubMachine(AssetReference),
}

#[derive(Clone, Debug)]
pub(super) struct CompiledState {
    pub(super) name: String,
    pub(super) kind: CompiledStateKind,
}

impl CompiledState {
    pub(super) fn graph_samples<'a>(
        &'a self,
        values: &[Option<AnimationParameterValue>],
    ) -> CompiledGraphSamples<'a> {
        self.graph_samples_with_hint(values, None)
    }

    pub(super) fn graph_samples_with_hint<'a>(
        &'a self,
        values: &[Option<AnimationParameterValue>],
        triangle_hint: Option<&mut Option<usize>>,
    ) -> CompiledGraphSamples<'a> {
        match &self.kind {
            CompiledStateKind::Clip(_) => [None, None, None],
            CompiledStateKind::GraphRef(graph) => [Some((graph, 1.0)), None, None],
            CompiledStateKind::BlendSpace1D {
                parameter,
                blend,
                graphs,
            } => {
                let Some(AnimationParameterValue::Scalar(value)) =
                    values.get(parameter.index()).and_then(Option::as_ref)
                else {
                    return [None, None, None];
                };
                let Some(weights) = blend.sample(*value) else {
                    return [None, None, None];
                };
                let pairs = weights.as_pairs();
                [
                    graph_sample(graphs, pairs[0]),
                    graph_sample(graphs, pairs[1]),
                    None,
                ]
            }
            CompiledStateKind::BlendSpace2D {
                parameter,
                blend,
                graphs,
            } => {
                let Some(AnimationParameterValue::Vec2(value)) =
                    values.get(parameter.index()).and_then(Option::as_ref)
                else {
                    return [None, None, None];
                };
                let retained_hint = triangle_hint.as_ref().and_then(|hint| **hint);
                let Some((weights, next_hint)) =
                    blend.sample_with_hint(Vec2::from_array(*value), retained_hint)
                else {
                    return [None, None, None];
                };
                if let (Some(hint), Some(next_hint)) = (triangle_hint, next_hint) {
                    *hint = Some(next_hint);
                }
                let pairs = weights.as_pairs();
                [
                    graph_sample(graphs, pairs[0]),
                    graph_sample(graphs, pairs[1]),
                    graph_sample(graphs, pairs[2]),
                ]
            }
            CompiledStateKind::SubMachine(_) => [None, None, None],
        }
    }

    pub(super) fn clip(&self) -> Option<&AssetReference> {
        match &self.kind {
            CompiledStateKind::Clip(clip) => Some(clip),
            _ => None,
        }
    }

    pub(super) fn sub_machine(&self) -> Option<&AssetReference> {
        match &self.kind {
            CompiledStateKind::SubMachine(machine) => Some(machine),
            _ => None,
        }
    }
}

fn graph_sample(
    graphs: &[AssetReference],
    (index, weight): (u32, Real),
) -> Option<(&AssetReference, Real)> {
    (weight > 0.0)
        .then(|| graphs.get(index as usize).map(|graph| (graph, weight)))
        .flatten()
}
