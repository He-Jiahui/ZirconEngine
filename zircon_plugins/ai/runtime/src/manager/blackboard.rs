use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiManagerError,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use std::sync::Arc;

use super::state::{AgentBlackboard, RegisteredBlackboardSchema};
use super::validation::{validate_blackboard_entries, validate_blackboard_schema_descriptor};
use super::DefaultAiManager;
use crate::blackboard::BlackboardLayout;
use crate::blackboard::{BlackboardLayoutError, BlackboardRuntimeError, BlackboardStore};

pub(super) fn register_schema(
    manager: &DefaultAiManager,
    descriptor: AiBlackboardSchemaDescriptor,
) -> Result<AiBlackboardSchemaId, AiManagerError> {
    validate_blackboard_schema_descriptor(&descriptor)?;

    let mut state = manager.lock_state();
    if state
        .blackboard_schemas
        .iter()
        .any(|entry| entry.descriptor.id == descriptor.id)
    {
        return Err(AiManagerError::DuplicateId { id: descriptor.id });
    }

    state.next_blackboard_schema_id += 1;
    let id = AiBlackboardSchemaId::new(state.next_blackboard_schema_id);
    let layout = Arc::new(
        BlackboardLayout::from_schema(&descriptor)
            .map_err(|error| map_layout_error(&descriptor.id, error))?,
    );
    state.blackboard_schemas.push(RegisteredBlackboardSchema {
        id,
        descriptor,
        layout,
    });
    Ok(id)
}

fn map_layout_error(schema_id: &str, error: BlackboardLayoutError) -> AiManagerError {
    match error {
        BlackboardLayoutError::DuplicateKey { key } => AiManagerError::DuplicateBlackboardKey {
            schema_id: schema_id.to_string(),
            key,
        },
        BlackboardLayoutError::UnknownValueType { key, value_type } => {
            AiManagerError::UnknownBlackboardValueType {
                schema_id: schema_id.to_string(),
                key,
                value_type,
            }
        }
    }
}

pub(super) fn schemas(manager: &DefaultAiManager) -> Vec<AiBlackboardSchemaDescriptor> {
    manager
        .lock_state()
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
    let mut state = manager.lock_state();
    let active_schema = state
        .active_behavior_trees
        .get(&(world, entity))
        .and_then(|active| active.blackboard_schema)
        .and_then(|schema_id| {
            state
                .blackboard_schemas
                .iter()
                .find(|schema| schema.id == schema_id)
                .cloned()
        });
    validate_blackboard_entries(
        active_schema.as_ref().map(|schema| &schema.descriptor),
        &entries,
    )?;
    if let Some(schema) = active_schema {
        let blackboard = state
            .blackboards
            .entry((world, entity))
            .or_insert_with(|| AgentBlackboard::Dense(BlackboardStore::new(schema.layout.clone())));
        if !matches!(
            blackboard,
            AgentBlackboard::Dense(store) if store.layout().schema_id() == schema.layout.schema_id()
        ) {
            *blackboard = AgentBlackboard::Dense(BlackboardStore::new(schema.layout.clone()));
        }
        if let AgentBlackboard::Dense(store) = blackboard {
            store
                .synchronize(&entries)
                .map_err(|error| map_runtime_error(schema.layout.schema_id(), error))?;
        }
    } else {
        state
            .blackboards
            .insert((world, entity), AgentBlackboard::Dynamic(entries));
    }
    Ok(())
}

pub(super) fn entries(
    manager: &DefaultAiManager,
    world: WorldHandle,
    entity: EntityId,
) -> Vec<AiBlackboardEntry> {
    manager
        .lock_state()
        .blackboards
        .get(&(world, entity))
        .map(AgentBlackboard::entries)
        .unwrap_or_default()
}

pub(super) fn map_runtime_error(schema_id: &str, error: BlackboardRuntimeError) -> AiManagerError {
    match error {
        BlackboardRuntimeError::UnknownKey { key } => AiManagerError::UnknownBlackboardKey {
            schema_id: schema_id.to_string(),
            key,
        },
        BlackboardRuntimeError::DuplicateKey { key } => {
            AiManagerError::DuplicateBlackboardEntry { key }
        }
        BlackboardRuntimeError::TypeMismatch {
            key,
            expected,
            actual,
        } => AiManagerError::BlackboardValueTypeMismatch {
            schema_id: schema_id.to_string(),
            key,
            expected: expected.as_str().to_string(),
            actual: actual.as_str().to_string(),
        },
        BlackboardRuntimeError::NonFiniteValue { key } => {
            AiManagerError::NonFiniteBlackboardValue { key }
        }
    }
}
