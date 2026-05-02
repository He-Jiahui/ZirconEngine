use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEvent, UiComponentEventEnvelope,
    UiComponentProjectionPatch, UiValue, UiValueKind,
};

const ASSET_EDITOR_COMPONENT_DOMAIN: &str = "asset_editor";
const SUBJECT_SOURCE_NAME: &str = "subject";
const VALUE_PROPERTY: &str = "value";

pub(crate) fn apply_asset_editor_component_envelope(
    manager: &EditorManager,
    envelope: &UiComponentEventEnvelope,
) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
    if envelope.target.domain != ASSET_EDITOR_COMPONENT_DOMAIN {
        return Err(UiComponentAdapterError::UnsupportedTargetDomain {
            domain: envelope.target.domain.clone(),
        });
    }

    let instance_id = ViewInstanceId::new(
        envelope
            .target
            .subject
            .as_deref()
            .ok_or_else(|| UiComponentAdapterError::MissingSource {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                source_name: SUBJECT_SOURCE_NAME.to_string(),
            })?
            .to_string(),
    );

    let (property, value) = match &envelope.event {
        UiComponentEvent::ValueChanged { property, value }
        | UiComponentEvent::Commit { property, value } => (property, value),
        event => {
            return Err(UiComponentAdapterError::UnsupportedEvent {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                event_kind: event.kind(),
            });
        }
    };

    if property != VALUE_PROPERTY {
        return Err(UiComponentAdapterError::RejectedInput {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: format!("expected property '{VALUE_PROPERTY}', got '{property}'"),
        });
    }

    let literal =
        asset_editor_literal_value(value, &envelope.target.domain, &envelope.target.path)?;
    let changed =
        match envelope.target.path.as_str() {
            "widget.control_id" => manager
                .set_ui_asset_editor_selected_widget_control_id(&instance_id, literal.clone()),
            "widget.text" => manager
                .set_ui_asset_editor_selected_widget_text_property(&instance_id, literal.clone()),
            "component.root_class_policy" => manager
                .set_ui_asset_editor_selected_component_root_class_policy(
                    &instance_id,
                    literal.clone(),
                ),
            "slot.mount" => {
                manager.set_ui_asset_editor_selected_slot_mount(&instance_id, literal.clone())
            }
            "slot.padding" => {
                manager.set_ui_asset_editor_selected_slot_padding(&instance_id, literal.clone())
            }
            "slot.width_preferred" => manager
                .set_ui_asset_editor_selected_slot_width_preferred(&instance_id, literal.clone()),
            "slot.height_preferred" => manager
                .set_ui_asset_editor_selected_slot_height_preferred(&instance_id, literal.clone()),
            "layout.width_preferred" => manager
                .set_ui_asset_editor_selected_layout_width_preferred(&instance_id, literal.clone()),
            "layout.height_preferred" => manager
                .set_ui_asset_editor_selected_layout_height_preferred(
                    &instance_id,
                    literal.clone(),
                ),
            "slot.semantic.value" => manager
                .set_ui_asset_editor_selected_slot_semantic_value(&instance_id, literal.clone()),
            "layout.semantic.value" => manager
                .set_ui_asset_editor_selected_layout_semantic_value(&instance_id, literal.clone()),
            "binding.id" => {
                manager.set_ui_asset_editor_selected_binding_id(&instance_id, literal.clone())
            }
            "binding.event" => {
                manager.set_ui_asset_editor_selected_binding_event(&instance_id, literal.clone())
            }
            "binding.route" => {
                manager.set_ui_asset_editor_selected_binding_route(&instance_id, literal.clone())
            }
            "binding.route_target" => manager
                .set_ui_asset_editor_selected_binding_route_target(&instance_id, literal.clone()),
            "binding.action_target" => manager
                .set_ui_asset_editor_selected_binding_action_target(&instance_id, literal.clone()),
            path if path.starts_with("slot.semantic.field.") => manager
                .set_ui_asset_editor_selected_slot_semantic_field(
                    &instance_id,
                    path.trim_start_matches("slot.semantic.field."),
                    literal.clone(),
                ),
            path if path.starts_with("layout.semantic.field.") => manager
                .set_ui_asset_editor_selected_layout_semantic_field(
                    &instance_id,
                    path.trim_start_matches("layout.semantic.field."),
                    literal.clone(),
                ),
            _ => {
                return Err(UiComponentAdapterError::UnsupportedTargetPath {
                    domain: envelope.target.domain.clone(),
                    path: envelope.target.path.clone(),
                });
            }
        }
        .map_err(|error| UiComponentAdapterError::HostMutation {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: error.to_string(),
        })?;

    let patch = UiComponentProjectionPatch::new(envelope.control_id.clone())
        .with_attribute(VALUE_PROPERTY, UiValue::String(literal.clone()))
        .with_state_value(
            envelope.target.path.clone(),
            UiValue::String(literal.clone()),
        );

    let result = if changed {
        UiComponentAdapterResult::changed()
            .with_transaction(format!(
                "asset_editor:{}:{}",
                instance_id.0, envelope.target.path
            ))
            .with_mutation_source("asset_editor")
            .with_status(format!("asset editor {} updated", envelope.target.path))
            .with_patch(patch)
    } else {
        UiComponentAdapterResult::unchanged()
            .dirty(false)
            .with_transaction(format!(
                "asset_editor:{}:{}",
                instance_id.0, envelope.target.path
            ))
            .with_mutation_source("asset_editor")
            .with_status(format!("asset editor {} unchanged", envelope.target.path))
            .with_patch(patch)
    };

    Ok(result)
}

fn asset_editor_literal_value(
    value: &UiValue,
    domain: &str,
    path: &str,
) -> Result<String, UiComponentAdapterError> {
    match value {
        UiValue::String(value)
        | UiValue::Color(value)
        | UiValue::AssetRef(value)
        | UiValue::InstanceRef(value)
        | UiValue::Enum(value) => Ok(value.clone()),
        UiValue::Bool(value) => Ok(value.to_string()),
        UiValue::Int(value) => Ok(value.to_string()),
        UiValue::Float(value) => Ok(value.to_string()),
        UiValue::Flags(values) => Ok(values.join(",")),
        _ => Err(UiComponentAdapterError::InvalidValueKind {
            domain: domain.to_string(),
            path: path.to_string(),
            expected: UiValueKind::String,
            actual: value.kind(),
        }),
    }
}
