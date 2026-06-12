use std::collections::BTreeMap;

use crate::asset::{AnimationStateMachineAsset, ProjectAssetManager};
use crate::core::framework::animation::{
    AnimationManager, AnimationParameterMap, AnimationPoseOutput, AnimationPoseSource,
    AnimationStateTransitionEvaluation,
};
use crate::core::math::Real;
use crate::core::resource::AssetReference;
use crate::scene::{AnimationStateTransitionRuntime, EntityId};

use crate::animation::AnimationClipEvent;

use super::graph::{
    blend_weighted_poses, sample_graph_evaluation_clip_events, sample_graph_evaluation_pose,
};
use super::pending::PendingStateMachinePoseSample;

pub(super) fn resolve_state_machine_pose_requests(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingStateMachinePoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<AnimationClipEvent>,
    Vec<(EntityId, Option<String>)>,
    BTreeMap<EntityId, AnimationStateTransitionRuntime>,
) {
    let mut poses = BTreeMap::new();
    let mut events = Vec::new();
    let mut active_state_updates = Vec::new();
    let mut transition_updates = BTreeMap::new();

    for pending in pending_samples {
        let Some(state_machine) = asset_manager
            .load_animation_state_machine_asset(pending.state_machine_id)
            .ok()
        else {
            continue;
        };
        let evaluation = animation.evaluate_state_machine(
            &state_machine,
            pending.active_state.as_deref(),
            &pending.parameters,
        );
        let transition = resolve_state_machine_transition_runtime(
            pending.transition.clone(),
            evaluation.transition.as_ref(),
            pending.to_time_seconds,
            pending.delta_seconds,
        );
        let state_update = transition
            .as_ref()
            .map(|transition| {
                if transition.elapsed_seconds >= transition.duration_seconds {
                    transition.to_state.clone()
                } else {
                    transition.from_state.clone()
                }
            })
            .or_else(|| evaluation.active_state.clone());
        active_state_updates.push((pending.entity, state_update.clone()));

        if let Some(active_transition) = transition.as_ref() {
            events.extend(sample_state_transition_clip_events(
                animation,
                asset_manager,
                &state_machine,
                &evaluation.parameters,
                &pending,
                active_transition,
            ));
            let Some((entity, pose)) = sample_state_transition_pose(
                animation,
                asset_manager,
                &state_machine,
                &evaluation.parameters,
                &pending,
                active_transition,
            ) else {
                continue;
            };
            poses.insert(entity, pose);
            if active_transition.elapsed_seconds < active_transition.duration_seconds {
                transition_updates.insert(entity, active_transition.clone());
            }
            continue;
        }

        events.extend(sample_state_graph_clip_events(
            animation,
            asset_manager,
            evaluation.graph.as_ref(),
            &evaluation.parameters,
            pending.entity,
            pending.from_time_seconds,
            pending.to_time_seconds,
        ));
        let Some((entity, pose)) = sample_state_graph_pose(
            animation,
            asset_manager,
            &state_machine,
            evaluation.graph.as_ref(),
            &evaluation.parameters,
            pending.entity,
            pending.skeleton_id,
            pending.to_time_seconds,
            state_update,
        ) else {
            continue;
        };
        poses.insert(entity, pose);
    }

    (poses, events, active_state_updates, transition_updates)
}

fn resolve_state_machine_transition_runtime(
    previous: Option<AnimationStateTransitionRuntime>,
    requested: Option<&AnimationStateTransitionEvaluation>,
    time_seconds: Real,
    delta_seconds: Real,
) -> Option<AnimationStateTransitionRuntime> {
    let delta_seconds = if delta_seconds.is_finite() {
        delta_seconds.max(0.0)
    } else {
        0.0
    };
    if let Some(mut previous) = previous {
        previous.elapsed_seconds = (previous.elapsed_seconds + delta_seconds)
            .min(previous.duration_seconds)
            .max(0.0);
        previous.from_time_seconds = (previous.from_time_seconds + delta_seconds).max(0.0);
        previous.to_time_seconds = (previous.to_time_seconds + delta_seconds).max(0.0);
        return Some(previous);
    }

    requested.map(|requested| AnimationStateTransitionRuntime {
        from_state: requested.from_state.clone(),
        to_state: requested.to_state.clone(),
        duration_seconds: requested.duration_seconds,
        elapsed_seconds: delta_seconds.min(requested.duration_seconds).max(0.0),
        from_time_seconds: time_seconds.max(0.0),
        to_time_seconds: delta_seconds,
    })
}

fn sample_state_transition_pose(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    state_machine: &AnimationStateMachineAsset,
    parameters: &AnimationParameterMap,
    pending: &PendingStateMachinePoseSample,
    transition: &AnimationStateTransitionRuntime,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let from_graph = state_machine_graph_reference(state_machine, &transition.from_state)?;
    let to_graph = state_machine_graph_reference(state_machine, &transition.to_state)?;
    let (_, from_pose) = sample_state_graph_pose(
        animation,
        asset_manager,
        state_machine,
        Some(from_graph),
        parameters,
        pending.entity,
        pending.skeleton_id,
        transition.from_time_seconds,
        Some(transition.from_state.clone()),
    )?;
    let (_, to_pose) = sample_state_graph_pose(
        animation,
        asset_manager,
        state_machine,
        Some(to_graph),
        parameters,
        pending.entity,
        pending.skeleton_id,
        transition.to_time_seconds,
        Some(transition.to_state.clone()),
    )?;
    let progress = (transition.elapsed_seconds / transition.duration_seconds).clamp(0.0, 1.0);
    blend_weighted_poses(
        vec![(from_pose, 1.0 - progress), (to_pose, progress)],
        AnimationPoseSource::StateMachine,
        Some(if progress >= 1.0 {
            transition.to_state.clone()
        } else {
            transition.from_state.clone()
        }),
    )
    .map(|pose| (pending.entity, pose))
}

fn sample_state_transition_clip_events(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    state_machine: &AnimationStateMachineAsset,
    parameters: &AnimationParameterMap,
    pending: &PendingStateMachinePoseSample,
    transition: &AnimationStateTransitionRuntime,
) -> Vec<AnimationClipEvent> {
    let mut events = Vec::new();
    let from_graph = state_machine_graph_reference(state_machine, &transition.from_state);
    let to_graph = state_machine_graph_reference(state_machine, &transition.to_state);
    let (from_start_seconds, to_start_seconds) = pending
        .transition
        .as_ref()
        .map(|previous| (previous.from_time_seconds, previous.to_time_seconds))
        .unwrap_or((pending.from_time_seconds, 0.0));

    events.extend(sample_state_graph_clip_events(
        animation,
        asset_manager,
        from_graph,
        parameters,
        pending.entity,
        from_start_seconds,
        transition.from_time_seconds,
    ));
    events.extend(sample_state_graph_clip_events(
        animation,
        asset_manager,
        to_graph,
        parameters,
        pending.entity,
        to_start_seconds,
        transition.to_time_seconds,
    ));
    events
}

fn sample_state_graph_clip_events(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    graph_reference: Option<&AssetReference>,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
) -> Vec<AnimationClipEvent> {
    let Some(graph_reference) = graph_reference else {
        return Vec::new();
    };
    let Some(graph_id) = asset_manager.resolve_asset_id(&graph_reference.locator) else {
        return Vec::new();
    };
    let Ok(graph) = asset_manager.load_animation_graph_asset(graph_id) else {
        return Vec::new();
    };
    let graph_evaluation = animation.evaluate_graph(&graph, parameters);
    sample_graph_evaluation_clip_events(
        asset_manager,
        entity,
        from_time_seconds,
        to_time_seconds,
        &graph_evaluation,
    )
}

fn sample_state_graph_pose(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    _state_machine: &AnimationStateMachineAsset,
    graph_reference: Option<&AssetReference>,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    skeleton_id: crate::asset::AssetId,
    time_seconds: Real,
    active_state: Option<String>,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let graph_reference = graph_reference?;
    let graph_id = asset_manager.resolve_asset_id(&graph_reference.locator)?;
    let graph = asset_manager.load_animation_graph_asset(graph_id).ok()?;
    let graph_evaluation = animation.evaluate_graph(&graph, parameters);
    sample_graph_evaluation_pose(
        animation,
        asset_manager,
        entity,
        skeleton_id,
        time_seconds,
        AnimationPoseSource::StateMachine,
        active_state,
        &graph_evaluation,
    )
}

fn state_machine_graph_reference<'a>(
    state_machine: &'a AnimationStateMachineAsset,
    state_name: &str,
) -> Option<&'a AssetReference> {
    state_machine
        .states
        .iter()
        .find(|state| state.name == state_name)
        .map(|state| &state.graph)
}
