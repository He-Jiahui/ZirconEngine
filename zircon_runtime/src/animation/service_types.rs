use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::asset::{
    AnimationClipAsset, AnimationConditionOperatorAsset, AnimationGraphAsset,
    AnimationGraphNodeAsset, AnimationSkeletonAsset, AnimationStateMachineAsset,
};
use crate::core::framework::animation::{
    AnimationGraphClipInstance, AnimationGraphEvaluation, AnimationParameterMap,
    AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseBone, AnimationPoseOutput,
    AnimationPoseSource, AnimationStateMachineEvaluation, AnimationTrackPath,
};
use crate::core::math::{Quat, Real, Transform, Vec3};
use crate::core::{CoreError, CoreHandle};

use super::AnimationInterface;

#[derive(Clone, Debug, Default)]
pub struct AnimationDriver;

#[derive(Clone, Debug)]
pub struct DefaultAnimationManager {
    core: Option<CoreHandle>,
    playback_settings: Arc<Mutex<AnimationPlaybackSettings>>,
}

impl Default for DefaultAnimationManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultAnimationManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        let playback_settings = core
            .as_ref()
            .and_then(|core| core.load_config(super::ANIMATION_PLAYBACK_CONFIG_KEY).ok())
            .unwrap_or_default();
        Self {
            core,
            playback_settings: Arc::new(Mutex::new(playback_settings)),
        }
    }

    pub fn store_playback_settings(
        &self,
        playback_settings: AnimationPlaybackSettings,
    ) -> Result<(), CoreError> {
        *self
            .playback_settings
            .lock()
            .expect("animation playback mutex poisoned") = playback_settings.clone();
        if let Some(core) = &self.core {
            core.store_config(super::ANIMATION_PLAYBACK_CONFIG_KEY, &playback_settings)?;
        }
        Ok(())
    }
}

impl crate::core::framework::animation::AnimationManager for DefaultAnimationManager {
    fn playback_settings(&self) -> AnimationPlaybackSettings {
        self.playback_settings
            .lock()
            .expect("animation playback mutex poisoned")
            .clone()
    }

    fn normalize_track_path(&self, path: &AnimationTrackPath) -> AnimationTrackPath {
        path.clone()
    }

    fn parameter_defaults(&self, graph: &AnimationGraphAsset) -> AnimationParameterMap {
        graph
            .parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.default_value.clone()))
            .collect()
    }

    fn parameter_value(
        &self,
        parameters: &AnimationParameterMap,
        name: &str,
    ) -> Option<AnimationParameterValue> {
        parameters.get(name).cloned()
    }

    fn set_parameter(
        &self,
        parameters: &mut AnimationParameterMap,
        name: &str,
        value: AnimationParameterValue,
    ) {
        parameters.insert(name.to_string(), value);
    }

    fn evaluate_graph(
        &self,
        graph: &AnimationGraphAsset,
        overrides: &AnimationParameterMap,
    ) -> AnimationGraphEvaluation {
        let mut parameters = self.parameter_defaults(graph);
        for (name, value) in overrides {
            parameters.insert(name.clone(), value.clone());
        }

        let output_node = graph.nodes.iter().find_map(|node| match node {
            AnimationGraphNodeAsset::Output { source } => Some(source.clone()),
            _ => None,
        });
        let clips = output_node
            .as_deref()
            .map(|source| collect_graph_clips(graph, source, &parameters, &mut HashSet::new()))
            .unwrap_or_default();

        AnimationGraphEvaluation {
            parameters,
            output_node,
            clips,
        }
    }

    fn evaluate_state_machine(
        &self,
        state_machine: &AnimationStateMachineAsset,
        current_state: Option<&str>,
        parameters: &AnimationParameterMap,
    ) -> AnimationStateMachineEvaluation {
        let mut active_state = current_state
            .map(ToOwned::to_owned)
            .or_else(|| Some(state_machine.entry_state.clone()));
        let mut transitioned = false;

        if let Some(current) = active_state.as_deref() {
            if let Some(transition) = state_machine.transitions.iter().find(|transition| {
                transition.from_state == current
                    && transition
                        .conditions
                        .iter()
                        .all(|condition| condition_matches(parameters, condition))
            }) {
                if active_state.as_deref() != Some(transition.to_state.as_str()) {
                    active_state = Some(transition.to_state.clone());
                    transitioned = true;
                }
            }
        }

        let graph = active_state.as_deref().and_then(|state_name| {
            state_machine
                .states
                .iter()
                .find(|state| state.name == state_name)
                .map(|state| state.graph.clone())
        });

        AnimationStateMachineEvaluation {
            parameters: parameters.clone(),
            active_state,
            transitioned,
            graph,
        }
    }

    fn sample_clip_pose(
        &self,
        skeleton: &AnimationSkeletonAsset,
        clip: &AnimationClipAsset,
        time_seconds: Real,
        looping: bool,
    ) -> Result<AnimationPoseOutput, String> {
        let sample_time = resolve_clip_sample_time(clip.duration_seconds, time_seconds, looping);
        let mut bones = skeleton
            .bones
            .iter()
            .map(|bone| AnimationPoseBone {
                name: bone.name.clone(),
                local_transform: Transform {
                    translation: Vec3::from_array(bone.local_translation),
                    rotation: Quat::from_array(bone.local_rotation).normalize(),
                    scale: Vec3::from_array(bone.local_scale),
                },
            })
            .collect::<Vec<_>>();

        for track in &clip.tracks {
            let Some(bone) = bones.iter_mut().find(|bone| bone.name == track.bone_name) else {
                continue;
            };
            if let Some(sample) = track.translation.sample(sample_time) {
                bone.local_transform.translation = sample_vec3(&sample)?;
            }
            if let Some(sample) = track.rotation.sample(sample_time) {
                bone.local_transform.rotation = sample_quaternion(&sample)?;
            }
            if let Some(sample) = track.scale.sample(sample_time) {
                bone.local_transform.scale = sample_vec3(&sample)?;
            }
        }

        Ok(AnimationPoseOutput {
            source: AnimationPoseSource::Clip,
            active_state: None,
            bones,
        })
    }
}

impl AnimationInterface for DefaultAnimationManager {
    fn store_playback_settings(
        &self,
        playback_settings: AnimationPlaybackSettings,
    ) -> Result<(), CoreError> {
        self.store_playback_settings(playback_settings)
    }
}

fn collect_graph_clips(
    graph: &AnimationGraphAsset,
    node_id: &str,
    parameters: &AnimationParameterMap,
    visited: &mut HashSet<String>,
) -> Vec<AnimationGraphClipInstance> {
    if !visited.insert(node_id.to_string()) {
        return Vec::new();
    }

    let result = graph
        .nodes
        .iter()
        .find_map(|node| match node {
            AnimationGraphNodeAsset::Clip {
                id,
                clip,
                playback_speed,
                looping,
            } if id == node_id => Some(vec![AnimationGraphClipInstance {
                clip: clip.clone(),
                playback_speed: *playback_speed,
                looping: *looping,
                weight: 1.0,
            }]),
            AnimationGraphNodeAsset::Blend {
                id,
                inputs,
                weight_parameter,
            } if id == node_id => {
                let scalar = weight_parameter
                    .as_deref()
                    .and_then(|name| parameter_scalar(parameters, name))
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                let input_count = inputs.len().max(1);
                let input_weights = if input_count == 1 {
                    vec![1.0]
                } else {
                    let trailing = if input_count > 1 {
                        scalar / (input_count - 1) as Real
                    } else {
                        0.0
                    };
                    std::iter::once(1.0 - scalar)
                        .chain(std::iter::repeat_n(trailing, input_count - 1))
                        .collect::<Vec<_>>()
                };
                let mut clips = Vec::new();
                for (index, input) in inputs.iter().enumerate() {
                    let weight = input_weights.get(index).copied().unwrap_or(1.0);
                    clips.extend(
                        collect_graph_clips(graph, input, parameters, visited)
                            .into_iter()
                            .map(|mut clip| {
                                clip.weight *= weight;
                                clip
                            }),
                    );
                }
                Some(clips)
            }
            _ => None,
        })
        .unwrap_or_default();

    visited.remove(node_id);
    result
}

fn condition_matches(
    parameters: &AnimationParameterMap,
    condition: &crate::asset::AnimationTransitionConditionAsset,
) -> bool {
    let current = parameters.get(&condition.parameter);
    match condition.operator {
        AnimationConditionOperatorAsset::Triggered => {
            matches!(current, Some(AnimationParameterValue::Trigger))
        }
        AnimationConditionOperatorAsset::Equal => current == condition.value.as_ref(),
        AnimationConditionOperatorAsset::NotEqual => current != condition.value.as_ref(),
        AnimationConditionOperatorAsset::Greater => {
            numeric_parameter(current) > numeric_parameter(condition.value.as_ref())
        }
        AnimationConditionOperatorAsset::GreaterEqual => {
            numeric_parameter(current) >= numeric_parameter(condition.value.as_ref())
        }
        AnimationConditionOperatorAsset::Less => {
            numeric_parameter(current) < numeric_parameter(condition.value.as_ref())
        }
        AnimationConditionOperatorAsset::LessEqual => {
            numeric_parameter(current) <= numeric_parameter(condition.value.as_ref())
        }
    }
}

fn numeric_parameter(value: Option<&AnimationParameterValue>) -> Real {
    match value {
        Some(AnimationParameterValue::Integer(value)) => *value as Real,
        Some(AnimationParameterValue::Scalar(value)) => *value,
        _ => 0.0,
    }
}

fn parameter_scalar(parameters: &AnimationParameterMap, name: &str) -> Option<Real> {
    match parameters.get(name) {
        Some(AnimationParameterValue::Integer(value)) => Some(*value as Real),
        Some(AnimationParameterValue::Scalar(value)) => Some(*value),
        _ => None,
    }
}

fn resolve_clip_sample_time(duration_seconds: Real, time_seconds: Real, looping: bool) -> Real {
    if duration_seconds <= Real::EPSILON {
        return 0.0;
    }
    let clamped = time_seconds.max(0.0);
    if looping {
        if clamped <= duration_seconds {
            clamped
        } else {
            clamped.rem_euclid(duration_seconds)
        }
    } else {
        clamped.min(duration_seconds)
    }
}

fn sample_vec3(value: &crate::asset::AnimationChannelValueAsset) -> Result<Vec3, String> {
    match value {
        crate::asset::AnimationChannelValueAsset::Vec3(value) => Ok(Vec3::from_array(*value)),
        other => Err(format!("expected vec3 animation sample, found {other:?}")),
    }
}

fn sample_quaternion(value: &crate::asset::AnimationChannelValueAsset) -> Result<Quat, String> {
    match value {
        crate::asset::AnimationChannelValueAsset::Quaternion(value) => {
            Ok(Quat::from_array(*value).normalize())
        }
        other => Err(format!(
            "expected quaternion animation sample, found {other:?}"
        )),
    }
}
