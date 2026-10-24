//! Captures Inspector scope before discovery or dispatch. The UI owns only one
//! metadata page and one selected schema; factories capture no shell/context.

use crate::core::editing::command::{
    AddComponentCatalog, AddComponentCatalogEntry, AddComponentRequest, AddComponentScope,
    GatewayStamp,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditCommandError, EditContext};
use crate::core::editing::operation::OperationCommandFactoryError;
use crate::core::editor_operation::EditorOperationInvocation;
use crate::core::play::{PlayModeKind, WorldDomain};
use zircon_runtime::scene::world::DynamicComponentCreationTemplate;

use super::EditorHostEventController;

pub(crate) const COMPONENT_ADD_PAGE_SIZE: usize = 32;

impl EditorHostEventController {
    fn with_inspector_component_add_context<R>(
        &self,
        expected: Option<&AddComponentScope>,
        read: impl FnOnce(&CoreEditContext, AddComponentScope) -> Result<R, EditCommandError>,
    ) -> Result<R, String> {
        if self.play_sessions().mode() != PlayModeKind::Edit {
            return Err("component creation requires the Edit world".into());
        }
        let shell = self.shell().lock();
        let state = &shell.state;
        state
            .ensure_inspector_binding_can_begin()
            .map_err(|error| error.to_string())?;
        if state.is_playing()
            || state.viewport_controller.selection().active_domain() != WorldDomain::Edit
        {
            return Err("component creation requires the Edit selection".into());
        }
        let history = state
            .authoring_scene_history_context()
            .map_err(|error| error.to_string())?;
        state
            .ensure_transaction_context_selection_is_current()
            .map_err(|error| error.to_string())?;
        let selected = state.viewport_controller.selection();
        let targets = selected.active_items().iter().copied().collect::<Vec<_>>();
        let primary = selected.active_primary();
        let ui_selection_generation = selected.generation(WorldDomain::Edit);
        if targets.is_empty() {
            return Err("component creation requires a selection".into());
        }
        state
            .context
            .transactions()
            .with_context::<CoreEditContext, _>(|core| {
                let route = core.capture_world_route(WorldDomain::Edit)?;
                let identity =
                    route
                        .gateway_identity()
                        .ok_or(EditCommandError::WorldRouteUnavailable {
                            world_domain: WorldDomain::Edit,
                        })?;
                let scope = AddComponentScope {
                    history,
                    gateway: GatewayStamp::capture(identity),
                    ui_selection_generation,
                    core_selection_generation: core.selection_snapshot().generation(),
                    world_generation: core.with_scene(|scene| scene.world_generation())?,
                    targets,
                    primary,
                };
                if expected.is_some_and(|expected| expected != &scope) {
                    return Err(EditCommandError::InvariantViolation {
                        invariant: "component creation scope changed",
                    });
                }
                scope.validate(core)?;
                read(core, scope)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "component creation transaction context is unavailable".to_string())?
            .map_err(|error| error.to_string())
    }

    pub(crate) fn inspector_component_add_catalog(
        &self,
        page: usize,
        expected: Option<&AddComponentScope>,
    ) -> Result<AddComponentCatalog, String> {
        self.with_inspector_component_add_context(expected, |core, scope| {
            let (total, page, entries) = core.with_scene(|scene| {
                let total = scene.dynamic_component_creation_descriptors().count();
                let page = page.min(total.saturating_sub(1) / COMPONENT_ADD_PAGE_SIZE);
                let entries = scene
                    .dynamic_component_creation_descriptors()
                    .skip(page * COMPONENT_ADD_PAGE_SIZE)
                    .take(COMPONENT_ADD_PAGE_SIZE)
                    .map(|descriptor| AddComponentCatalogEntry {
                        type_path: descriptor.type_id.clone(),
                        display_name: descriptor.display_name.clone(),
                        already_present: scope.targets.iter().any(|&target| {
                            scene
                                .dynamic_component(target, &descriptor.type_id)
                                .is_some()
                        }),
                    })
                    .collect();
                (total, page, entries)
            })?;
            Ok(AddComponentCatalog {
                scope,
                total,
                page,
                entries,
            })
        })
    }

    pub(crate) fn inspector_component_add_template(
        &self,
        scope: &AddComponentScope,
        type_path: &str,
    ) -> Result<DynamicComponentCreationTemplate, String> {
        self.with_inspector_component_add_context(Some(scope), |core, scope| {
            core.with_scene(|scene| {
                if scope
                    .targets
                    .iter()
                    .any(|&target| scene.dynamic_component(target, type_path).is_some())
                {
                    return Err(EditCommandError::InvariantViolation {
                        invariant: "component already present",
                    });
                }
                scene
                    .dynamic_component_creation_template(type_path)
                    .map_err(|source| EditCommandError::ReflectError {
                        source: Box::new(source),
                    })
            })?
        })
    }

    pub(crate) fn preflight_inspector_script_binding_set(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<(), OperationCommandFactoryError> {
        let request =
            crate::core::editing::command::SetScriptBindingsRequest::from_invocation(invocation)?;
        self.with_inspector_component_add_context(Some(&request.scope), |core, _| {
            request.validate(core)
        })
        .map_err(|reason| OperationCommandFactoryError::InvalidArguments {
            operation: invocation.operation_id.clone(),
            reason,
        })
    }

    pub(crate) fn preflight_inspector_component_add(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<(), OperationCommandFactoryError> {
        let request = AddComponentRequest::from_invocation(invocation)?;
        self.with_inspector_component_add_context(Some(&request.scope), |core, _| {
            request.validate(core)
        })
        .map_err(|reason| OperationCommandFactoryError::InvalidArguments {
            operation: invocation.operation_id.clone(),
            reason,
        })
    }
}
