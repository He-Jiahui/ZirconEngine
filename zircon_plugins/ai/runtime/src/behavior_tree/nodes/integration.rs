use std::collections::BTreeMap;

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeParameter, AiBehaviorNodeParameterValue, AiDecisionStatus,
};
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use zircon_runtime::core::framework::script::{
    ScriptBehaviorBridge, ScriptBehaviorCallbackRef, ScriptHostValue,
};
use zircon_runtime::plugin::BridgeImport;
use zircon_runtime::scene::World;

use crate::manager::parameters::{
    parse_task_result, ANIMATION_PARAMETER_PARAMETER_KEY, ANIMATION_TRIGGER_PARAMETER_KEY,
    ANIMATION_VALUE_PARAMETER_KEY, MOVE_TARGET_PARAMETER_KEY, SCRIPT_CALLBACK_PARAMETER_KEY,
};

pub(crate) struct BehaviorIntegrationTaskContext<'a> {
    pub(crate) node_id: &'a str,
    pub(crate) parameters: &'a [AiBehaviorNodeParameter],
    pub(crate) entity: u64,
    pub(crate) delta_seconds: f32,
    pub(crate) started: bool,
}

impl BehaviorIntegrationTaskContext<'_> {
    fn parameter(&self, key: &str) -> Option<&AiBehaviorNodeParameterValue> {
        self.parameters
            .iter()
            .find(|parameter| parameter.key == key)
            .map(|parameter| &parameter.value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntegrationTaskResult {
    pub(crate) status: AiDecisionStatus,
    pub(crate) diagnostic: Option<String>,
}

impl IntegrationTaskResult {
    fn succeeded() -> Self {
        Self::with_status(AiDecisionStatus::Succeeded)
    }

    fn running() -> Self {
        Self::with_status(AiDecisionStatus::Running)
    }

    fn failed(message: impl Into<String>) -> Self {
        Self::with_diagnostic(AiDecisionStatus::Failed, message)
    }

    fn blocked(message: impl Into<String>) -> Self {
        Self::with_diagnostic(AiDecisionStatus::Blocked, message)
    }

    fn with_status(status: AiDecisionStatus) -> Self {
        Self {
            status,
            diagnostic: None,
        }
    }

    fn with_diagnostic(status: AiDecisionStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            diagnostic: Some(message.into()),
        }
    }
}

pub(crate) trait BehaviorIntegrationHost {
    fn move_to(&mut self, context: &BehaviorIntegrationTaskContext<'_>) -> IntegrationTaskResult;

    fn play_animation(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult;

    fn script_task(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult;

    fn abort(&mut self, _context: &BehaviorIntegrationTaskContext<'_>) {}
}

pub(crate) struct RuntimeBehaviorIntegrationHost<'world> {
    world: &'world mut World,
    script: Option<BridgeImport<dyn ScriptBehaviorBridge>>,
    navigation_available: bool,
    navigation_feedback: BTreeMap<u64, NavigationAgentOutcome>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NavigationAgentOutcome {
    Arrived([f32; 3]),
    NoPath([f32; 3]),
}

impl<'world> RuntimeBehaviorIntegrationHost<'world> {
    pub(crate) fn new(
        world: &'world mut World,
        script: Option<BridgeImport<dyn ScriptBehaviorBridge>>,
    ) -> Self {
        let navigation_events = world.events::<NavAgentTickReport>();
        let navigation_available = navigation_events.is_some();
        let mut navigation_feedback = BTreeMap::new();
        for report in navigation_events
            .into_iter()
            .flat_map(|events| events.iter())
        {
            for (entity, destination) in &report.arrived_agents {
                navigation_feedback.insert(*entity, NavigationAgentOutcome::Arrived(*destination));
            }
            for (entity, destination) in &report.no_path_agents {
                navigation_feedback.insert(*entity, NavigationAgentOutcome::NoPath(*destination));
            }
        }
        Self {
            world,
            script,
            navigation_available,
            navigation_feedback,
        }
    }

    fn nav_target(&self, entity: u64) -> Result<Option<[f32; 3]>, String> {
        let value = self
            .world
            .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
            .ok_or_else(|| format!("entity {entity} does not have a NavMeshAgent component"))?;
        let Some(values) = value.get("destination").and_then(|value| value.as_array()) else {
            return Ok(None);
        };
        if values.len() != 3 {
            return Err(format!(
                "entity {entity} has an invalid NavMeshAgent destination"
            ));
        }
        let mut target = [0.0; 3];
        for (index, value) in values.iter().enumerate() {
            target[index] = value.as_f64().ok_or_else(|| {
                format!("entity {entity} has a non-numeric NavMeshAgent destination")
            })? as f32;
        }
        Ok(Some(target))
    }

    fn write_nav_target(&mut self, entity: u64, target: Option<[f32; 3]>) -> Result<(), String> {
        let target = target.unwrap_or_else(|| {
            self.world
                .world_transform(entity)
                .map(|transform| transform.translation.to_array())
                .unwrap_or_default()
        });
        let path = ComponentPropertyPath::new(
            NAV_MESH_AGENT_COMPONENT_TYPE,
            vec!["destination".to_string()],
        )
        .map_err(|error| error.to_string())?;
        self.world
            .set_property(entity, &path, ScenePropertyValue::Vec3(target))
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn clear_nav_target(&mut self, entity: u64) -> Result<(), String> {
        if self.nav_target(entity)?.is_none() {
            return Ok(());
        }
        self.write_nav_target(entity, None)
    }
}

impl BehaviorIntegrationHost for RuntimeBehaviorIntegrationHost<'_> {
    fn move_to(&mut self, context: &BehaviorIntegrationTaskContext<'_>) -> IntegrationTaskResult {
        if !self.navigation_available {
            return IntegrationTaskResult::blocked(format!(
                "AI MoveTo node `{}` cannot run because navigation is unavailable",
                context.node_id
            ));
        }
        let Some(AiBehaviorNodeParameterValue::Vec3(target)) =
            context.parameter(MOVE_TARGET_PARAMETER_KEY)
        else {
            return IntegrationTaskResult::failed(format!(
                "AI MoveTo node `{}` requires a vec3 `target` parameter",
                context.node_id
            ));
        };
        let target = target.to_array();
        let current_target = match self.nav_target(context.entity) {
            Ok(target) => target,
            Err(error) => return IntegrationTaskResult::failed(error),
        };
        if context.started || current_target != Some(target) {
            if let Err(error) = self.write_nav_target(context.entity, Some(target)) {
                return IntegrationTaskResult::failed(error);
            }
            return IntegrationTaskResult::running();
        }
        if let Some(feedback) = self.navigation_feedback.get(&context.entity) {
            match feedback {
                NavigationAgentOutcome::NoPath(destination)
                    if squared_distance(*destination, target) <= f32::EPSILON =>
                {
                    if let Err(error) = self.clear_nav_target(context.entity) {
                        return IntegrationTaskResult::failed(error);
                    }
                    return IntegrationTaskResult::failed(format!(
                        "AI MoveTo node `{}` could not reach its target",
                        context.node_id
                    ));
                }
                NavigationAgentOutcome::Arrived(destination)
                    if squared_distance(*destination, target) <= f32::EPSILON =>
                {
                    if let Err(error) = self.clear_nav_target(context.entity) {
                        return IntegrationTaskResult::failed(error);
                    }
                    return IntegrationTaskResult::succeeded();
                }
                NavigationAgentOutcome::Arrived(_) | NavigationAgentOutcome::NoPath(_) => {}
            }
        }
        IntegrationTaskResult::running()
    }

    fn play_animation(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult {
        let (parameter_name, value) = match animation_parameter(context) {
            Ok(parameter) => parameter,
            Err(error) => return IntegrationTaskResult::failed(error),
        };
        if let Some(mut player) = self
            .world
            .animation_state_machine_player(context.entity)
            .cloned()
        {
            player.parameters.insert(parameter_name, value);
            player.playing = true;
            return match self
                .world
                .set_animation_state_machine_player(context.entity, Some(player))
            {
                Ok(_) => IntegrationTaskResult::succeeded(),
                Err(error) => IntegrationTaskResult::failed(error.to_string()),
            };
        }
        if let Some(mut player) = self.world.animation_graph_player(context.entity).cloned() {
            player.parameters.insert(parameter_name, value);
            player.playing = true;
            return match self
                .world
                .set_animation_graph_player(context.entity, Some(player))
            {
                Ok(_) => IntegrationTaskResult::succeeded(),
                Err(error) => IntegrationTaskResult::failed(error.to_string()),
            };
        }
        IntegrationTaskResult::failed(format!(
            "AI PlayAnimation node `{}` requires an animation state-machine or graph player on entity {}",
            context.node_id, context.entity
        ))
    }

    fn script_task(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult {
        let Some(callback_id) = context
            .parameter(SCRIPT_CALLBACK_PARAMETER_KEY)
            .and_then(AiBehaviorNodeParameterValue::as_string)
        else {
            return IntegrationTaskResult::failed(format!(
                "AI ScriptTask node `{}` requires a string `callback` parameter",
                context.node_id
            ));
        };
        let callback = match ScriptBehaviorCallbackRef::parse(callback_id) {
            Ok(callback) => callback,
            Err(error) => return IntegrationTaskResult::failed(error.message),
        };
        let Some(bridge) = self.script.as_ref() else {
            return IntegrationTaskResult::blocked(format!(
                "AI ScriptTask node `{}` cannot run because the VM is unavailable",
                context.node_id
            ));
        };
        let arguments = [
            ScriptHostValue::HostHandle(context.entity),
            ScriptHostValue::Float(f64::from(context.delta_seconds)),
        ];
        let invocation = match bridge.call(|bridge| bridge.invoke(&callback, &arguments)) {
            Ok(invocation) => invocation,
            Err(error) => {
                return IntegrationTaskResult::blocked(format!(
                "AI ScriptTask node `{}` cannot run because script behavior bridge is {error:?}",
                context.node_id
            ))
            }
        };
        match invocation {
            Ok(None | Some(ScriptHostValue::Null)) => IntegrationTaskResult::succeeded(),
            Ok(Some(ScriptHostValue::Bool(true))) => IntegrationTaskResult::succeeded(),
            Ok(Some(ScriptHostValue::Bool(false))) => IntegrationTaskResult::failed(format!(
                "VM behavior callback `{callback_id}` reported failure"
            )),
            Ok(Some(ScriptHostValue::String(status))) => parse_task_result(&status)
                .map(IntegrationTaskResult::with_status)
                .unwrap_or_else(|| {
                    IntegrationTaskResult::failed(format!(
                        "VM behavior callback `{callback_id}` returned unknown status `{status}`"
                    ))
                }),
            Ok(Some(value)) => IntegrationTaskResult::failed(format!(
                "VM behavior callback `{callback_id}` returned unsupported value kind {:?}",
                value.kind()
            )),
            Err(error) => IntegrationTaskResult::failed(format!(
                "VM behavior callback `{callback_id}` failed: {}",
                error.message
            )),
        }
    }

    fn abort(&mut self, context: &BehaviorIntegrationTaskContext<'_>) {
        if context.parameter(MOVE_TARGET_PARAMETER_KEY).is_some() {
            let _ = self.clear_nav_target(context.entity);
        }
    }
}

fn animation_parameter(
    context: &BehaviorIntegrationTaskContext<'_>,
) -> Result<(String, AnimationParameterValue), String> {
    if let Some(trigger) = context
        .parameter(ANIMATION_TRIGGER_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
    {
        return Ok((trigger.to_string(), AnimationParameterValue::Trigger));
    }
    let Some(parameter) = context
        .parameter(ANIMATION_PARAMETER_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
    else {
        return Err(format!(
            "AI PlayAnimation node `{}` requires a string `parameter` or `trigger`",
            context.node_id
        ));
    };
    let value = match context.parameter(ANIMATION_VALUE_PARAMETER_KEY) {
        None => {
            return Err(format!(
                "AI PlayAnimation node `{}` requires a typed `value` for parameter `{parameter}`",
                context.node_id
            ))
        }
        Some(AiBehaviorNodeParameterValue::Bool(value)) => AnimationParameterValue::Bool(*value),
        Some(AiBehaviorNodeParameterValue::Integer(value)) => i32::try_from(*value)
            .map(AnimationParameterValue::Integer)
            .map_err(|_| {
                format!(
                    "AI PlayAnimation node `{}` integer `value` is outside i32 range",
                    context.node_id
                )
            })?,
        Some(AiBehaviorNodeParameterValue::Scalar(value)) => {
            AnimationParameterValue::Scalar(*value)
        }
        Some(AiBehaviorNodeParameterValue::Vec3(value)) => {
            AnimationParameterValue::Vec3(value.to_array())
        }
        Some(value) => {
            return Err(format!(
                "AI PlayAnimation node `{}` cannot map `{}` to an animation parameter",
                context.node_id,
                value.value_type()
            ))
        }
    };
    Ok((parameter.to_string(), value))
}

fn squared_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum()
}
