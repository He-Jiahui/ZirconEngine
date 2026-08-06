use crate::ui::layouts::windows::workbench_host_window::{
    InspectorPaneViewData, InspectorPluginComponentViewData, PaneContentSize,
};
use crate::ui::retained_host as host_contract;

const INSPECTOR_FIELD_ROW_HEIGHT: f32 = 28.0;
const INSPECTOR_FIELD_ROW_GAP: f32 = 6.0;
const INSPECTOR_FIELD_PADDING: f32 = 8.0;
const INSPECTOR_VECTOR_FIELD_GAP: f32 = 6.0;
const INSPECTOR_ACTION_BUTTON_WIDTH: f32 = 84.0;
const INSPECTOR_ACTION_BUTTON_HEIGHT: f32 = 24.0;

pub(super) struct InspectorVisualFields {
    pub(super) info: String,
    pub(super) name: String,
    pub(super) parent: String,
    pub(super) x: String,
    pub(super) y: String,
    pub(super) z: String,
    pub(super) delete_enabled: bool,
    pub(super) plugin_components: Vec<InspectorPluginComponentViewData>,
}

impl InspectorVisualFields {
    pub(super) fn from_view_data(data: &InspectorPaneViewData) -> Self {
        Self {
            info: data.info.to_string(),
            name: data.inspector_name.to_string(),
            parent: data.inspector_parent.to_string(),
            x: data.inspector_x.to_string(),
            y: data.inspector_y.to_string(),
            z: data.inspector_z.to_string(),
            delete_enabled: data.delete_enabled,
            plugin_components: data.plugin_components.clone(),
        }
    }

    fn has_selection(&self) -> bool {
        self.delete_enabled || !self.name.trim().is_empty()
    }
}

pub(super) fn inspector_field_nodes(
    fields: &InspectorVisualFields,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let body_frame = inspector_body_frame(template_nodes, content_size);
    let width = if body_frame.width > 0.0 {
        body_frame.width
    } else {
        content_size.width.max(0.0)
    };
    let field_width = (width - INSPECTOR_FIELD_PADDING * 2.0).max(0.0);
    let start_x = body_frame.x + INSPECTOR_FIELD_PADDING;
    let start_y = body_frame.y + INSPECTOR_FIELD_PADDING;
    let field_disabled = !fields.has_selection();
    let mut nodes = Vec::new();

    let panel_height = inspector_field_panel_height(fields);
    let mut panel = inspector_node(
        "inspector_field_panel",
        "InspectorEditableFieldsPanel",
        "Panel",
        "",
        host_contract::TemplateNodeFrameData {
            x: body_frame.x,
            y: body_frame.y,
            width,
            height: panel_height,
        },
    );
    panel.surface_variant = if field_disabled {
        "inset".into()
    } else {
        "inspector-fields".into()
    };
    panel.text_tone = if field_disabled {
        "muted".into()
    } else {
        "default".into()
    };
    panel.selected = !field_disabled;
    nodes.push(panel);

    nodes.push(inspector_text_field_node(
        "name",
        "NameField",
        "Name",
        &fields.name,
        "inspector.field.name.edit",
        start_x,
        start_y,
        field_width,
        field_disabled,
    ));
    nodes.push(inspector_text_field_node(
        "parent",
        "ParentField",
        "Parent",
        &fields.parent,
        "inspector.field.parent.edit",
        start_x,
        start_y + INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP,
        field_width,
        field_disabled,
    ));

    let transform_label_y = start_y + (INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP) * 2.0;
    let mut transform_label = inspector_node(
        "inspector_transform_label",
        "InspectorTransformLabel",
        "Label",
        "Transform",
        host_contract::TemplateNodeFrameData {
            x: start_x,
            y: transform_label_y,
            width: field_width,
            height: 18.0,
        },
    );
    transform_label.text_tone = "muted".into();
    nodes.push(transform_label);

    let vector_y = transform_label_y + 20.0;
    let vector_width = ((field_width - INSPECTOR_VECTOR_FIELD_GAP * 2.0) / 3.0).max(0.0);
    nodes.push(inspector_number_field_node(
        "position_x",
        "PositionXField",
        "X",
        &fields.x,
        "inspector.transform.position_x.edit",
        start_x,
        vector_y,
        vector_width,
        field_disabled,
    ));
    nodes.push(inspector_number_field_node(
        "position_y",
        "PositionYField",
        "Y",
        &fields.y,
        "inspector.transform.position_y.edit",
        start_x + vector_width + INSPECTOR_VECTOR_FIELD_GAP,
        vector_y,
        vector_width,
        field_disabled,
    ));
    nodes.push(inspector_number_field_node(
        "position_z",
        "PositionZField",
        "Z",
        &fields.z,
        "inspector.transform.position_z.edit",
        start_x + (vector_width + INSPECTOR_VECTOR_FIELD_GAP) * 2.0,
        vector_y,
        vector_width,
        field_disabled,
    ));

    let mut next_y = vector_y + INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
    let plugin_nodes =
        inspector_plugin_component_nodes(fields, start_x, next_y, field_width, field_disabled);
    if !plugin_nodes.is_empty() {
        next_y += inspector_plugin_component_height(fields);
        nodes.extend(plugin_nodes);
    }

    if let Some(message) = inspector_plugin_component_fallback_message(&fields.info) {
        let mut diagnostic = inspector_node(
            "inspector_plugin_component_fallback",
            "InspectorPluginComponentFallback",
            "Diagnostic",
            "Plugin component protected",
            host_contract::TemplateNodeFrameData {
                x: start_x,
                y: next_y,
                width: field_width,
                height: INSPECTOR_FIELD_ROW_HEIGHT,
            },
        );
        diagnostic.value_text = fields.info.clone().into();
        diagnostic.validation_level = "warning".into();
        diagnostic.validation_message = message.into();
        diagnostic.surface_variant = "inset".into();
        diagnostic.text_tone = "warning".into();
        diagnostic.disabled = true;
        nodes.push(diagnostic);
        next_y += INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
    } else if field_disabled {
        let mut empty = inspector_node(
            "inspector_empty_selection_hint",
            "InspectorEmptySelectionHint",
            "Label",
            "No scene entity selected",
            host_contract::TemplateNodeFrameData {
                x: start_x,
                y: next_y,
                width: field_width,
                height: 20.0,
            },
        );
        empty.text_tone = "muted".into();
        nodes.push(empty);
        next_y += 20.0 + INSPECTOR_FIELD_ROW_GAP;
    }

    nodes.push(inspector_action_button_node(
        "apply",
        "ApplyBatchButton",
        "Apply",
        "inspector.apply_batch.invoke",
        start_x,
        next_y,
        field_disabled,
    ));
    nodes.push(inspector_action_button_node(
        "delete",
        "DeleteSelected",
        "Delete",
        "workbench.selection.delete_selected",
        start_x + INSPECTOR_ACTION_BUTTON_WIDTH + INSPECTOR_FIELD_ROW_GAP,
        next_y,
        !fields.delete_enabled,
    ));

    nodes
}

fn inspector_field_panel_height(fields: &InspectorVisualFields) -> f32 {
    let base_rows = 4.0;
    let diagnostic_rows = if inspector_plugin_component_fallback_message(&fields.info).is_some()
        || !fields.has_selection()
    {
        1.0
    } else {
        0.0
    };
    INSPECTOR_FIELD_PADDING * 2.0
        + base_rows * INSPECTOR_FIELD_ROW_HEIGHT
        + (base_rows + diagnostic_rows) * INSPECTOR_FIELD_ROW_GAP
        + diagnostic_rows * INSPECTOR_FIELD_ROW_HEIGHT
        + inspector_plugin_component_height(fields)
        + INSPECTOR_ACTION_BUTTON_HEIGHT
}

fn inspector_plugin_component_nodes(
    fields: &InspectorVisualFields,
    x: f32,
    mut y: f32,
    width: f32,
    field_disabled: bool,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut nodes = Vec::new();
    for component in &fields.plugin_components {
        let component_key = inspector_component_key(&component.component_id);
        let mut header = inspector_node(
            format!("inspector_plugin_component_header_{component_key}"),
            format!("PluginComponentHeader:{}", component.component_id),
            "Label",
            component.display_name.clone(),
            host_contract::TemplateNodeFrameData {
                x,
                y,
                width,
                height: 20.0,
            },
        );
        header.text_tone = if component.customization_available {
            "default".into()
        } else {
            "warning".into()
        };
        header.surface_variant = if component.customization_available {
            "panel".into()
        } else {
            "inset".into()
        };
        header.selected = component.customization_available;
        header.focused = component.customization_available;
        if let Some(template_id) = &component.customization_template_id {
            header.value_text = template_id.clone().into();
        }
        if let Some(ui_document) = &component.customization_ui_document {
            header.validation_message = ui_document.clone().into();
        }
        nodes.push(header);
        y += 20.0 + INSPECTOR_FIELD_ROW_GAP;

        if let Some(diagnostic) = &component.diagnostic {
            let mut diagnostic_node = inspector_node(
                format!("inspector_plugin_component_diagnostic_{component_key}"),
                format!("PluginComponentDiagnostic:{}", component.component_id),
                "Diagnostic",
                "Plugin component protected",
                host_contract::TemplateNodeFrameData {
                    x,
                    y,
                    width,
                    height: INSPECTOR_FIELD_ROW_HEIGHT,
                },
            );
            diagnostic_node.value_text = diagnostic.clone().into();
            diagnostic_node.validation_level = "warning".into();
            diagnostic_node.validation_message = diagnostic.clone().into();
            diagnostic_node.surface_variant = "inset".into();
            diagnostic_node.text_tone = "warning".into();
            diagnostic_node.disabled = true;
            nodes.push(diagnostic_node);
            y += INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
        }

        for property in &component.properties {
            let control_id = inspector_dynamic_component_control_id(&property.field_id);
            let disabled =
                field_disabled || !component.customization_available || !property.editable;
            let mut node = if inspector_numeric_kind(&property.value_kind) {
                inspector_number_field_node(
                    &inspector_component_key(&property.field_id),
                    &control_id,
                    property.label.as_str(),
                    &property.value,
                    &inspector_dynamic_component_edit_action_id(&property.field_id),
                    x,
                    y,
                    width,
                    disabled,
                )
            } else {
                inspector_text_field_node(
                    &inspector_component_key(&property.field_id),
                    &control_id,
                    property.label.as_str(),
                    &property.value,
                    &format!("InspectorView/{control_id}"),
                    x,
                    y,
                    width,
                    disabled,
                )
            };
            if !component.customization_available {
                node.validation_level = "warning".into();
                node.validation_message = component
                    .diagnostic
                    .clone()
                    .unwrap_or_else(|| "Plugin inspector customization unavailable".to_string())
                    .into();
            }
            nodes.push(node);
            y += INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
        }
    }
    nodes
}

fn inspector_plugin_component_height(fields: &InspectorVisualFields) -> f32 {
    fields
        .plugin_components
        .iter()
        .map(|component| {
            let diagnostic_rows = if component.diagnostic.is_some() {
                1.0
            } else {
                0.0
            };
            20.0 + INSPECTOR_FIELD_ROW_GAP
                + diagnostic_rows * (INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP)
                + component.properties.len() as f32
                    * (INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP)
        })
        .sum()
}

fn inspector_dynamic_component_control_id(field_id: &str) -> String {
    format!("DynamicComponentField:{field_id}")
}

fn inspector_dynamic_component_edit_action_id(field_id: &str) -> String {
    format!(
        "inspector.dynamic_component.{}.edit",
        inspector_component_key(field_id)
    )
}

fn inspector_component_key(value: &str) -> String {
    let mut key = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
        } else {
            key.push_str("_u");
            key.push_str(&format!("{:x}", ch as u32));
            key.push('_');
        }
    }
    key
}

fn inspector_numeric_kind(value_kind: &str) -> bool {
    matches!(
        value_kind.to_ascii_lowercase().as_str(),
        "number"
            | "float"
            | "scalar"
            | "real"
            | "double"
            | "integer"
            | "int"
            | "signed"
            | "unsigned"
            | "u32"
            | "u64"
            | "i32"
            | "i64"
    )
}

fn inspector_body_frame(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> host_contract::TemplateNodeFrameData {
    template_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "InspectorBodySection")
        .map(|node| node.frame.clone())
        .filter(|frame| frame.width > 0.0 || frame.height > 0.0)
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: content_size.width.max(0.0),
            height: content_size.height.max(0.0),
        })
}

fn inspector_text_field_node(
    suffix: &str,
    control_id: &str,
    label: &str,
    value: &str,
    edit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    disabled: bool,
) -> host_contract::TemplatePaneNodeData {
    let mut node = inspector_node(
        format!("inspector_field_{suffix}"),
        control_id,
        "InputField",
        label,
        host_contract::TemplateNodeFrameData {
            x,
            y,
            width,
            height: INSPECTOR_FIELD_ROW_HEIGHT,
        },
    );
    node.component_role = "input-field".into();
    node.value_text = value.to_string().into();
    node.edit_action_id = edit_action_id.to_string().into();
    node.commit_action_id = "inspector.apply_batch.commit".into();
    node.disabled = disabled;
    node.surface_variant = if disabled {
        "inset".into()
    } else {
        "inspector-field".into()
    };
    node.text_tone = if disabled {
        "muted".into()
    } else {
        "default".into()
    };
    node.corner_radius = 4.0;
    node.border_width = 1.0;
    node
}

fn inspector_number_field_node(
    suffix: &str,
    control_id: &str,
    label: &str,
    value: &str,
    edit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    disabled: bool,
) -> host_contract::TemplatePaneNodeData {
    let mut node = inspector_text_field_node(
        suffix,
        control_id,
        label,
        value,
        edit_action_id,
        x,
        y,
        width,
        disabled,
    );
    node.role = "NumberField".into();
    node.component_role = "number-field".into();
    node.value_number = value.parse::<f32>().unwrap_or(0.0);
    node
}

fn inspector_action_button_node(
    suffix: &str,
    control_id: &str,
    label: &str,
    action_id: &str,
    x: f32,
    y: f32,
    disabled: bool,
) -> host_contract::TemplatePaneNodeData {
    let mut node = inspector_node(
        format!("inspector_action_{suffix}"),
        control_id,
        "Button",
        label,
        host_contract::TemplateNodeFrameData {
            x,
            y,
            width: INSPECTOR_ACTION_BUTTON_WIDTH,
            height: INSPECTOR_ACTION_BUTTON_HEIGHT,
        },
    );
    let dispatch_kind = if disabled { "" } else { "inspector" };
    node.dispatch_kind = dispatch_kind.into();
    node.action_id = action_id.to_string().into();
    node.button_variant = "secondary".into();
    node.surface_variant = if disabled {
        "inset".into()
    } else {
        "panel".into()
    };
    node.text_tone = if disabled {
        "muted".into()
    } else {
        "default".into()
    };
    node.selected = !disabled;
    node.focused = !disabled;
    node.disabled = disabled;
    node
}

fn inspector_plugin_component_fallback_message(info: &str) -> Option<String> {
    let lower = info.to_ascii_lowercase();
    let mentions_plugin_component =
        lower.contains("plugin") || lower.contains("inspector customization");
    let mentions_unavailable = lower.contains("unloaded")
        || lower.contains("missing")
        || lower.contains("unavailable")
        || lower.contains("disabled");
    (mentions_plugin_component && mentions_unavailable).then(|| {
        "Plugin inspector customization is unavailable; serialized component data stays protected until the plugin reloads."
            .to_string()
    })
}

fn inspector_node(
    node_id: impl Into<String>,
    control_id: impl Into<String>,
    role: impl Into<String>,
    text: impl Into<String>,
    frame: host_contract::TemplateNodeFrameData,
) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: node_id.into().into(),
        control_id: control_id.into().into(),
        role: role.into().into(),
        text: text.into().into(),
        frame,
        ..host_contract::TemplatePaneNodeData::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        inspector_component_key, inspector_dynamic_component_control_id,
        inspector_dynamic_component_edit_action_id,
    };

    #[test]
    fn plugin_field_identity_distinguishes_punctuation_that_was_previously_collapsed() {
        let hyphenated = "camera-offset";
        let underscored = "camera_offset";

        assert_ne!(
            inspector_component_key(hyphenated),
            inspector_component_key(underscored)
        );
        assert_ne!(
            inspector_dynamic_component_control_id(hyphenated),
            inspector_dynamic_component_control_id(underscored)
        );
        assert_ne!(
            inspector_dynamic_component_edit_action_id(hyphenated),
            inspector_dynamic_component_edit_action_id(underscored)
        );
    }
}
