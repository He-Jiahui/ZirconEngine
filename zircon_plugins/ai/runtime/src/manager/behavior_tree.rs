use zircon_runtime::core::framework::ai::{
    AiBehaviorTreeDescriptor, AiBehaviorTreeId, AiManagerError,
};

use super::state::RegisteredBehaviorTree;
use super::validation::validate_behavior_tree_descriptor;
use super::DefaultAiManager;

pub(super) fn register(
    manager: &DefaultAiManager,
    descriptor: AiBehaviorTreeDescriptor,
) -> Result<AiBehaviorTreeId, AiManagerError> {
    validate_behavior_tree_descriptor(&descriptor)?;

    let mut state = manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned");
    if state
        .behavior_trees
        .iter()
        .any(|entry| entry.descriptor.id == descriptor.id)
    {
        return Err(AiManagerError::DuplicateId { id: descriptor.id });
    }

    state.next_behavior_tree_id += 1;
    let id = AiBehaviorTreeId::new(state.next_behavior_tree_id);
    state
        .behavior_trees
        .push(RegisteredBehaviorTree { id, descriptor });
    Ok(id)
}

pub(super) fn descriptors(manager: &DefaultAiManager) -> Vec<AiBehaviorTreeDescriptor> {
    manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned")
        .behavior_trees
        .iter()
        .map(|entry| entry.descriptor.clone())
        .collect()
}
