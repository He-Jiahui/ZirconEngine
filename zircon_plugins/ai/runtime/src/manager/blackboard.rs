use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiManagerError,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use super::state::RegisteredBlackboardSchema;
use super::validation::{validate_blackboard_entries, validate_blackboard_schema_descriptor};
use super::DefaultAiManager;

pub(super) fn register_schema(
    manager: &DefaultAiManager,
    descriptor: AiBlackboardSchemaDescriptor,
) -> Result<AiBlackboardSchemaId, AiManagerError> {
    validate_blackboard_schema_descriptor(&descriptor)?;

    let mut state = manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned");
    if state
        .blackboard_schemas
        .iter()
        .any(|entry| entry.descriptor.id == descriptor.id)
    {
        return Err(AiManagerError::DuplicateId { id: descriptor.id });
    }

    state.next_blackboard_schema_id += 1;
    let id = AiBlackboardSchemaId::new(state.next_blackboard_schema_id);
    state
        .blackboard_schemas
        .push(RegisteredBlackboardSchema { id, descriptor });
    Ok(id)
}

pub(super) fn schemas(manager: &DefaultAiManager) -> Vec<AiBlackboardSchemaDescriptor> {
    manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned")
        .blackboard_schemas
        .iter()
        .map(|entry| entry.descriptor.clone())
        .collect()
}

pub(super) fn set_entries(
    manager: &DefaultAiManager,
    world: WorldHandle,
    entity: EntityId,
    entries: Vec<AiBlackboardEntry>,
) -> Result<(), AiManagerError> {
    validate_blackboard_entries(None, &entries)?;

    manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned")
        .blackboards
        .insert((world, entity), entries);
    Ok(())
}

pub(super) fn entries(
    manager: &DefaultAiManager,
    world: WorldHandle,
    entity: EntityId,
) -> Vec<AiBlackboardEntry> {
    manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned")
        .blackboards
        .get(&(world, entity))
        .cloned()
        .unwrap_or_default()
}
