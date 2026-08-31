use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use thiserror::Error;
use zircon_runtime::scene::WorldInspectionHierarchyRow;
use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::world_sync::{WorldInspectionFieldRow, WorldQueryResult};
use zircon_runtime_interface::GatewaySessionIdentity;

use crate::core::extension::{FieldEditorContainer, FieldEditorInstance, InspectorField};
use crate::ui::workbench::snapshot::{
    InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot, InspectorSnapshot,
};

pub(super) const PLAY_INSPECTOR_QUERY_INTERVAL: Duration = Duration::from_millis(100);

const NAME_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const HIERARCHY_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";
const LOCAL_TRANSFORM_COMPONENT_TYPE_PATH: &str =
    "zircon_runtime::scene::components::LocalTransform";

#[derive(Debug, Error)]
pub(super) enum PlayInspectorProjectionError {
    #[error("play Inspector query returned a non-Inspector projection")]
    UnexpectedProjection,
    #[error("play Inspector returned NotModified before a matching identity/entity snapshot")]
    MissingBaseSnapshot,
    #[error("play Inspector result entity {observed} does not match requested entity {expected}")]
    EntityMismatch { expected: u64, observed: u64 },
    #[error(
        "play Inspector NotModified generation {observed} does not match cached generation {expected}"
    )]
    GenerationMismatch { expected: u64, observed: u64 },
}

#[derive(Clone, Debug)]
struct PlayInspectorState {
    identity: GatewaySessionIdentity,
    entity: u64,
    generation: u64,
    snapshot: Option<InspectorSnapshot>,
}

#[derive(Default)]
pub(super) struct PlayInspectorProjection {
    state: Option<PlayInspectorState>,
    last_query_at: Option<Instant>,
    last_query_target: Option<(GatewaySessionIdentity, u64)>,
}

impl PlayInspectorProjection {
    /// Returns `None` when the focused Inspector cadence has not elapsed. The nested option is
    /// the generation hint for a query that must run now.
    pub(super) fn begin_query(
        &mut self,
        identity: &GatewaySessionIdentity,
        entity: u64,
        now: Instant,
    ) -> Option<Option<u64>> {
        let same_attempt_target =
            self.last_query_target
                .as_ref()
                .is_some_and(|(attempt_identity, attempt_entity)| {
                    attempt_identity == identity && *attempt_entity == entity
                });
        let due = !same_attempt_target
            || self.last_query_at.is_none_or(|last| {
                now.saturating_duration_since(last) >= PLAY_INSPECTOR_QUERY_INTERVAL
            });
        if !due {
            return None;
        }
        self.last_query_at = Some(now);
        self.last_query_target = Some((identity.clone(), entity));
        Some(
            self.state
                .as_ref()
                .filter(|state| &state.identity == identity && state.entity == entity)
                .map(|state| state.generation),
        )
    }

    pub(super) fn apply(
        &mut self,
        identity: GatewaySessionIdentity,
        requested_entity: u64,
        result: WorldQueryResult,
        hierarchy_row: Option<&WorldInspectionHierarchyRow>,
    ) -> Result<bool, PlayInspectorProjectionError> {
        match result {
            WorldQueryResult::InspectionFields {
                generation,
                entity,
                fields,
            } => {
                if entity != requested_entity {
                    return Err(PlayInspectorProjectionError::EntityMismatch {
                        expected: requested_entity,
                        observed: entity,
                    });
                }
                let snapshot = Some(project_inspector(entity, hierarchy_row, &fields));
                Ok(self.replace(identity, entity, generation, snapshot))
            }
            WorldQueryResult::EntityMissing { generation, entity } => {
                if entity != requested_entity {
                    return Err(PlayInspectorProjectionError::EntityMismatch {
                        expected: requested_entity,
                        observed: entity,
                    });
                }
                Ok(self.replace(identity, entity, generation, None))
            }
            WorldQueryResult::NotModified { generation } => {
                let Some(state) = self
                    .state
                    .as_ref()
                    .filter(|state| state.identity == identity && state.entity == requested_entity)
                else {
                    return Err(PlayInspectorProjectionError::MissingBaseSnapshot);
                };
                if state.generation != generation {
                    return Err(PlayInspectorProjectionError::GenerationMismatch {
                        expected: state.generation,
                        observed: generation,
                    });
                }
                Ok(false)
            }
            WorldQueryResult::ComponentRows { .. }
            | WorldQueryResult::HierarchyRows { .. }
            | WorldQueryResult::TransformSnapshot { .. } => {
                Err(PlayInspectorProjectionError::UnexpectedProjection)
            }
        }
    }

    pub(super) fn clear(&mut self) -> bool {
        self.last_query_at = None;
        self.last_query_target = None;
        self.state.take().is_some()
    }

    pub(super) fn snapshot_for(
        &self,
        identity: &GatewaySessionIdentity,
        entity: u64,
    ) -> Option<InspectorSnapshot> {
        self.state
            .as_ref()
            .filter(|state| &state.identity == identity && state.entity == entity)
            .and_then(|state| state.snapshot.clone())
    }

    fn replace(
        &mut self,
        identity: GatewaySessionIdentity,
        entity: u64,
        generation: u64,
        snapshot: Option<InspectorSnapshot>,
    ) -> bool {
        let changed = self.state.as_ref().is_none_or(|state| {
            state.identity != identity || state.entity != entity || state.snapshot != snapshot
        });
        self.state = Some(PlayInspectorState {
            identity,
            entity,
            generation,
            snapshot,
        });
        changed
    }
}

fn project_inspector(
    entity: u64,
    hierarchy_row: Option<&WorldInspectionHierarchyRow>,
    fields: &[WorldInspectionFieldRow],
) -> InspectorSnapshot {
    let name = string_field(fields, NAME_COMPONENT_TYPE_PATH, "value")
        .map(str::to_string)
        .or_else(|| hierarchy_row.map(|row| row.display_name.clone()))
        .unwrap_or_else(|| entity.to_string());
    let parent = entity_field(fields, HIERARCHY_COMPONENT_TYPE_PATH, "parent")
        .flatten()
        .or_else(|| hierarchy_row.and_then(|row| row.parent))
        .map(|parent| parent.to_string())
        .unwrap_or_default();
    let translation = vec3_field(fields, LOCAL_TRANSFORM_COMPONENT_TYPE_PATH, "translation")
        .map(format_vec3)
        .unwrap_or_else(empty_vec3);
    let scale = vec3_field(fields, LOCAL_TRANSFORM_COMPONENT_TYPE_PATH, "scale")
        .map(format_vec3)
        .unwrap_or_else(empty_vec3);

    InspectorSnapshot {
        id: entity,
        name,
        parent,
        translation,
        scale,
        plugin_components: project_plugin_components(fields),
    }
}

fn project_plugin_components(
    fields: &[WorldInspectionFieldRow],
) -> Vec<InspectorPluginComponentSnapshot> {
    let mut components = BTreeMap::<&str, Vec<&WorldInspectionFieldRow>>::new();
    for field in fields.iter().filter(|field| field.plugin_owned) {
        components
            .entry(field.component_type_path.as_str())
            .or_default()
            .push(field);
    }
    let field_editors = FieldEditorContainer::builtin();
    components
        .into_iter()
        .map(|(component_id, fields)| {
            let display_name = fields
                .first()
                .map(|field| field.component_display_name.clone())
                .unwrap_or_else(|| component_id.to_string());
            let properties = fields
                .into_iter()
                .map(|field| play_property(component_id, field, &field_editors))
                .collect();
            InspectorPluginComponentSnapshot {
                component_id: component_id.to_string(),
                display_name,
                plugin_id: plugin_id_from_component_id(component_id),
                customization_available: false,
                customization_ui_document: None,
                customization_controller: None,
                customization_template_id: None,
                customization_data_root: None,
                customization_bindings: Vec::new(),
                diagnostic: None,
                properties,
            }
        })
        .collect()
}

fn play_property(
    component_id: &str,
    field: &WorldInspectionFieldRow,
    field_editors: &FieldEditorContainer,
) -> InspectorPluginComponentPropertySnapshot {
    let field_id = format!("{component_id}.{}", field.field_name);
    let value = reflected_value_label(&field.value);
    let editable = field.writable && field.serializable;
    let field_editor = InspectorField::new(
        field_id.clone(),
        field.field_display_name.clone(),
        field.value_type_path.clone(),
        value.clone(),
        editable,
    )
    .map(|field| field_editors.resolve(field))
    .unwrap_or_else(|_| FieldEditorInstance::automatic());
    InspectorPluginComponentPropertySnapshot {
        field_id,
        name: field.field_name.clone(),
        label: field.field_display_name.clone(),
        value,
        value_kind: field.value_type_path.clone(),
        editable,
        field_editor,
    }
}

fn string_field<'a>(
    fields: &'a [WorldInspectionFieldRow],
    component: &str,
    name: &str,
) -> Option<&'a str> {
    match &field(fields, component, name)?.value {
        ReflectedValue::String(value) => Some(value),
        _ => None,
    }
}

fn entity_field(
    fields: &[WorldInspectionFieldRow],
    component: &str,
    name: &str,
) -> Option<Option<u64>> {
    match &field(fields, component, name)?.value {
        ReflectedValue::Entity(value) => Some(*value),
        _ => None,
    }
}

fn vec3_field(fields: &[WorldInspectionFieldRow], component: &str, name: &str) -> Option<[f32; 3]> {
    match &field(fields, component, name)?.value {
        ReflectedValue::Vec3(value) => Some(*value),
        _ => None,
    }
}

fn field<'a>(
    fields: &'a [WorldInspectionFieldRow],
    component: &str,
    name: &str,
) -> Option<&'a WorldInspectionFieldRow> {
    fields
        .iter()
        .find(|field| field.component_type_path == component && field.field_name == name)
}

fn format_vec3(value: [f32; 3]) -> [String; 3] {
    value.map(|axis| format!("{axis:.2}"))
}

fn empty_vec3() -> [String; 3] {
    [String::new(), String::new(), String::new()]
}

fn plugin_id_from_component_id(component_id: &str) -> String {
    component_id
        .split_once('.')
        .map(|(plugin_id, _)| plugin_id.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn reflected_value_label(value: &ReflectedValue) -> String {
    match value {
        ReflectedValue::Null => String::new(),
        ReflectedValue::Bool(value) => value.to_string(),
        ReflectedValue::Integer(value) => value.to_string(),
        ReflectedValue::Unsigned(value) => value.to_string(),
        ReflectedValue::Scalar(value) => value.to_string(),
        ReflectedValue::String(value)
        | ReflectedValue::Enum(value)
        | ReflectedValue::Resource(value) => value.clone(),
        ReflectedValue::Vec2(value) => format!("{}, {}", value[0], value[1]),
        ReflectedValue::Vec3(value) => format!("{}, {}, {}", value[0], value[1], value[2]),
        ReflectedValue::Vec4(value) | ReflectedValue::Quaternion(value) => {
            format!("{}, {}, {}, {}", value[0], value[1], value[2], value[3])
        }
        ReflectedValue::Entity(Some(value)) => value.to_string(),
        ReflectedValue::Entity(None) => String::new(),
        ReflectedValue::List(values) => format!("{} items", values.len()),
        ReflectedValue::Map(values) => format!("{} fields", values.len()),
        ReflectedValue::Json(value) => match value {
            serde_json::Value::String(value) => value.clone(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use zircon_runtime_interface::reflect::ReflectedValue;
    use zircon_runtime_interface::world_sync::{WorldInspectionFieldRow, WorldQueryResult};
    use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

    use super::{PlayInspectorProjection, PLAY_INSPECTOR_QUERY_INTERVAL};

    fn identity(gateway_generation: u64) -> GatewaySessionIdentity {
        GatewaySessionIdentity::new(3, ZrRuntimeSessionHandle::new(5), 7, Some(11))
            .with_gateway_generation(gateway_generation)
    }

    fn field(
        component_type_path: &str,
        component_display_name: &str,
        field_name: &str,
        value: ReflectedValue,
        plugin_owned: bool,
    ) -> WorldInspectionFieldRow {
        WorldInspectionFieldRow {
            component_type_path: component_type_path.to_string(),
            component_display_name: component_display_name.to_string(),
            field_name: field_name.to_string(),
            field_display_name: field_name.to_string(),
            value_type_path: value.type_name().to_string(),
            value,
            writable: true,
            serializable: true,
            plugin_owned,
        }
    }

    #[test]
    fn focused_query_cadence_is_immediate_then_generation_qualified() {
        let mut projection = PlayInspectorProjection::default();
        let identity = identity(1);
        let started = Instant::now();

        assert_eq!(projection.begin_query(&identity, 7, started), Some(None));
        projection
            .apply(
                identity.clone(),
                7,
                WorldQueryResult::InspectionFields {
                    generation: 4,
                    entity: 7,
                    fields: Vec::new(),
                },
                None,
            )
            .expect("first focused projection should be valid");
        assert_eq!(
            projection.begin_query(
                &identity,
                7,
                started + PLAY_INSPECTOR_QUERY_INTERVAL - Duration::from_millis(1)
            ),
            None
        );
        assert_eq!(
            projection.begin_query(&identity, 7, started + PLAY_INSPECTOR_QUERY_INTERVAL),
            Some(Some(4))
        );
    }

    #[test]
    fn play_snapshot_projects_runtime_values_and_writable_plugin_fields() {
        let mut projection = PlayInspectorProjection::default();
        let identity = identity(1);
        let fields = vec![
            field(
                "zircon_runtime::scene::components::Name",
                "Name",
                "value",
                ReflectedValue::String("Runtime Hero".to_string()),
                false,
            ),
            field(
                "zircon_runtime::scene::components::LocalTransform",
                "Transform",
                "translation",
                ReflectedValue::Vec3([1.0, 2.0, 3.0]),
                false,
            ),
            field(
                "zircon_runtime::scene::components::LocalTransform",
                "Transform",
                "scale",
                ReflectedValue::Vec3([4.0, 5.0, 6.0]),
                false,
            ),
            field(
                "weather.cloud_layer",
                "Cloud Layer",
                "coverage",
                ReflectedValue::Scalar(0.75),
                true,
            ),
        ];

        assert!(projection
            .apply(
                identity.clone(),
                7,
                WorldQueryResult::InspectionFields {
                    generation: 4,
                    entity: 7,
                    fields: fields.clone(),
                },
                None,
            )
            .expect("runtime Inspector fields should project"));
        let snapshot = projection
            .snapshot_for(&identity, 7)
            .expect("matching identity/entity should expose the runtime Inspector");
        assert_eq!(snapshot.name, "Runtime Hero");
        assert_eq!(snapshot.translation, ["1.00", "2.00", "3.00"]);
        assert_eq!(snapshot.scale, ["4.00", "5.00", "6.00"]);
        assert_eq!(snapshot.plugin_components.len(), 1);
        assert!(snapshot.plugin_components[0].properties[0].editable);
        assert!(!projection
            .apply(
                identity,
                7,
                WorldQueryResult::InspectionFields {
                    generation: 5,
                    entity: 7,
                    fields,
                },
                None,
            )
            .expect("an unchanged visible Inspector may still advance its generation"));
    }

    #[test]
    fn stale_identity_snapshot_is_never_exposed_to_a_replacement_runtime() {
        let mut projection = PlayInspectorProjection::default();
        let original = identity(1);
        projection
            .apply(
                original.clone(),
                7,
                WorldQueryResult::InspectionFields {
                    generation: 4,
                    entity: 7,
                    fields: Vec::new(),
                },
                None,
            )
            .expect("base Inspector should project");

        assert!(projection.snapshot_for(&identity(2), 7).is_none());
        assert!(projection.snapshot_for(&original, 8).is_none());
    }
}
