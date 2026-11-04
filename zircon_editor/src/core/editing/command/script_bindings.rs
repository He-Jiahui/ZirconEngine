//! Typed, whole-array script authoring. Discovery is a UI aid, never a replay dependency.

use super::*;
use crate::core::editing::operation::{
    OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
};
use crate::core::editor_operation::EditorOperationInvocation;
use zircon_runtime::asset::SceneScriptBindingAsset;
use zircon_runtime_interface::retained_memory::*;

pub(crate) const SET_SCRIPT_BINDINGS_OPERATION: &str = "inspector.script_binding.set";
pub(crate) const SET_SCRIPT_BINDINGS_JOURNAL: &str = "zircon.editor.scene.set_script_bindings";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetScriptBindingsPayload {
    pub(crate) node_id: NodeId,
    pub(crate) before: Option<Vec<SceneScriptBindingAsset>>,
    pub(crate) after: Option<Vec<SceneScriptBindingAsset>>,
}
zircon_runtime_interface::impl_retained_owner_v1!(SetScriptBindingsPayload { before, after });

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetScriptBindingsRequest {
    pub(crate) scope: AddComponentScope,
    // Carry the captured before value so factory construction stays context-free and
    // the write cannot silently replace a binding changed since the form opened.
    pub(crate) bindings: SetScriptBindingsPayload,
}
impl SetScriptBindingsRequest {
    pub(crate) fn from_invocation(
        invocation: &EditorOperationInvocation,
    ) -> Result<Self, OperationCommandFactoryError> {
        serde_json::from_value(invocation.arguments.clone()).map_err(|error| {
            OperationCommandFactoryError::InvalidArguments {
                operation: invocation.operation_id.clone(),
                reason: error.to_string(),
            }
        })
    }

    pub(crate) fn validate(&self, core: &CoreEditContext) -> Result<(), EditCommandError> {
        self.scope.validate(core)?;
        self.validate_target()?;
        validate_bindings(self.bindings.after.as_deref())?;
        core.with_scene(|scene| {
            if read_script_bindings(scene, self.bindings.node_id)? != self.bindings.before {
                return Err(invalid("script bindings changed since capture"));
            }
            Ok(())
        })?
    }

    fn validate_target(&self) -> Result<(), EditCommandError> {
        if self.scope.targets.as_slice() != [self.bindings.node_id]
            || self.scope.primary != Some(self.bindings.node_id)
        {
            return Err(invalid(
                "script binding authoring requires one selected entity",
            ));
        }
        Ok(())
    }
}

pub(crate) struct SetScriptBindingsFactory;
impl OperationCommandFactory for SetScriptBindingsFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        let request = SetScriptBindingsRequest::from_invocation(invocation)?;
        let create = || {
            request.validate_target()?;
            if !matches!(
                request.scope.history,
                crate::core::editing::engine::HistoryContextId::Document(_)
            ) {
                return Err(invalid("script bindings require document history"));
            }
            let mut command = SetScriptBindingsCommand::from_journal(request.bindings.clone())?;
            command.initial_scope = Some(request.scope.clone());
            command.gateway = Some(request.scope.gateway.clone());
            Ok(OperationCommand::new(
                Box::new(EditorCommand::SetScriptBindings(command)),
                request.scope.history,
            ))
        };
        create().map_err(
            |error: EditCommandError| OperationCommandFactoryError::InvalidArguments {
                operation: invocation.operation_id.clone(),
                reason: error.to_string(),
            },
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SetScriptBindingsCommand {
    pub(super) payload: SetScriptBindingsPayload,
    initial_scope: Option<AddComponentScope>,
    gateway: Option<GatewayStamp>,
}
zircon_runtime_interface::impl_retained_owner_v1!(SetScriptBindingsCommand {
    payload,
    initial_scope,
    gateway
});

impl SetScriptBindingsCommand {
    pub(crate) fn capture(
        scene: &Scene,
        node_id: NodeId,
        after: Option<Vec<SceneScriptBindingAsset>>,
    ) -> Result<Option<Self>, EditCommandError> {
        let before = read_script_bindings(scene, node_id)?;
        let after = normalize(after);
        validate_bindings(after.as_deref())?;
        if before == after {
            return Ok(None);
        }
        Self::from_journal(SetScriptBindingsPayload {
            node_id,
            before,
            after,
        })
        .map(Some)
    }

    pub(crate) fn from_journal(
        mut payload: SetScriptBindingsPayload,
    ) -> Result<Self, EditCommandError> {
        payload.before = normalize(payload.before);
        payload.after = normalize(payload.after);
        validate_bindings(payload.after.as_deref())?;
        if payload.before == payload.after {
            return Err(invalid("script binding command must change the value"));
        }
        Ok(Self {
            payload,
            initial_scope: None,
            gateway: None,
        })
    }

    pub(crate) fn validate(
        &self,
        core: &CoreEditContext,
        undo: bool,
    ) -> Result<(), EditCommandError> {
        if let Some(gateway) = &self.gateway {
            super::add_component::validate_gateway(core, gateway)?;
        }
        if !undo {
            if let Some(scope) = &self.initial_scope {
                scope.validate(core)?;
            }
        }
        validate_bindings(self.payload.after.as_deref())?;
        core.with_scene(|scene| {
            let expected = if undo {
                &self.payload.after
            } else {
                &self.payload.before
            };
            if &read_script_bindings(scene, self.payload.node_id)? != expected {
                return Err(invalid("script bindings changed since capture"));
            }
            Ok(())
        })?
    }

    pub(super) fn apply(
        &mut self,
        core: &mut CoreEditContext,
    ) -> Result<(), CommandExecutionError> {
        self.validate(core, false).map_err(unchanged)?;
        let result = self.write(core, false);
        if result.is_ok()
            || result
                .as_ref()
                .is_err_and(|error| error.effect == CommandEffect::Applied)
        {
            self.initial_scope = None;
        }
        result
    }

    pub(super) fn revert(
        &mut self,
        core: &mut CoreEditContext,
    ) -> Result<(), CommandExecutionError> {
        self.validate(core, true).map_err(unchanged)?;
        self.write(core, true)
    }

    fn write(&self, core: &mut CoreEditContext, undo: bool) -> Result<(), CommandExecutionError> {
        match execute_scene_write(core, |scene| {
            let (expected, value) = if undo {
                (&self.payload.after, &self.payload.before)
            } else {
                (&self.payload.before, &self.payload.after)
            };
            if &read_script_bindings(scene, self.payload.node_id)? != expected {
                return Err(invalid("script bindings changed since capture"));
            }
            let result = match value {
                Some(bindings) => scene.set_dynamic_component(
                    self.payload.node_id,
                    SceneScriptBindingAsset::COMPONENT_TYPE_PATH,
                    serde_json::to_value(bindings)
                        .map_err(|error| reflect_error(error.to_string()))?,
                ),
                None => scene.remove_dynamic_component(
                    self.payload.node_id,
                    SceneScriptBindingAsset::COMPONENT_TYPE_PATH,
                ),
            };
            result.map_err(|error| scene_error("set script bindings", error))
        })? {
            SceneWriteCompletion::Completed(_) => Ok(()),
            SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => Err(applied(error)),
        }
    }

    pub(super) fn journal_payload(
        &self,
    ) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(SET_SCRIPT_BINDINGS_JOURNAL, &self.payload)
    }
}

pub(crate) fn read_script_bindings(
    scene: &Scene,
    node_id: NodeId,
) -> Result<Option<Vec<SceneScriptBindingAsset>>, EditCommandError> {
    if !scene.contains_entity(node_id) {
        return Err(EditCommandError::TargetMissing {
            target: format!("scene node {node_id}"),
        });
    }
    let path = SceneScriptBindingAsset::COMPONENT_TYPE_PATH;
    let address = ReflectObjectAddress::component(node_id, path).map_err(reflect_source_error)?;
    let field_id = ReflectFieldId::from_stable_keys(path, "bindings");
    match scene.reflect_read(ReflectReadRequest::new(address, field_id)) {
        Ok(response) => match response.field.value {
            ReflectedValue::Json(value) => serde_json::from_value(value)
                .map(Some)
                .map(normalize)
                .map_err(|error| reflect_error(error.to_string())),
            _ => Err(invalid(
                "script bindings reflection must return a typed array",
            )),
        },
        Err(ReflectError::MissingComponent { .. }) => Ok(None),
        Err(error) => Err(reflect_source_error(error)),
    }
}

fn normalize(
    bindings: Option<Vec<SceneScriptBindingAsset>>,
) -> Option<Vec<SceneScriptBindingAsset>> {
    bindings.filter(|bindings| !bindings.is_empty())
}
fn validate_bindings(bindings: Option<&[SceneScriptBindingAsset]>) -> Result<(), EditCommandError> {
    if bindings.is_some_and(|bindings| {
        bindings
            .iter()
            .any(|binding| binding.package.trim().is_empty() || binding.module.trim().is_empty())
    }) {
        return Err(invalid(
            "script bindings require nonempty package and module names",
        ));
    }
    Ok(())
}
fn invalid(invariant: &'static str) -> EditCommandError {
    EditCommandError::InvariantViolation { invariant }
}

#[cfg(test)]
#[path = "script_bindings/tests.rs"]
mod tests;
