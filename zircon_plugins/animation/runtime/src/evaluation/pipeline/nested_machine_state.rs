use std::collections::BTreeSet;

use zircon_runtime::scene::EntityId;

use super::AnimationEvaluationPipeline;

impl AnimationEvaluationPipeline {
    pub(super) fn retain_nested_machine_instances(&mut self, active: &BTreeSet<EntityId>) {
        self.nested_machine_states
            .retain(|instance, _| active.contains(&instance.entity()));
        self.nested_machine_transitions
            .retain(|instance, _| active.contains(&instance.entity()));
    }
}
