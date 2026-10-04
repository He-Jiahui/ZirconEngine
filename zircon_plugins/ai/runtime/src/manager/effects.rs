use zircon_runtime::core::framework::ai::{
    AiBehaviorEffectCommand, AiBehaviorEffectId, AiBehaviorEffectOutcome, AiBehaviorEffectReceipt,
    AiBlackboardEntry, AiBlackboardValue, AiGameplayEvent,
};

use crate::behavior_tree::BehaviorIntegrationHost;

use super::state::{AgentBlackboard, AiRuntimeState};
use super::DefaultAiManager;

const MAX_RETAINED_EFFECT_IDS: usize = 4_096;
const MAX_EFFECTS_PER_TICK: usize = 256;
const MAX_BLACKBOARD_KEY_BYTES: usize = 256;
const MAX_EFFECT_STRING_BYTES: usize = 1_024;
const MAX_GAMEPLAY_EVENT_NAME_BYTES: usize = 128;

pub(super) fn commit_behavior_effects(
    manager: &DefaultAiManager,
    commands: Vec<AiBehaviorEffectCommand>,
    blackboard: &mut AgentBlackboard,
    integration_host: Option<&mut dyn BehaviorIntegrationHost>,
) -> Result<(), String> {
    if commands.is_empty() {
        return Ok(());
    }
    if commands.len() > MAX_EFFECTS_PER_TICK {
        return Err("AI behavior effect batch exceeds its per-tick command budget".to_string());
    }

    validate_effect_commands(&commands, blackboard)?;
    let has_gameplay_events = commands
        .iter()
        .any(|command| matches!(command, AiBehaviorEffectCommand::EmitEvent { .. }));
    let mut host = integration_host.ok_or_else(|| {
        "AI behavior effect sink is unavailable because the integration host is missing".to_string()
    })?;
    host.can_publish_behavior_effects(has_gameplay_events, true)?;

    let mut state = manager.lock_state();
    let (next_blackboard, gameplay_events, receipts, committed_ids) =
        prepare_effect_commit(&commands, blackboard, &state)?;
    for id in committed_ids {
        remember_effect_id(&mut state, id);
    }
    drop(state);

    host.publish_behavior_effects(&gameplay_events, &receipts);
    *blackboard = next_blackboard;
    Ok(())
}

fn validate_effect_commands(
    commands: &[AiBehaviorEffectCommand],
    blackboard: &AgentBlackboard,
) -> Result<(), String> {
    let mut shadow = blackboard.clone();
    for command in commands {
        match command {
            AiBehaviorEffectCommand::SetBlackboard { key, value, .. } => {
                validate_key_and_value(key, value)?;
                write_blackboard(&mut shadow, key, value.clone())?;
            }
            AiBehaviorEffectCommand::EmitEvent { name, payload, .. } => {
                validate_event(name, payload.as_ref())?;
            }
        }
    }
    Ok(())
}

fn prepare_effect_commit(
    commands: &[AiBehaviorEffectCommand],
    blackboard: &AgentBlackboard,
    state: &AiRuntimeState,
) -> Result<
    (
        AgentBlackboard,
        Vec<AiGameplayEvent>,
        Vec<AiBehaviorEffectReceipt>,
        Vec<AiBehaviorEffectId>,
    ),
    String,
> {
    let mut next_blackboard = blackboard.clone();
    let mut gameplay_events = Vec::new();
    let mut receipts = Vec::with_capacity(commands.len());
    let mut committed_ids = Vec::with_capacity(commands.len());
    let mut batch_ids = std::collections::HashSet::with_capacity(commands.len());

    for command in commands {
        let effect_id = effect_id(command);
        if state.committed_effect_ids.contains(effect_id) || !batch_ids.insert(effect_id.clone()) {
            receipts.push(AiBehaviorEffectReceipt {
                effect_id: effect_id.clone(),
                outcome: AiBehaviorEffectOutcome::DuplicateSuppressed,
            });
            continue;
        }

        let outcome = match command {
            AiBehaviorEffectCommand::SetBlackboard { key, value, .. } => {
                let changed = write_blackboard(&mut next_blackboard, key, value.clone())?;
                AiBehaviorEffectOutcome::BlackboardWrite { changed }
            }
            AiBehaviorEffectCommand::EmitEvent { name, payload, .. } => {
                gameplay_events.push(AiGameplayEvent {
                    effect_id: effect_id.clone(),
                    name: name.clone(),
                    payload: payload.clone(),
                });
                AiBehaviorEffectOutcome::GameplayEventQueued
            }
        };
        committed_ids.push(effect_id.clone());
        receipts.push(AiBehaviorEffectReceipt {
            effect_id: effect_id.clone(),
            outcome,
        });
    }

    Ok((next_blackboard, gameplay_events, receipts, committed_ids))
}

fn effect_id(command: &AiBehaviorEffectCommand) -> &AiBehaviorEffectId {
    match command {
        AiBehaviorEffectCommand::SetBlackboard { effect_id, .. }
        | AiBehaviorEffectCommand::EmitEvent { effect_id, .. } => effect_id,
    }
}

fn remember_effect_id(state: &mut AiRuntimeState, effect_id: AiBehaviorEffectId) {
    if !state.committed_effect_ids.insert(effect_id.clone()) {
        return;
    }
    state.committed_effect_order.push_back(effect_id);
    while state.committed_effect_order.len() > MAX_RETAINED_EFFECT_IDS {
        if let Some(expired) = state.committed_effect_order.pop_front() {
            state.committed_effect_ids.remove(&expired);
        }
    }
}

fn validate_key_and_value(key: &str, value: &AiBlackboardValue) -> Result<(), String> {
    if key.trim().is_empty() || key.len() > MAX_BLACKBOARD_KEY_BYTES {
        return Err("AI SetBlackboard key is empty or exceeds its size limit".to_string());
    }
    if !value.is_finite() {
        return Err("AI SetBlackboard value must be finite".to_string());
    }
    if matches!(value, AiBlackboardValue::String(text) if text.len() > MAX_EFFECT_STRING_BYTES) {
        return Err("AI SetBlackboard string value exceeds its size limit".to_string());
    }
    Ok(())
}

fn validate_event(name: &str, payload: Option<&AiBlackboardValue>) -> Result<(), String> {
    if name.trim().is_empty()
        || name.len() > MAX_GAMEPLAY_EVENT_NAME_BYTES
        || name.chars().any(char::is_control)
    {
        return Err(
            "AI gameplay event name is empty, invalid, or exceeds its size limit".to_string(),
        );
    }
    if let Some(payload) = payload {
        validate_key_and_value("payload", payload)?;
    }
    Ok(())
}

fn write_blackboard(
    blackboard: &mut AgentBlackboard,
    key: &str,
    value: AiBlackboardValue,
) -> Result<bool, String> {
    match blackboard {
        AgentBlackboard::Dynamic(entries) => {
            if let Some(entry) = entries.iter_mut().find(|entry| entry.key == key) {
                let changed = entry.value != value;
                entry.value = value;
                Ok(changed)
            } else {
                entries.push(AiBlackboardEntry::new(key, value));
                entries.sort_by(|left, right| left.key.cmp(&right.key));
                Ok(true)
            }
        }
        AgentBlackboard::Dense(store) => store
            .write(key, value)
            .map(|outcome| outcome.changed)
            .map_err(|error| error.to_string()),
    }
}

#[cfg(test)]
#[path = "tests/effects.rs"]
mod tests;
