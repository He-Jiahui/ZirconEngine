use std::any::Any;
use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::Arc;

use super::animation_document::AnimationAuthoringDocumentStore;
use super::engine::{EditCommandError, EditContext, EditWorldRoute, SelectionSnapshot};
use super::selection::SceneSelection;
use crate::core::gateway::{EditorRuntimeGatewayHandle, EditorRuntimeOperationRoute, GatewayError};
#[cfg(test)]
use crate::core::gateway::{InProcessGateway, SharedEditorRuntimeGateway};
use crate::core::play::WorldDomain;
#[cfg(test)]
use zircon_runtime::scene::LevelSystem;
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::world_sync::{WorldQuery, WorldQueryResult};
use zircon_runtime_interface::{
    ZrRuntimeEditorTransformWriteV1, ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RuntimeTransformSnapshot {
    pub(crate) entity: u64,
    pub(crate) world_replacement_epoch: u64,
    pub(crate) transform: Transform,
}

/// Headless-safe owner for the core edit state used by the transaction engine.
pub(crate) struct CoreEditContext {
    animation_documents: AnimationAuthoringDocumentStore,
    selections: BTreeMap<WorldDomain, SelectionSnapshot>,
    selection_generation: u64,
    authoring_gateway: EditorRuntimeGatewayHandle,
    play_gateway: EditorRuntimeGatewayHandle,
    active_route: Option<EditWorldRoute>,
}

/// Result of one serialized scene-write callback.
///
/// A gateway may report a transport or protocol failure only after it has invoked the callback.
/// The transaction command must therefore distinguish an unexecuted callback from a completed
/// scene mutation that needs compensation.
pub(crate) struct SceneMutationOutcome<R> {
    result: R,
    post_callback_error: Option<EditCommandError>,
}

impl<R> SceneMutationOutcome<R> {
    pub(crate) fn into_parts(self) -> (R, Option<EditCommandError>) {
        (self.result, self.post_callback_error)
    }
}

impl CoreEditContext {
    pub(crate) fn new(authoring_gateway: EditorRuntimeGatewayHandle) -> Self {
        Self::with_world_gateways(authoring_gateway, EditorRuntimeGatewayHandle::detached())
    }

    pub(crate) fn with_world_gateways(
        authoring_gateway: EditorRuntimeGatewayHandle,
        play_gateway: EditorRuntimeGatewayHandle,
    ) -> Self {
        let mut selections = BTreeMap::new();
        selections.insert(WorldDomain::Edit, SelectionSnapshot::default());
        Self {
            animation_documents: AnimationAuthoringDocumentStore::default(),
            selections,
            selection_generation: 0,
            authoring_gateway,
            play_gateway,
            active_route: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn bind_scene(
        &mut self,
        scene: LevelSystem,
        selection: SceneSelection,
    ) -> Result<(), EditCommandError> {
        let gateway: SharedEditorRuntimeGateway =
            Arc::new(InProcessGateway::for_authoring_level(scene));
        self.authoring_gateway
            .replace(gateway)
            .map_err(gateway_failure)?;
        self.active_route = None;
        self.bind_authoring_selection(selection)
    }

    pub(crate) fn bind_authoring_selection(
        &mut self,
        selection: SceneSelection,
    ) -> Result<(), EditCommandError> {
        self.bind_selection(WorldDomain::Edit, selection)
    }

    pub(crate) fn bind_selection(
        &mut self,
        world_domain: WorldDomain,
        selection: SceneSelection,
    ) -> Result<(), EditCommandError> {
        let snapshot = SelectionSnapshot::scene(self.next_selection_generation()?, selection);
        self.selections.insert(world_domain, snapshot);
        Ok(())
    }

    pub(crate) fn clear_scene(&mut self) -> Result<(), EditCommandError> {
        let generation = self.next_selection_generation()?;
        self.selections
            .insert(WorldDomain::Edit, SelectionSnapshot::empty(generation));
        Ok(())
    }

    pub(crate) fn with_scene<R>(
        &self,
        read: impl FnOnce(&Scene) -> R,
    ) -> Result<R, EditCommandError> {
        let mut read = Some(read);
        let mut result = None;
        let mut duplicate_callback = false;
        let (gateway, identity) = self.active_gateway_route()?;
        gateway
            .with_world_at_identity(&identity, &mut |scene| {
                let Some(read) = read.take() else {
                    duplicate_callback = true;
                    return;
                };
                result = Some(read(scene));
            })
            .map_err(gateway_failure)?;
        if duplicate_callback {
            return Err(gateway_callback_protocol_failure());
        }
        result.ok_or_else(missing_scene)
    }

    pub(crate) fn with_scene_mut<R>(
        &self,
        write: impl FnOnce(&mut Scene) -> R,
    ) -> Result<SceneMutationOutcome<R>, EditCommandError> {
        let mut write = Some(write);
        let mut result = None;
        let mut duplicate_callback = false;
        let (gateway, identity) = self.active_gateway_route()?;
        let dispatch = gateway.with_world_mut_at_identity(&identity, &mut |scene| {
            let Some(write) = write.take() else {
                duplicate_callback = true;
                return;
            };
            result = Some(write(scene));
        });
        let Some(result) = result else {
            return Err(match dispatch {
                Ok(()) => missing_scene(),
                Err(error) => gateway_failure(error),
            });
        };
        let post_callback_error = if duplicate_callback {
            Some(gateway_callback_protocol_failure())
        } else {
            dispatch.err().map(gateway_failure)
        };
        Ok(SceneMutationOutcome {
            result,
            post_callback_error,
        })
    }

    pub(crate) fn capture_runtime_transform(
        &self,
        entity: u64,
    ) -> Result<RuntimeTransformSnapshot, EditCommandError> {
        let (gateway, identity) = self.active_gateway_route()?;
        match gateway
            .query_world_at_identity(&identity, WorldQuery::transform_snapshot(entity))
            .map_err(gateway_failure)?
        {
            WorldQueryResult::TransformSnapshot {
                world_replacement_epoch,
                entity: observed_entity,
                transform,
                ..
            } if observed_entity == entity && world_replacement_epoch != 0 => {
                Ok(RuntimeTransformSnapshot {
                    entity,
                    world_replacement_epoch,
                    transform,
                })
            }
            WorldQueryResult::TransformSnapshot {
                entity: observed_entity,
                ..
            } => Err(EditCommandError::InvariantViolation {
                invariant: if observed_entity == entity {
                    "runtime transform query returned an invalid replacement epoch"
                } else {
                    "runtime transform query returned another entity"
                },
            }),
            WorldQueryResult::EntityMissing { .. } => Err(EditCommandError::TargetMissing {
                target: format!("runtime scene node {entity}"),
            }),
            _ => Err(EditCommandError::InvariantViolation {
                invariant: "runtime transform query returned another projection kind",
            }),
        }
    }

    pub(crate) fn dispatch_runtime_transform(
        &self,
        request: &ZrRuntimeEditorTransformWriteV1,
    ) -> Result<(), EditCommandError> {
        if !matches!(self.active_world_domain(), WorldDomain::Play(_)) {
            return Err(EditCommandError::InvariantViolation {
                invariant: "runtime transform dispatch requires an active Play world route",
            });
        }
        let (gateway, identity) = self.active_gateway_route()?;
        gateway
            .handle_event_at_identity(
                &identity,
                ZrRuntimeEventV1::editor_transform_write(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                    request,
                ),
            )
            .map_err(gateway_failure)
    }

    pub(crate) fn scene_selection(&self) -> Result<SceneSelection, EditCommandError> {
        self.current_selection().scene_selection()
    }

    pub(crate) fn animation_documents(&self) -> &AnimationAuthoringDocumentStore {
        &self.animation_documents
    }

    pub(crate) fn animation_documents_mut(&mut self) -> &mut AnimationAuthoringDocumentStore {
        &mut self.animation_documents
    }

    pub(crate) fn selection_snapshot(&self) -> SelectionSnapshot {
        self.current_selection().clone()
    }

    pub(crate) fn restore_selection_snapshot(
        &mut self,
        snapshot: &SelectionSnapshot,
    ) -> Result<(), EditCommandError> {
        self.selections
            .insert(self.active_world_domain(), snapshot.clone());
        self.selection_generation = self.selection_generation.max(snapshot.generation());
        Ok(())
    }

    pub(crate) fn set_scene_selection(
        &mut self,
        selection: SceneSelection,
    ) -> Result<(), EditCommandError> {
        let snapshot = SelectionSnapshot::scene(self.next_selection_generation()?, selection);
        self.selections.insert(self.active_world_domain(), snapshot);
        Ok(())
    }

    fn gateway_for(&self, world_domain: WorldDomain) -> &EditorRuntimeGatewayHandle {
        match world_domain {
            WorldDomain::Edit => &self.authoring_gateway,
            WorldDomain::Play(_) => &self.play_gateway,
        }
    }

    fn active_gateway_route(
        &self,
    ) -> Result<
        (
            &EditorRuntimeGatewayHandle,
            zircon_runtime_interface::GatewaySessionIdentity,
        ),
        EditCommandError,
    > {
        let Some(route) = self.active_route.as_ref() else {
            return Ok((&self.authoring_gateway, self.authoring_gateway.identity()));
        };
        let identity =
            route
                .gateway_identity()
                .cloned()
                .ok_or(EditCommandError::WorldRouteUnavailable {
                    world_domain: route.world_domain(),
                })?;
        Ok((self.gateway_for(route.world_domain()), identity))
    }

    fn active_world_domain(&self) -> WorldDomain {
        self.active_route
            .as_ref()
            .map_or(WorldDomain::Edit, EditWorldRoute::world_domain)
    }

    fn current_selection(&self) -> &SelectionSnapshot {
        self.selections
            .get(&self.active_world_domain())
            .or_else(|| self.selections.get(&WorldDomain::Edit))
            .expect("the edit context always owns an authoring selection slot")
    }

    fn next_selection_generation(&mut self) -> Result<u64, EditCommandError> {
        self.selection_generation = self
            .selection_generation
            .checked_add(1)
            .ok_or(EditCommandError::SelectionGenerationExhausted)?;
        Ok(self.selection_generation)
    }
}

fn missing_scene() -> EditCommandError {
    EditCommandError::TargetMissing {
        target: "active editor scene".to_string(),
    }
}

fn gateway_failure(error: GatewayError) -> EditCommandError {
    EditCommandError::ExternalEffect {
        source: Box::new(error),
    }
}

fn gateway_callback_protocol_failure() -> EditCommandError {
    gateway_failure(GatewayError::Protocol {
        message: "borrowed world callback was invoked more than once".to_owned(),
    })
}

impl Default for CoreEditContext {
    fn default() -> Self {
        Self::new(EditorRuntimeGatewayHandle::detached())
    }
}

impl EditContext for CoreEditContext {
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError> {
        let identity = self.gateway_for(world_domain).identity();
        let matches_domain = match world_domain {
            WorldDomain::Edit => identity.play_instance().is_none(),
            WorldDomain::Play(instance) => identity.play_instance() == Some(instance.raw()),
        };
        if !matches_domain {
            return Err(EditCommandError::WorldRouteUnavailable { world_domain });
        }
        Ok(EditWorldRoute::runtime(world_domain, identity))
    }

    fn activate_world_route(&mut self, route: &EditWorldRoute) -> Result<(), EditCommandError> {
        if self.capture_world_route(route.world_domain())? != *route {
            return Err(EditCommandError::WorldRouteStale {
                world_domain: route.world_domain(),
            });
        }
        let world_domain = route.world_domain();
        if !self.selections.contains_key(&world_domain) {
            let generation = self.next_selection_generation()?;
            self.selections
                .insert(world_domain, SelectionSnapshot::empty(generation));
        }
        self.active_route = Some(route.clone());
        Ok(())
    }

    fn retire_world_route(&mut self, world_domain: WorldDomain) -> Result<(), EditCommandError> {
        if world_domain == WorldDomain::Edit {
            return Err(EditCommandError::InvariantViolation {
                invariant: "the authoring world route cannot be retired",
            });
        }
        if self.active_world_domain() == world_domain {
            return Err(EditCommandError::InvariantViolation {
                invariant: "the active edit world route cannot be retired",
            });
        }
        self.selections.remove(&world_domain);
        Ok(())
    }

    fn runtime_operations(&self) -> Result<EditorRuntimeOperationRoute, EditCommandError> {
        let route = self
            .active_route
            .as_ref()
            .ok_or(EditCommandError::InvariantViolation {
                invariant: "runtime operations require an active edit world route",
            })?;
        let identity = route
            .gateway_identity()
            .ok_or(EditCommandError::WorldRouteUnavailable {
                world_domain: route.world_domain(),
            })?;
        EditorRuntimeOperationRoute::capture_at_identity(
            self.gateway_for(route.world_domain()),
            identity,
        )
        .map_err(gateway_failure)
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        self.current_selection().clone()
    }

    fn restore_selection(&mut self, snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        self.restore_selection_snapshot(snapshot)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::play::PlayInstanceId;

    #[test]
    fn retiring_repeated_play_routes_keeps_selection_storage_bounded() {
        let mut context = CoreEditContext::default();

        for raw in 1..=256 {
            let world_domain = WorldDomain::Play(PlayInstanceId::for_test(raw));
            context
                .selections
                .insert(world_domain, SelectionSnapshot::fixture_value(raw, raw));

            context
                .retire_world_route(world_domain)
                .expect("an inactive play route should retire");
            assert_eq!(context.selections.len(), 1);
            assert!(context.selections.contains_key(&WorldDomain::Edit));
        }
    }

    #[test]
    fn retiring_the_active_world_route_is_rejected() {
        let mut context = CoreEditContext::default();
        let world_domain = WorldDomain::Play(PlayInstanceId::for_test(1));
        context
            .selections
            .insert(world_domain, SelectionSnapshot::fixture_value(1, 1));
        context.active_route = Some(EditWorldRoute::logical(world_domain));

        assert!(matches!(
            context.retire_world_route(world_domain),
            Err(EditCommandError::InvariantViolation { .. })
        ));
        assert!(context.selections.contains_key(&world_domain));
    }
}
