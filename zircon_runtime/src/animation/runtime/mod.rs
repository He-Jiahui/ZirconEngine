use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::asset::{
    AnimationClipAsset, AnimationConditionOperatorAsset, AnimationGraphAsset,
    AnimationGraphNodeAsset, AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
    AnimationStateMachineAsset,
};
use crate::core::framework::animation::{
    AnimationGraphClipInstance, AnimationGraphEvaluation, AnimationManager, AnimationParameterMap,
    AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseBone, AnimationPoseOutput,
    AnimationPoseSource, AnimationStateMachineEvaluation, AnimationTrackPath,
};
use crate::core::math::{Quat, Real, Transform, Vec3};
use crate::core::{CoreError, CoreHandle};

use crate::animation::sequence::AnimationChannelSampleExt;

const DEFAULT_GRAPH_CLIP_PLAYBACK_SPEED: Real = 1.0;

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

impl AnimationManager for DefaultAnimationManager {
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
            .filter(|parameter| animation_parameter_value_is_finite(&parameter.default_value))
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
        if animation_parameter_value_is_finite(&value) {
            parameters.insert(name.to_string(), value);
        }
    }

    fn evaluate_graph(
        &self,
        graph: &AnimationGraphAsset,
        overrides: &AnimationParameterMap,
    ) -> AnimationGraphEvaluation {
        let mut parameters = self.parameter_defaults(graph);
        for (name, value) in overrides {
            if animation_parameter_value_is_finite(value) {
                parameters.insert(name.clone(), value.clone());
            }
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
            .filter(|state| state_machine_has_state(state_machine, state))
            .map(ToOwned::to_owned)
            .or_else(|| {
                state_machine_has_state(state_machine, &state_machine.entry_state)
                    .then(|| state_machine.entry_state.clone())
            });
        let mut transitioned = false;
        let mut transition_evaluation = None;

        if let Some(current) = active_state.as_deref() {
            if let Some(transition) = state_machine.transitions.iter().find(|transition| {
                transition.from_state == current
                    && state_machine_has_state(state_machine, &transition.to_state)
                    && transition
                        .conditions
                        .iter()
                        .all(|condition| condition_matches(parameters, condition))
            }) {
                let duration_seconds = if transition.duration_seconds.is_finite() {
                    transition.duration_seconds.max(0.0)
                } else {
                    0.0
                };
                if duration_seconds > Real::EPSILON {
                    transition_evaluation = Some(
                        crate::core::framework::animation::AnimationStateTransitionEvaluation {
                            from_state: current.to_string(),
                            to_state: transition.to_state.clone(),
                            duration_seconds,
                        },
                    );
                } else if active_state.as_deref() != Some(transition.to_state.as_str()) {
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
            transition: transition_evaluation,
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
            .map(animation_pose_bone_from_skeleton)
            .collect::<Result<Vec<_>, _>>()?;

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

fn animation_pose_bone_from_skeleton(
    bone: &AnimationSkeletonBoneAsset,
) -> Result<AnimationPoseBone, String> {
    if !real_array_is_finite(&bone.local_translation) {
        return Err(format!(
            "non-finite skeleton bind translation for bone `{}`: {:?}",
            bone.name, bone.local_translation
        ));
    }
    if !real_array_is_finite(&bone.local_rotation) {
        return Err(format!(
            "non-finite skeleton bind rotation for bone `{}`: {:?}",
            bone.name, bone.local_rotation
        ));
    }
    if !quaternion_array_is_normalizable(&bone.local_rotation) {
        return Err(format!(
            "zero-length skeleton bind rotation for bone `{}`: {:?}",
            bone.name, bone.local_rotation
        ));
    }
    if !real_array_is_finite(&bone.local_scale) {
        return Err(format!(
            "non-finite skeleton bind scale for bone `{}`: {:?}",
            bone.name, bone.local_scale
        ));
    }

    Ok(AnimationPoseBone {
        name: bone.name.clone(),
        local_transform: Transform {
            translation: Vec3::from_array(bone.local_translation),
            rotation: Quat::from_array(bone.local_rotation).normalize(),
            scale: Vec3::from_array(bone.local_scale),
        },
    })
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
                playback_speed: finite_graph_clip_playback_speed(*playback_speed),
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
    let Some(current) = parameters.get(&condition.parameter) else {
        return false;
    };
    if !animation_parameter_value_is_finite(current)
        || condition
            .value
            .as_ref()
            .is_some_and(|value| !animation_parameter_value_is_finite(value))
    {
        return false;
    }
    if matches!(
        condition.operator,
        AnimationConditionOperatorAsset::Triggered
    ) {
        return matches!(current, AnimationParameterValue::Trigger);
    }

    let Some(expected) = condition.value.as_ref() else {
        return false;
    };
    match condition.operator {
        AnimationConditionOperatorAsset::Triggered => unreachable!(),
        AnimationConditionOperatorAsset::Equal => current == expected,
        AnimationConditionOperatorAsset::NotEqual => current != expected,
        AnimationConditionOperatorAsset::Greater => {
            numeric_parameter(Some(current)) > numeric_parameter(Some(expected))
        }
        AnimationConditionOperatorAsset::GreaterEqual => {
            numeric_parameter(Some(current)) >= numeric_parameter(Some(expected))
        }
        AnimationConditionOperatorAsset::Less => {
            numeric_parameter(Some(current)) < numeric_parameter(Some(expected))
        }
        AnimationConditionOperatorAsset::LessEqual => {
            numeric_parameter(Some(current)) <= numeric_parameter(Some(expected))
        }
    }
}

fn state_machine_has_state(state_machine: &AnimationStateMachineAsset, name: &str) -> bool {
    state_machine.states.iter().any(|state| state.name == name)
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
        Some(AnimationParameterValue::Scalar(value)) if value.is_finite() => Some(*value),
        _ => None,
    }
}

fn finite_graph_clip_playback_speed(playback_speed: Real) -> Real {
    if playback_speed.is_finite() {
        playback_speed
    } else {
        DEFAULT_GRAPH_CLIP_PLAYBACK_SPEED
    }
}

fn animation_parameter_value_is_finite(value: &AnimationParameterValue) -> bool {
    match value {
        AnimationParameterValue::Scalar(value) => value.is_finite(),
        AnimationParameterValue::Vec2(value) => value.iter().all(|component| component.is_finite()),
        AnimationParameterValue::Vec3(value) => value.iter().all(|component| component.is_finite()),
        AnimationParameterValue::Vec4(value) => value.iter().all(|component| component.is_finite()),
        AnimationParameterValue::Bool(_)
        | AnimationParameterValue::Integer(_)
        | AnimationParameterValue::Trigger => true,
    }
}

fn resolve_clip_sample_time(duration_seconds: Real, time_seconds: Real, looping: bool) -> Real {
    if !duration_seconds.is_finite() || duration_seconds <= Real::EPSILON {
        return 0.0;
    }
    if !time_seconds.is_finite() {
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

fn real_array_is_finite<const N: usize>(value: &[Real; N]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn quaternion_array_is_normalizable(value: &[Real; 4]) -> bool {
    value
        .iter()
        .map(|component| component * component)
        .sum::<Real>()
        > Real::EPSILON
}

fn sample_vec3(value: &crate::asset::AnimationChannelValueAsset) -> Result<Vec3, String> {
    match value {
        crate::asset::AnimationChannelValueAsset::Vec3(value)
            if value.iter().all(|c| c.is_finite()) =>
        {
            Ok(Vec3::from_array(*value))
        }
        crate::asset::AnimationChannelValueAsset::Vec3(value) => {
            Err(format!("non-finite vec3 animation sample: {value:?}"))
        }
        other => Err(format!("expected vec3 animation sample, found {other:?}")),
    }
}

fn sample_quaternion(value: &crate::asset::AnimationChannelValueAsset) -> Result<Quat, String> {
    match value {
        crate::asset::AnimationChannelValueAsset::Quaternion(value)
            if value.iter().all(|c| c.is_finite()) && quaternion_array_is_normalizable(value) =>
        {
            Ok(Quat::from_array(*value).normalize())
        }
        crate::asset::AnimationChannelValueAsset::Quaternion(value)
            if value.iter().all(|c| c.is_finite()) =>
        {
            Err(format!(
                "zero-length quaternion animation sample: {value:?}"
            ))
        }
        crate::asset::AnimationChannelValueAsset::Quaternion(value) => {
            Err(format!("non-finite quaternion animation sample: {value:?}"))
        }
        other => Err(format!(
            "expected quaternion animation sample, found {other:?}"
        )),
    }
}
