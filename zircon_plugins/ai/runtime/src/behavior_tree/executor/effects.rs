use std::collections::BTreeMap;

use zircon_runtime::core::framework::ai::{
    AiBehaviorEffectCommand, AiBehaviorEffectId, AiBehaviorNodeParameterValue, AiBehaviorTreeId,
    AiBlackboardValue, AiDecisionStatus,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use crate::blackboard::BlackboardStore;
use crate::manager::parameters::{
    BLACKBOARD_KEY_PARAMETER_KEY, BLACKBOARD_VALUE_PARAMETER_KEY,
    GAMEPLAY_EVENT_NAME_PARAMETER_KEY, GAMEPLAY_EVENT_PAYLOAD_PARAMETER_KEY,
};

use super::{
    blocked, node_result, parameter, BehaviorNodeSemantics, BehaviorTreeExecution,
    BehaviorTreeExecutionContext, CompiledBehaviorNode, CompiledBehaviorTree,
};

pub(super) const MAX_BEHAVIOR_EFFECTS_PER_TICK: usize = 256;
const MAX_BLACKBOARD_KEY_BYTES: usize = 256;
const MAX_GAMEPLAY_EVENT_NAME_BYTES: usize = 128;
const MAX_EFFECT_STRING_BYTES: usize = 1_024;

#[derive(Clone, Copy)]
pub(super) struct EffectExecutionIdentity {
    pub(super) world: WorldHandle,
    pub(super) entity: u64,
    pub(super) behavior_tree: AiBehaviorTreeId,
    pub(super) compiled_tree_generation: u64,
    pub(super) effect_generation: Option<u64>,
}

impl EffectExecutionIdentity {
    fn effect_id(
        self,
        instance_tick: u64,
        tree_id: &str,
        node_id: &str,
        ordinal: u32,
    ) -> Option<AiBehaviorEffectId> {
        Some(AiBehaviorEffectId {
            world: self.world,
            entity: self.entity,
            behavior_tree: self.behavior_tree,
            compiled_tree_generation: self.compiled_tree_generation,
            effect_generation: self.effect_generation?,
            tick: instance_tick,
            tree_id: tree_id.to_string(),
            node_id: node_id.to_string(),
            ordinal,
        })
    }
}

#[derive(Default)]
pub(super) struct BehaviorTreeEffectStaging {
    commands: Vec<AiBehaviorEffectCommand>,
    blackboard_overlay: BTreeMap<String, AiBlackboardValue>,
    dense_blackboard_shadow: Option<BlackboardStore>,
    failure: Option<(String, String)>,
}

impl BehaviorTreeEffectStaging {
    pub(super) fn blackboard_overlay(&self) -> &BTreeMap<String, AiBlackboardValue> {
        &self.blackboard_overlay
    }

    pub(super) fn finish(self) -> (Vec<AiBehaviorEffectCommand>, Option<(String, String)>) {
        (self.commands, self.failure)
    }

    fn fail(&mut self, node_id: &str, reason: impl Into<String>) {
        if self.failure.is_none() {
            self.failure = Some((node_id.to_string(), reason.into()));
        }
        self.commands.clear();
        self.blackboard_overlay.clear();
        self.dense_blackboard_shadow = None;
    }

    fn stage_blackboard_write(
        &mut self,
        blackboard_store: Option<&BlackboardStore>,
        key: &str,
        value: &AiBlackboardValue,
    ) -> Result<(), String> {
        let Some(blackboard_store) = blackboard_store else {
            return Ok(());
        };
        self.dense_blackboard_shadow
            .get_or_insert_with(|| blackboard_store.clone())
            .write(key, value.clone())
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn next_effect_id(
        &mut self,
        identity: Option<EffectExecutionIdentity>,
        instance_tick: u64,
        tree: &CompiledBehaviorTree,
        node: &CompiledBehaviorNode,
    ) -> Result<AiBehaviorEffectId, &'static str> {
        if self.commands.len() >= MAX_BEHAVIOR_EFFECTS_PER_TICK {
            return Err("exceeded the per-tick behavior effect budget");
        }
        let ordinal = u32::try_from(self.commands.len())
            .map_err(|_| "exceeded the per-tick behavior effect ordinal range")?;
        identity
            .and_then(|identity| identity.effect_id(instance_tick, tree.id(), node.id(), ordinal))
            .ok_or("cannot issue an effect ID because the AI effect generation is exhausted")
    }
}

pub(super) fn evaluate_effect_task(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) -> BehaviorTreeExecution {
    if context.effects.failure.is_some() {
        return blocked(
            node.id(),
            "cannot run because an earlier effect command was invalid",
        );
    }

    let effect_id = match context.effects.next_effect_id(
        context.effect_identity,
        context.instance.tick,
        tree,
        node,
    ) {
        Ok(effect_id) => effect_id,
        Err(reason) => {
            context.effects.fail(node.id(), reason);
            return blocked(node.id(), "cannot stage its typed effect command");
        }
    };

    let command = match node.semantics() {
        BehaviorNodeSemantics::SetBlackboard => match set_blackboard_command(node, effect_id) {
            Ok((command, key, value)) => {
                if let Err(reason) =
                    context
                        .effects
                        .stage_blackboard_write(context.blackboard_store, &key, &value)
                {
                    context.effects.fail(node.id(), reason);
                    return blocked(
                        node.id(),
                        "cannot stage its typed Blackboard value against the active schema",
                    );
                }
                context.effects.blackboard_overlay.insert(key, value);
                command
            }
            Err(reason) => {
                context.effects.fail(node.id(), reason);
                return blocked(node.id(), "has invalid typed Blackboard effect parameters");
            }
        },
        BehaviorNodeSemantics::EmitEvent => match emit_event_command(node, effect_id) {
            Ok(command) => command,
            Err(reason) => {
                context.effects.fail(node.id(), reason);
                return blocked(node.id(), "has invalid typed gameplay event parameters");
            }
        },
        _ => {
            context
                .effects
                .fail(node.id(), "does not identify a supported effect task");
            return blocked(node.id(), "does not identify a supported effect task");
        }
    };
    context.effects.commands.push(command);
    node_result(node, AiDecisionStatus::Succeeded)
}

fn set_blackboard_command(
    node: &CompiledBehaviorNode,
    effect_id: AiBehaviorEffectId,
) -> Result<(AiBehaviorEffectCommand, String, AiBlackboardValue), &'static str> {
    let key = parameter(node, BLACKBOARD_KEY_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
        .filter(|key| !key.trim().is_empty() && key.len() <= MAX_BLACKBOARD_KEY_BYTES)
        .ok_or("Blackboard key is missing, empty, or too long")?;
    let value = parameter(node, BLACKBOARD_VALUE_PARAMETER_KEY)
        .and_then(typed_blackboard_value)
        .filter(valid_effect_value)
        .ok_or("Blackboard value is missing, nonfinite, or too large")?;
    Ok((
        AiBehaviorEffectCommand::SetBlackboard {
            effect_id,
            key: key.to_string(),
            value: value.clone(),
        },
        key.to_string(),
        value,
    ))
}

fn emit_event_command(
    node: &CompiledBehaviorNode,
    effect_id: AiBehaviorEffectId,
) -> Result<AiBehaviorEffectCommand, &'static str> {
    let name = parameter(node, GAMEPLAY_EVENT_NAME_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
        .filter(|name| {
            !name.trim().is_empty()
                && name.len() <= MAX_GAMEPLAY_EVENT_NAME_BYTES
                && !name.chars().any(char::is_control)
        })
        .ok_or("gameplay event name is missing, invalid, or too long")?;
    let payload = match parameter(node, GAMEPLAY_EVENT_PAYLOAD_PARAMETER_KEY) {
        Some(value) => Some(
            typed_blackboard_value(value)
                .filter(valid_effect_value)
                .ok_or("gameplay event payload is nonfinite or too large")?,
        ),
        None => None,
    };
    Ok(AiBehaviorEffectCommand::EmitEvent {
        effect_id,
        name: name.to_string(),
        payload,
    })
}

fn typed_blackboard_value(value: &AiBehaviorNodeParameterValue) -> Option<AiBlackboardValue> {
    Some(match value {
        AiBehaviorNodeParameterValue::Bool(value) => AiBlackboardValue::Bool(*value),
        AiBehaviorNodeParameterValue::Integer(value) => AiBlackboardValue::Integer(*value),
        AiBehaviorNodeParameterValue::Scalar(value) => AiBlackboardValue::Scalar(*value),
        AiBehaviorNodeParameterValue::String(value) => AiBlackboardValue::String(value.clone()),
        AiBehaviorNodeParameterValue::Vec3(value) => AiBlackboardValue::Vec3(*value),
        AiBehaviorNodeParameterValue::Entity(value) => AiBlackboardValue::Entity(*value),
    })
}

fn valid_effect_value(value: &AiBlackboardValue) -> bool {
    value.is_finite()
        && !matches!(value, AiBlackboardValue::String(text) if text.len() > MAX_EFFECT_STRING_BYTES)
}

pub(super) fn failed_effect_execution(
    execution: &mut BehaviorTreeExecution,
    failure: Option<(String, String)>,
) {
    if let Some((node_id, diagnostic)) = failure {
        execution.status = AiDecisionStatus::Blocked;
        execution.active_node = Some(node_id);
        execution.diagnostic = Some(diagnostic);
    }
}
