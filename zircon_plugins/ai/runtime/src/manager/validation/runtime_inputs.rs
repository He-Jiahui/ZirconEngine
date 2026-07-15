use std::collections::HashSet;

use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiManagerError, AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::scene::EntityId;

use super::ensure_non_empty;

pub(in crate::manager) fn validate_blackboard_schema_descriptor(
    descriptor: &AiBlackboardSchemaDescriptor,
) -> Result<(), AiManagerError> {
    ensure_non_empty(&descriptor.id, "blackboard_schema.id")?;

    let mut keys = HashSet::new();
    for key in &descriptor.keys {
        ensure_non_empty(&key.key, "blackboard_key.key")?;
        ensure_non_empty(&key.value_type, "blackboard_key.value_type")?;
        if key.expected_value_type().is_none() {
            return Err(AiManagerError::UnknownBlackboardValueType {
                schema_id: descriptor.id.clone(),
                key: key.key.clone(),
                value_type: key.value_type.clone(),
            });
        }
        if !keys.insert(key.key.as_str()) {
            return Err(AiManagerError::DuplicateBlackboardKey {
                schema_id: descriptor.id.clone(),
                key: key.key.clone(),
            });
        }
    }
    Ok(())
}

pub(in crate::manager) fn validate_blackboard_entries(
    schema: Option<&AiBlackboardSchemaDescriptor>,
    entries: &[AiBlackboardEntry],
) -> Result<(), AiManagerError> {
    let mut seen_entries = HashSet::new();
    for entry in entries {
        ensure_non_empty(&entry.key, "blackboard_entry.key")?;
        if !entry.value.is_finite() {
            return Err(AiManagerError::NonFiniteBlackboardValue {
                key: entry.key.clone(),
            });
        }
        if !seen_entries.insert(entry.key.as_str()) {
            return Err(AiManagerError::DuplicateBlackboardEntry {
                key: entry.key.clone(),
            });
        }
    }

    let Some(schema) = schema else {
        return Ok(());
    };
    for descriptor in &schema.keys {
        let matching_entry = entries.iter().find(|entry| entry.key == descriptor.key);
        if descriptor.required && matching_entry.is_none() {
            return Err(AiManagerError::MissingBlackboardKey {
                schema_id: schema.id.clone(),
                key: descriptor.key.clone(),
            });
        }
        if let Some(entry) = matching_entry {
            let Some(expected) = descriptor.expected_value_type() else {
                return Err(AiManagerError::UnknownBlackboardValueType {
                    schema_id: schema.id.clone(),
                    key: descriptor.key.clone(),
                    value_type: descriptor.value_type.clone(),
                });
            };
            let actual = entry.value.value_type();
            if expected != actual {
                return Err(AiManagerError::BlackboardValueTypeMismatch {
                    schema_id: schema.id.clone(),
                    key: entry.key.clone(),
                    expected: expected.as_str().to_string(),
                    actual: actual.as_str().to_string(),
                });
            }
        }
    }
    for entry in entries {
        if !schema
            .keys
            .iter()
            .any(|descriptor| descriptor.key == entry.key)
        {
            return Err(AiManagerError::UnknownBlackboardKey {
                schema_id: schema.id.clone(),
                key: entry.key.clone(),
            });
        }
    }
    Ok(())
}

pub(in crate::manager) fn validate_perception_snapshot(
    entity: EntityId,
    snapshot: &AiPerceptionSnapshot,
) -> Result<(), AiManagerError> {
    if snapshot.agent != entity {
        return Err(AiManagerError::PerceptionAgentMismatch {
            expected: entity,
            actual: snapshot.agent,
        });
    }
    for stimulus in &snapshot.stimuli {
        if !stimulus.position.is_finite()
            || !stimulus.strength.is_finite()
            || !stimulus.age_seconds.is_finite()
        {
            return Err(AiManagerError::NonFinitePerceptionStimulus {
                source: stimulus.source,
            });
        }
    }
    Ok(())
}
