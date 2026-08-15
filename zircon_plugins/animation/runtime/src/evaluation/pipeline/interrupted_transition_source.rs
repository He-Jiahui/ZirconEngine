use std::collections::BTreeSet;
use std::sync::Arc;

use super::machine_instance_key::MachineInstanceKey;
use super::AnimationEvaluationPipeline;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::scene::EntityId;

#[derive(Clone, Debug)]
pub(super) struct InterruptedTransitionSource {
    pub(super) from_state: String,
    pub(super) to_state: String,
    pub(super) pose: Arc<AnimationPoseOutput>,
}

impl AnimationEvaluationPipeline {
    pub(super) fn record_interrupted_transition_source(
        &mut self,
        instance: MachineInstanceKey,
        from_state: &str,
        to_state: &str,
        pose: AnimationPoseOutput,
    ) {
        self.interrupted_transition_sources.insert(
            instance,
            InterruptedTransitionSource {
                from_state: from_state.to_string(),
                to_state: to_state.to_string(),
                pose: Arc::new(pose),
            },
        );
    }

    pub(super) fn interrupted_transition_source(
        &self,
        instance: &MachineInstanceKey,
        from_state: &str,
        to_state: &str,
    ) -> Option<Arc<AnimationPoseOutput>> {
        self.interrupted_transition_sources
            .get(instance)
            .filter(|source| source.from_state == from_state && source.to_state == to_state)
            .map(|source| Arc::clone(&source.pose))
    }

    pub(super) fn clear_interrupted_transition_source(&mut self, instance: &MachineInstanceKey) {
        self.interrupted_transition_sources.remove(instance);
    }

    pub(super) fn retain_interrupted_transition_sources(&mut self, active: &BTreeSet<EntityId>) {
        self.interrupted_transition_sources
            .retain(|instance, _| active.contains(&instance.entity()));
    }
}
