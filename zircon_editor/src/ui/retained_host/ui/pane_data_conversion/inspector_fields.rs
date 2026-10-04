use std::fmt::Write as _;

use crate::ui::layouts::windows::workbench_host_window::{
    InspectorPaneViewData, InspectorPluginComponentViewData, PaneContentSize,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::{current_host_metrics, HostControlMetrics};

const INSPECTOR_BASE_NODE_COUNT: usize = 9;

#[cfg(test)]
#[path = "inspector_fields/tests/capacity_tests.rs"]
mod capacity_tests;

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
    inspector_field_nodes_with_metrics(fields, template_nodes, content_size, current_host_metrics())
}

fn inspector_field_nodes_with_metrics(
    fields: &InspectorVisualFields,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
    metrics: HostControlMetrics,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let body_frame = inspector_body_frame(template_nodes, content_size);
    let width = if body_frame.width > 0.0 {
        body_frame.width
    } else {
        content_size.width.max(0.0)
    };
    let field_gap = inspector_field_gap(metrics);
    let field_width = (width - metrics.gap_m * 2.0).max(0.0);
    let start_x = body_frame.x + metrics.gap_m;
    let start_y = body_frame.y + metrics.gap_m;
    let field_disabled = !fields.has_selection();
    let mut nodes = Vec::with_capacity(
        INSPECTOR_BASE_NODE_COUNT
            .saturating_add(inspector_plugin_component_node_count(fields))
            .saturating_add(1),
    );

    let panel_height = inspector_field_panel_height(fields, metrics);
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
        metrics,
    ));
    nodes.push(inspector_text_field_node(
        "parent",
        "ParentField",
        "Parent",
        &fields.parent,
        "inspector.field.parent.edit",
        start_x,
        start_y + metrics.row_height + field_gap,
        field_width,
        field_disabled,
        metrics,
    ));

    let transform_label_y = start_y + (metrics.row_height + field_gap) * 2.0;
    let transform_label_height = inspector_transform_label_height(metrics);
    let mut transform_label = inspector_node(
        "inspector_transform_label",
        "InspectorTransformLabel",
        "Label",
        "Transform",
        host_contract::TemplateNodeFrameData {
            x: start_x,
            y: transform_label_y,
            width: field_width,
            height: transform_label_height,
        },
    );
    transform_label.text_tone = "muted".into();
    nodes.push(transform_label);

    let vector_y = transform_label_y + transform_label_height;
    let vector_width = ((field_width - field_gap * 2.0) / 3.0).max(0.0);
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
        metrics,
    ));
    nodes.push(inspector_number_field_node(
        "position_y",
        "PositionYField",
        "Y",
        &fields.y,
        "inspector.transform.position_y.edit",
        start_x + vector_width + field_gap,
        vector_y,
        vector_width,
        field_disabled,
        metrics,
    ));
    nodes.push(inspector_number_field_node(
        "position_z",
        "PositionZField",
        "Z",
        &fields.z,
        "inspector.transform.position_z.edit",
        start_x + (vector_width + field_gap) * 2.0,
        vector_y,
        vector_width,
        field_disabled,
        metrics,
    ));

    let mut next_y = vector_y + metrics.row_height + field_gap;
    let plugin_nodes = inspector_plugin_component_nodes(
        fields,
        start_x,
        next_y,
        field_width,
        field_disabled,
        metrics,
        field_gap,
    );
    if !plugin_nodes.is_empty() {
        next_y += inspector_plugin_component_height(fields, metrics, field_gap);
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
                height: metrics.row_height,
            },
        );
        diagnostic.value_text = fields.info.clone().into();
        diagnostic.validation_level = "warning".into();
        diagnostic.validation_message = message.into();
        diagnostic.surface_variant = "inset".into();
        diagnostic.text_tone = "warning".into();
        diagnostic.disabled = true;
        nodes.push(diagnostic);
        next_y += metrics.row_height + field_gap;
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
                height: inspector_auxiliary_label_height(metrics),
            },
        );
        empty.text_tone = "muted".into();
        nodes.push(empty);
        next_y += inspector_auxiliary_label_height(metrics) + field_gap;
    }

    nodes.push(inspector_action_button_node(
        "apply",
        "ApplyBatchButton",
        "Apply",
        "inspector.apply_batch.invoke",
        start_x,
        next_y,
        field_disabled,
        metrics,
    ));
    nodes.push(inspector_action_button_node(
        "delete",
        "DeleteSelected",
        "Delete",
        "workbench.selection.delete_selected",
        start_x + inspector_action_button_width(metrics) + field_gap,
        next_y,
        !fields.delete_enabled,
        metrics,
    ));

    nodes
}

fn inspector_field_panel_height(
    fields: &InspectorVisualFields,
    metrics: HostControlMetrics,
) -> f32 {
    let field_gap = inspector_field_gap(metrics);
    let base_rows = 4.0;
    let diagnostic_rows = if inspector_plugin_component_fallback_message(&fields.info).is_some()
        || !fields.has_selection()
    {
        1.0
    } else {
        0.0
    };
    metrics.gap_m * 2.0
        + base_rows * metrics.row_height
        + (base_rows + diagnostic_rows) * field_gap
        + diagnostic_rows * metrics.row_height
        + inspector_plugin_component_height(fields, metrics, field_gap)
        + inspector_action_button_height(metrics)
}

fn inspector_plugin_component_nodes(
    fields: &InspectorVisualFields,
    x: f32,
    mut y: f32,
    width: f32,
    field_disabled: bool,
    metrics: HostControlMetrics,
    field_gap: f32,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut nodes = Vec::with_capacity(inspector_plugin_component_node_count(fields));
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
                height: inspector_auxiliary_label_height(metrics),
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
        y += inspector_auxiliary_label_height(metrics) + field_gap;

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
                    height: metrics.row_height,
                },
            );
            diagnostic_node.value_text = diagnostic.clone().into();
            diagnostic_node.validation_level = "warning".into();
            diagnostic_node.validation_message = diagnostic.clone().into();
            diagnostic_node.surface_variant = "inset".into();
            diagnostic_node.text_tone = "warning".into();
            diagnostic_node.disabled = true;
            nodes.push(diagnostic_node);
            y += metrics.row_height + field_gap;
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
                    metrics,
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
                    metrics,
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
            y += metrics.row_height + field_gap;
        }
    }
    nodes
}

fn inspector_plugin_component_node_count(fields: &InspectorVisualFields) -> usize {
    fields
        .plugin_components
        .iter()
        .map(|component| {
            1usize
                .saturating_add(usize::from(component.diagnostic.is_some()))
                .saturating_add(component.properties.len())
        })
        .sum()
}

fn inspector_plugin_component_height(
    fields: &InspectorVisualFields,
    metrics: HostControlMetrics,
    field_gap: f32,
) -> f32 {
    fields
        .plugin_components
        .iter()
        .map(|component| {
            let diagnostic_rows = if component.diagnostic.is_some() {
                1.0
            } else {
                0.0
            };
            inspector_auxiliary_label_height(metrics)
                + field_gap
                + diagnostic_rows * (metrics.row_height + field_gap)
                + component.properties.len() as f32 * (metrics.row_height + field_gap)
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
            write!(&mut key, "_u{:x}_", ch as u32).expect("writing to a String cannot fail");
        }
    }
    key
}

fn inspector_numeric_kind(value_kind: &str) -> bool {
    [
        "number", "float", "scalar", "real", "double", "integer", "int", "signed", "unsigned",
        "u32", "u64", "i32", "i64",
    ]
    .iter()
    .any(|numeric_kind| value_kind.eq_ignore_ascii_case(numeric_kind))
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
    metrics: HostControlMetrics,
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
            height: metrics.row_height,
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
    node.corner_radius = metrics.radius_control;
    node.border_width = metrics.border_width;
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
    metrics: HostControlMetrics,
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
        metrics,
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
    metrics: HostControlMetrics,
) -> host_contract::TemplatePaneNodeData {
    let mut node = inspector_node(
        format!("inspector_action_{suffix}"),
        control_id,
        "Button",
        label,
        host_contract::TemplateNodeFrameData {
            x,
            y,
            width: inspector_action_button_width(metrics),
            height: inspector_action_button_height(metrics),
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
    node.corner_radius = metrics.radius_control;
    node.border_width = metrics.border_width;
    node
}

fn inspector_transform_label_height(metrics: HostControlMetrics) -> f32 {
    (metrics.row_height - metrics.gap_s * 2.0 - metrics.border_width * 2.0).max(0.0)
}

fn inspector_auxiliary_label_height(metrics: HostControlMetrics) -> f32 {
    (metrics.row_height - metrics.gap_s * 2.0).max(0.0)
}

fn inspector_action_button_width(metrics: HostControlMetrics) -> f32 {
    metrics.button_pad_x * 7.0
}

fn inspector_action_button_height(metrics: HostControlMetrics) -> f32 {
    (metrics.row_height - metrics.gap_s).max(0.0)
}

fn inspector_field_gap(metrics: HostControlMetrics) -> f32 {
    (metrics.gap_m - metrics.gap_s / 2.0).max(0.0)
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
#[path = "tests/inspector_fields.rs"]
mod tests;
