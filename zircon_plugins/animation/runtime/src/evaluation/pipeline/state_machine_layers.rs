use std::collections::BTreeMap;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::{AnimationPoseBone, AnimationPoseOutput};
use zircon_runtime::scene::EntityId;

use super::machine_instance_key::MachineInstanceKey;
use super::nested_machine_resolve::resolve_machine_instance;
use super::nested_machine_sample::{
    normalized_machine_state_time, sample_machine_state_events, sample_machine_state_pose,
    sample_machine_transition_pose,
};
use super::requests::PendingStateMachinePoseSample;
use super::state_machine_cache::resolve_sub_machine_id;
use super::state_machine_transition::{
    advance_state_machine_transition, begin_state_machine_transition, select_interruption_candidate,
};
use super::AnimationEvaluationPipeline;
use crate::{
    AnimationStateMachineLayerDiagnostic, AnimationStateMachineLayerError,
    CompiledStateMachineLayer, MaskWeights, PoseBuffer, PoseLayer, PoseLayerBlendMode,
};

pub(super) struct StateMachineLayerApplyResult {
    pub(super) events: Vec<crate::AnimationClipEvent>,
    pub(super) diagnostics: Vec<AnimationStateMachineLayerDiagnostic>,
}

pub(super) fn apply_state_machine_layers(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    pending_samples: &[PendingStateMachinePoseSample],
    poses: &mut BTreeMap<EntityId, AnimationPoseOutput>,
) -> StateMachineLayerApplyResult {
    let mut events = Vec::new();
    let mut diagnostics = Vec::new();
    for pending in pending_samples {
        let Some(compiled) =
            pipeline.compiled_state_machine_layers(asset_manager, pending.state_machine_id)
        else {
            continue;
        };
        if compiled.layers().is_empty() || !poses.contains_key(&pending.entity) {
            continue;
        }
        for layer in compiled.layers() {
            let Some((layer_pose, layer_events)) =
                evaluate_layer_pose(pipeline, asset_manager, pending, layer)
            else {
                continue;
            };
            let Some(base_pose) = poses.get_mut(&pending.entity) else {
                continue;
            };
            match blend_layer_pose(
                base_pose,
                &layer_pose,
                layer.weight(),
                layer.blend_mode(),
                layer.mask(),
            ) {
                Ok(()) => events.extend(layer_events),
                Err(error) => diagnostics.push(AnimationStateMachineLayerDiagnostic {
                    entity: pending.entity,
                    layer: layer.name().to_string(),
                    error,
                }),
            }
        }
    }
    StateMachineLayerApplyResult {
        events,
        diagnostics,
    }
}

fn evaluate_layer_pose(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    pending: &PendingStateMachinePoseSample,
    layer: &CompiledStateMachineLayer,
) -> Option<(AnimationPoseOutput, Vec<crate::AnimationClipEvent>)> {
    let layer_machine_id = resolve_sub_machine_id(asset_manager, layer.machine())?;
    let base_instance = MachineInstanceKey::root(pending.entity, pending.state_machine_id);
    let layer_instance = base_instance.nested(layer.name(), layer_machine_id)?;
    let active_state = pipeline.nested_machine_states.get(&layer_instance).cloned();
    let active_transition = pipeline
        .nested_machine_transitions
        .get(&layer_instance)
        .cloned();
    let mut resolved = resolve_machine_instance(
        pipeline,
        asset_manager,
        layer_instance.clone(),
        layer_machine_id,
        active_state,
        active_transition,
        &pending.parameters,
        pending.skeleton_id,
        pending.to_time_seconds,
    )?;
    let instance = resolved.instance;
    let machine = resolved.machine;
    let evaluation = resolved.evaluation;
    let active_state = evaluation.active_state.as_deref()?;
    let mut interrupted_events = Vec::new();
    let mut interrupted_source = None;
    let mut event_start = (pending.from_time_seconds, 0.0);
    let transition = if let Some(previous) = resolved.transition.take() {
        event_start = (previous.from_time_seconds, previous.to_time_seconds);
        let advanced = advance_state_machine_transition(previous, pending.delta_seconds);
        let candidate = select_interruption_candidate(
            pipeline,
            asset_manager,
            machine.as_ref(),
            &evaluation.parameters,
            pending.skeleton_id,
            &advanced,
        );
        if let Some(candidate) = candidate {
            let previous_source = pipeline.interrupted_transition_source(
                &instance,
                &advanced.from_state,
                &advanced.to_state,
            );
            let sampled_source = sample_machine_transition_pose(
                pipeline,
                asset_manager,
                &instance,
                &machine,
                &evaluation.parameters,
                pending.entity,
                pending.skeleton_id,
                &advanced,
                previous_source.as_deref(),
            )
            .map(|(_, pose)| pose);
            if let Some(source) = sampled_source {
                interrupted_events.extend(sample_layer_transition_events(
                    pipeline,
                    asset_manager,
                    pending,
                    &instance,
                    &machine,
                    &evaluation.parameters,
                    &advanced,
                    event_start,
                ));
                interrupted_source = Some(source);
                event_start = (candidate.from_time_seconds, 0.0);
                Some(begin_state_machine_transition(
                    &candidate.transition,
                    candidate.from_time_seconds,
                    0.0,
                ))
            } else {
                Some(advanced)
            }
        } else {
            Some(advanced)
        }
    } else {
        let normalized_time = normalized_machine_state_time(
            pipeline,
            asset_manager,
            &instance,
            &machine,
            active_state,
            &evaluation.parameters,
            pending.skeleton_id,
            pending.to_time_seconds,
        );
        evaluation
            .transition
            .as_ref()
            .zip(resolved.requested_desc)
            .filter(|(_, desc)| desc.exit_ready(normalized_time))
            .map(|(requested, desc)| {
                begin_state_machine_transition(
                    requested,
                    pending.to_time_seconds,
                    if desc.exit_time().is_some() {
                        0.0
                    } else {
                        pending.delta_seconds
                    },
                )
            })
    };
    if let (Some(source), Some(active_transition)) = (interrupted_source, transition.as_ref()) {
        pipeline.record_interrupted_transition_source(
            instance.clone(),
            &active_transition.from_state,
            &active_transition.to_state,
            source,
        );
    }
    let state_update = transition
        .as_ref()
        .map(|transition| {
            if transition.elapsed_seconds >= transition.duration_seconds {
                transition.to_state.clone()
            } else {
                transition.from_state.clone()
            }
        })
        .or_else(|| evaluation.active_state.clone())?;
    if resolved.is_nested {
        pipeline
            .nested_machine_states
            .insert(layer_instance, resolved.root_active_state);
    }
    pipeline
        .nested_machine_states
        .insert(instance.clone(), state_update.clone());

    if let Some(transition) = transition {
        let active_source = pipeline.interrupted_transition_source(
            &instance,
            &transition.from_state,
            &transition.to_state,
        );
        let pose = sample_machine_transition_pose(
            pipeline,
            asset_manager,
            &instance,
            &machine,
            &evaluation.parameters,
            pending.entity,
            pending.skeleton_id,
            &transition,
            active_source.as_deref(),
        )?
        .1;
        interrupted_events.extend(sample_layer_transition_events(
            pipeline,
            asset_manager,
            pending,
            &instance,
            &machine,
            &evaluation.parameters,
            &transition,
            event_start,
        ));
        if transition.elapsed_seconds < transition.duration_seconds {
            pipeline
                .nested_machine_transitions
                .insert(instance, transition);
        } else {
            pipeline.nested_machine_transitions.remove(&instance);
            pipeline.clear_interrupted_transition_source(&instance);
        }
        return Some((pose, interrupted_events));
    }

    pipeline.nested_machine_transitions.remove(&instance);
    pipeline.clear_interrupted_transition_source(&instance);
    let events = sample_machine_state_events(
        pipeline,
        asset_manager,
        &instance,
        &machine,
        &state_update,
        &evaluation.parameters,
        pending.entity,
        pending.skeleton_id,
        pending.from_time_seconds,
        pending.to_time_seconds,
    );
    let pose = sample_machine_state_pose(
        pipeline,
        asset_manager,
        &instance,
        &machine,
        &state_update,
        &evaluation.parameters,
        pending.entity,
        pending.skeleton_id,
        pending.to_time_seconds,
    )?
    .1;
    Some((pose, events))
}

fn sample_layer_transition_events(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    pending: &PendingStateMachinePoseSample,
    instance: &MachineInstanceKey,
    machine: &crate::CompiledAnimationStateMachine,
    parameters: &zircon_runtime::core::framework::animation::AnimationParameterMap,
    transition: &zircon_runtime::scene::AnimationStateTransitionRuntime,
    start: (
        zircon_runtime::core::math::Real,
        zircon_runtime::core::math::Real,
    ),
) -> Vec<crate::AnimationClipEvent> {
    let mut events = sample_machine_state_events(
        pipeline,
        asset_manager,
        instance,
        machine,
        &transition.from_state,
        parameters,
        pending.entity,
        pending.skeleton_id,
        start.0,
        transition.from_time_seconds,
    );
    events.extend(sample_machine_state_events(
        pipeline,
        asset_manager,
        instance,
        machine,
        &transition.to_state,
        parameters,
        pending.entity,
        pending.skeleton_id,
        start.1,
        transition.to_time_seconds,
    ));
    events
}

fn blend_layer_pose(
    base: &mut AnimationPoseOutput,
    layer_pose: &AnimationPoseOutput,
    weight: zircon_runtime::core::math::Real,
    blend_mode: PoseLayerBlendMode,
    mask: Option<&MaskWeights>,
) -> Result<(), AnimationStateMachineLayerError> {
    if base.bones.len() != layer_pose.bones.len() {
        return Err(AnimationStateMachineLayerError::BoneCountMismatch {
            base: base.bones.len(),
            layer: layer_pose.bones.len(),
        });
    }
    if let Some((index, (base, layer))) = base
        .bones
        .iter()
        .zip(&layer_pose.bones)
        .enumerate()
        .find(|(_, (base, layer))| base.name != layer.name)
    {
        return Err(AnimationStateMachineLayerError::BoneNameMismatch {
            index,
            base: base.name.clone(),
            layer: layer.name.clone(),
        });
    }
    let mut base_buffer = PoseBuffer::new(base.bones.len());
    let mut layer_buffer = PoseBuffer::new(layer_pose.bones.len());
    for (index, bone) in base.bones.iter().enumerate() {
        base_buffer
            .set_transform(index, bone.local_transform)
            .map_err(AnimationStateMachineLayerError::BasePose)?;
    }
    for (index, bone) in layer_pose.bones.iter().enumerate() {
        layer_buffer
            .set_transform(index, bone.local_transform)
            .map_err(AnimationStateMachineLayerError::LayerPose)?;
    }
    let pose_layer = PoseLayer::new(&layer_buffer, weight, blend_mode);
    let pose_layer = match mask {
        Some(mask) => pose_layer.with_mask(mask),
        None => pose_layer,
    };
    base_buffer
        .blend_layers(&[pose_layer])
        .map_err(AnimationStateMachineLayerError::Blend)?;
    base.bones = base
        .bones
        .iter()
        .enumerate()
        .filter_map(|(index, bone)| {
            Some(AnimationPoseBone {
                name: bone.name.clone(),
                local_transform: base_buffer.transform(index)?,
            })
        })
        .collect();
    Ok(())
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::framework::animation::{
        AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
    };
    use zircon_runtime::core::math::Transform;

    use super::{blend_layer_pose, AnimationStateMachineLayerError};
    use crate::PoseLayerBlendMode;

    #[test]
    fn layer_pose_bone_name_mismatch_is_typed() {
        let mut base = pose("Hand");
        let layer = pose("Foot");

        let error = blend_layer_pose(&mut base, &layer, 1.0, PoseLayerBlendMode::Override, None)
            .unwrap_err();

        assert_eq!(
            error,
            AnimationStateMachineLayerError::BoneNameMismatch {
                index: 0,
                base: "Hand".to_string(),
                layer: "Foot".to_string(),
            }
        );
    }

    fn pose(name: &str) -> AnimationPoseOutput {
        AnimationPoseOutput {
            source: AnimationPoseSource::StateMachine,
            active_state: Some("State".to_string()),
            bones: vec![AnimationPoseBone {
                name: name.to_string(),
                local_transform: Transform::default(),
            }],
        }
    }
}
