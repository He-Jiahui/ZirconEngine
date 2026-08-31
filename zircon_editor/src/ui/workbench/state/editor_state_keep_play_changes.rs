use thiserror::Error;
use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::world_sync::{WorldInspectionFieldRow, WorldQuery, WorldQueryResult};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::engine::{HistoryContextId, MergeMode};
use crate::core::gateway::GatewayError;
use crate::core::play::WorldDomain;

use super::no_project_open::no_project_open;
use super::{editor_state::EditorState, EditorStateOperationError};

const HIERARCHY_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";

#[derive(Debug, Error, PartialEq)]
pub enum KeepPlayChangesError {
    #[error("Keep Play Changes requires exactly one selected play entity, found {count}")]
    RequiresSinglePlaySelection { count: usize },
    #[error(
        "Keep Play Changes expected play instance {expected}, but the gateway exposes {actual:?}"
    )]
    PlayGatewayIdentityMismatch { expected: u64, actual: Option<u64> },
    #[error("play entity {entity} disappeared before Keep Play Changes could capture it")]
    PlayEntityMissing { entity: NodeId },
    #[error(
        "Keep Play Changes queried play entity {expected}, but the runtime returned entity {actual}"
    )]
    ResponseEntityMismatch { expected: NodeId, actual: NodeId },
    #[error("Keep Play Changes received an invalid runtime query result: {kind}")]
    UnexpectedQueryResult { kind: &'static str },
    #[error("play entity {entity} has no authoring-world counterpart")]
    AuthoringCounterpartMissing { entity: NodeId },
    #[error(transparent)]
    Gateway(#[from] GatewayError),
}

impl EditorState {
    pub fn keep_play_changes(&mut self) -> Result<bool, EditorStateOperationError> {
        self.prepare_non_gizmo_scene_action()?;
        let selection = self.viewport_controller.selection();
        let WorldDomain::Play(instance) = selection.active_domain() else {
            return Err(KeepPlayChangesError::RequiresSinglePlaySelection {
                count: selection.active_items().len(),
            }
            .into());
        };
        let Some(selected) = selection.active_primary() else {
            return Err(KeepPlayChangesError::RequiresSinglePlaySelection {
                count: selection.active_items().len(),
            }
            .into());
        };
        if selection.active_items().len() != 1 || !selection.active_items().contains(&selected) {
            return Err(KeepPlayChangesError::RequiresSinglePlaySelection {
                count: selection.active_items().len(),
            }
            .into());
        }

        let gateway = self.context.play_gateway_handle();
        let identity = gateway.identity();
        if identity.play_instance() != Some(instance.raw()) {
            return Err(KeepPlayChangesError::PlayGatewayIdentityMismatch {
                expected: instance.raw(),
                actual: identity.play_instance(),
            }
            .into());
        }
        let query = gateway
            .query_world_at_identity(&identity, WorldQuery::inspection_fields(selected, None))
            .map_err(KeepPlayChangesError::from)?;
        let fields = match query {
            WorldQueryResult::InspectionFields { entity, fields, .. } if entity == selected => {
                fields
            }
            WorldQueryResult::InspectionFields { entity, .. } => {
                return Err(KeepPlayChangesError::ResponseEntityMismatch {
                    expected: selected,
                    actual: entity,
                }
                .into())
            }
            WorldQueryResult::EntityMissing { entity, .. } if entity == selected => {
                return Err(KeepPlayChangesError::PlayEntityMissing { entity }.into())
            }
            WorldQueryResult::EntityMissing { entity, .. } => {
                return Err(KeepPlayChangesError::ResponseEntityMismatch {
                    expected: selected,
                    actual: entity,
                }
                .into())
            }
            WorldQueryResult::NotModified { .. } => {
                return Err(KeepPlayChangesError::UnexpectedQueryResult {
                    kind: "not_modified_without_a_generation_hint",
                }
                .into())
            }
            WorldQueryResult::ComponentRows { .. } | WorldQueryResult::HierarchyRows { .. } => {
                return Err(KeepPlayChangesError::UnexpectedQueryResult {
                    kind: "non_inspector_projection",
                }
                .into())
            }
        };
        let fields = fields
            .into_iter()
            .filter(keepable_field)
            .collect::<Vec<_>>();
        let commands = self
            .world
            .with_world(
                |scene| -> Result<Vec<EditorCommand>, EditorStateOperationError> {
                    if scene.find_node(selected).is_none() {
                        return Err(KeepPlayChangesError::AuthoringCounterpartMissing {
                            entity: selected,
                        }
                        .into());
                    }
                    let mut commands = Vec::with_capacity(fields.len());
                    for field in fields {
                        if let Some(command) = EditorCommand::set_reflected_scene_field(
                            scene,
                            selected,
                            field.component_type_path,
                            field.field_name,
                            field.value,
                        )? {
                            commands.push(command);
                        }
                    }
                    Ok(commands)
                },
            )?
            .ok_or_else(no_project_open)??;
        if commands.is_empty() {
            self.set_status_line("Play properties already match the authoring entity");
            return Ok(false);
        }

        let document = self
            .active_scene_document
            .ok_or(EditorStateOperationError::SceneDocumentNotActive)?;
        self.bind_transaction_context_for(WorldDomain::Edit)?;
        let editor_context = std::sync::Arc::clone(&self.context);
        let mut scope = editor_context
            .transactions()
            .begin("Keep Play Changes", HistoryContextId::Document(document))?;
        scope.set_merge_mode(MergeMode::Disable);
        let changed_fields = commands.len();
        for command in commands {
            scope.push(command)?;
        }
        scope.commit()?;
        self.set_status_line(format!(
            "Kept {changed_fields} play property change(s) on entity {selected}"
        ));
        Ok(true)
    }
}

fn keepable_field(field: &WorldInspectionFieldRow) -> bool {
    field.writable
        && field.serializable
        && !(field.component_type_path == HIERARCHY_COMPONENT_TYPE_PATH
            && field.field_name == "parent")
}
