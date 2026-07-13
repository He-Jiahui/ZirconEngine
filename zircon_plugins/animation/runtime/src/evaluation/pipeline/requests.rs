use zircon_runtime::asset::AssetId;
use zircon_runtime::core::framework::animation::{AnimationParameterMap, AnimationPoseSource};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::{AnimationStateTransitionRuntime, EntityId};

#[derive(Clone, Debug, Default)]
pub(super) struct AnimationSceneScan {
    pub(super) sequences: Vec<PendingSequenceSample>,
    pub(super) clip_samples: Vec<PendingPoseSample>,
    pub(super) clip_event_samples: Vec<PendingClipEventSample>,
    pub(super) graph_samples: Vec<PendingGraphPoseSample>,
    pub(super) state_machine_samples: Vec<PendingStateMachinePoseSample>,
    pub(super) skeletons_by_entity:
        std::collections::BTreeMap<EntityId, zircon_runtime::asset::AssetId>,
    pub(super) next_graph_times: std::collections::BTreeMap<EntityId, Real>,
    pub(super) next_state_machine_times: std::collections::BTreeMap<EntityId, Real>,
}

#[derive(Clone, Debug)]
pub(super) struct PendingSequenceSample {
    pub(super) sequence_id: AssetId,
    pub(super) time_seconds: Real,
    pub(super) looping: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PendingPoseSample {
    pub(super) entity: EntityId,
    pub(super) skeleton_id: AssetId,
    pub(super) clip_id: AssetId,
    pub(super) time_seconds: Real,
    pub(super) looping: bool,
    pub(super) source: AnimationPoseSource,
    pub(super) active_state: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct PendingClipEventSample {
    pub(super) entity: EntityId,
    pub(super) clip_id: AssetId,
    pub(super) from_time_seconds: Real,
    pub(super) to_time_seconds: Real,
    pub(super) looping: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PendingGraphPoseSample {
    pub(super) entity: EntityId,
    pub(super) skeleton_id: AssetId,
    pub(super) graph_id: AssetId,
    pub(super) parameters: AnimationParameterMap,
    pub(super) from_time_seconds: Real,
    pub(super) to_time_seconds: Real,
}

#[derive(Clone, Debug)]
pub(super) struct PendingStateMachinePoseSample {
    pub(super) entity: EntityId,
    pub(super) skeleton_id: AssetId,
    pub(super) state_machine_id: AssetId,
    pub(super) parameters: AnimationParameterMap,
    pub(super) active_state: Option<String>,
    pub(super) from_time_seconds: Real,
    pub(super) to_time_seconds: Real,
    pub(super) delta_seconds: Real,
    pub(super) transition: Option<AnimationStateTransitionRuntime>,
}
