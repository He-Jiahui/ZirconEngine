use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags},
    layout::{AxisConstraint, BoxConstraints, StretchMode, UiContainerKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

use super::ReferenceSurfaceBuilder;

impl ReferenceSurfaceBuilder {
    pub(super) fn panel_node(
        &self,
        id: UiNodeId,
        path: &str,
        constraints: BoxConstraints,
        container: UiContainerKind,
        background: &str,
        border: Option<(&str, f32)>,
    ) -> UiTreeNode {
        let mut styles = vec![("background", value_string(background))];
        if let Some((color, width)) = border {
            styles.push(("border", value_string(color)));
            styles.push(("border_width", value_float(width)));
        }
        visual_node(
            id,
            path,
            "Panel",
            constraints,
            container,
            metadata("Panel", path, None, styles, Vec::new(), None),
        )
        .with_clip_to_bounds(true)
    }

    pub(super) fn label_node(
        &self,
        id: UiNodeId,
        path: &str,
        text: &str,
        constraints: BoxConstraints,
        font_size: f32,
        color: &str,
    ) -> UiTreeNode {
        visual_node(
            id,
            path,
            "Label",
            constraints,
            UiContainerKind::Container,
            metadata(
                "Label",
                path,
                Some(text),
                vec![
                    ("foreground", value_string(color)),
                    ("font_size", value_float(font_size)),
                    ("line_height", value_float(font_size + 4.0)),
                ],
                Vec::new(),
                None,
            ),
        )
    }

    pub(super) fn button_node(
        &self,
        id: UiNodeId,
        path: &str,
        label: &str,
        constraints: BoxConstraints,
        behavior: UiWidgetBehavior,
        active: bool,
    ) -> UiTreeNode {
        let background = if active {
            self.palette.control_background_active
        } else {
            self.palette.control_background
        };
        let foreground = if active {
            "#effcff"
        } else {
            self.palette.text_secondary
        };
        visual_node(
            id,
            path,
            "Button",
            constraints,
            UiContainerKind::Container,
            metadata(
                "Button",
                path,
                (!label.is_empty()).then_some(label),
                vec![
                    ("background", value_string(background)),
                    ("border", value_string(self.palette.control_border)),
                    ("border_width", value_float(1.0)),
                    ("corner_radius", value_float(5.0)),
                    ("foreground", value_string(foreground)),
                    ("font_size", value_float(13.0)),
                    ("line_height", value_float(17.0)),
                ],
                active
                    .then(|| vec![("selected", Value::Boolean(true))])
                    .unwrap_or_default(),
                Some(WidgetSpec {
                    behavior,
                    checked_property: None,
                    bindings: vec![binding(path, UiEventKind::Click)],
                }),
            ),
        )
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
    }

    pub(super) fn field_node(
        &self,
        id: UiNodeId,
        path: &str,
        text: &str,
        constraints: BoxConstraints,
        focused: bool,
    ) -> UiTreeNode {
        let border = if focused {
            self.palette.accent
        } else {
            self.palette.control_border
        };
        visual_node(
            id,
            path,
            "TextField",
            constraints,
            UiContainerKind::Container,
            metadata(
                "TextField",
                path,
                Some(text),
                vec![
                    ("background", value_string("#111820")),
                    ("border", value_string(border)),
                    ("border_width", value_float(1.0)),
                    ("corner_radius", value_float(4.0)),
                    ("foreground", value_string(self.palette.text_secondary)),
                    ("font_size", value_float(13.0)),
                    ("line_height", value_float(17.0)),
                ],
                vec![
                    ("value", value_string(text)),
                    ("editable_text", Value::Boolean(true)),
                    ("focused", Value::Boolean(focused)),
                ],
                Some(WidgetSpec {
                    behavior: UiWidgetBehavior::TextInput,
                    checked_property: None,
                    bindings: vec![binding(path, UiEventKind::Focus)],
                }),
            ),
        )
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
    }

    pub(super) fn toggle_node(
        &self,
        id: UiNodeId,
        path: &str,
        label: &str,
        checked: bool,
    ) -> UiTreeNode {
        let background = if checked {
            self.palette.accent_soft
        } else {
            self.palette.control_background
        };
        visual_node(
            id,
            path,
            "Checkbox",
            fixed_height_box(self.metrics.control_height),
            UiContainerKind::Container,
            metadata(
                "Checkbox",
                path,
                Some(label),
                vec![
                    ("background", value_string(background)),
                    ("border", value_string(self.palette.control_border)),
                    ("border_width", value_float(1.0)),
                    ("corner_radius", value_float(4.0)),
                    ("foreground", value_string(self.palette.text_secondary)),
                    ("font_size", value_float(13.0)),
                    ("line_height", value_float(17.0)),
                ],
                vec![("checked", Value::Boolean(checked))],
                Some(WidgetSpec {
                    behavior: UiWidgetBehavior::Toggle,
                    checked_property: Some("checked"),
                    bindings: vec![binding(path, UiEventKind::Change)],
                }),
            ),
        )
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            checked,
            ..pointer_state()
        })
    }

    pub(super) fn tree_row_node(
        &self,
        id: UiNodeId,
        path: &str,
        label: &str,
        depth: usize,
        selected: bool,
    ) -> UiTreeNode {
        let background = if selected {
            self.palette.accent_soft
        } else {
            self.palette.panel_background
        };
        let text = format!("{}{}", "  ".repeat(depth), label);
        visual_node(
            id,
            path,
            "TreeRow",
            fixed_height_box(self.metrics.compact_row_height),
            UiContainerKind::Container,
            metadata(
                "TreeRow",
                path,
                Some(&text),
                vec![
                    ("background", value_string(background)),
                    ("foreground", value_string(self.palette.text_secondary)),
                    ("font_size", value_float(13.0)),
                    ("line_height", value_float(17.0)),
                    ("corner_radius", value_float(4.0)),
                ],
                selected
                    .then(|| vec![("selected", Value::Boolean(true))])
                    .unwrap_or_default(),
                Some(WidgetSpec {
                    behavior: UiWidgetBehavior::Button,
                    checked_property: None,
                    bindings: vec![binding(path, UiEventKind::Click)],
                }),
            ),
        )
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
    }
}

#[derive(Clone, Debug)]
struct WidgetSpec {
    behavior: UiWidgetBehavior,
    checked_property: Option<&'static str>,
    bindings: Vec<UiBindingRef>,
}

fn visual_node(
    id: UiNodeId,
    path: &str,
    component: &str,
    constraints: BoxConstraints,
    container: UiContainerKind,
    metadata: UiTemplateNodeMetadata,
) -> UiTreeNode {
    UiTreeNode::new(id, UiNodePath::new(path))
        .with_constraints(constraints)
        .with_container(container)
        .with_layout_stretch_axes(
            constraints.width.stretch_mode == StretchMode::Stretch,
            constraints.height.stretch_mode == StretchMode::Stretch,
        )
        .with_template_metadata(UiTemplateNodeMetadata {
            component: component.to_string(),
            ..metadata
        })
}

pub(super) fn spacer_node(id: UiNodeId, path: &str, constraints: BoxConstraints) -> UiTreeNode {
    UiTreeNode::new(id, UiNodePath::new(path))
        .with_constraints(constraints)
        .with_layout_stretch_axes(
            constraints.width.stretch_mode == StretchMode::Stretch,
            constraints.height.stretch_mode == StretchMode::Stretch,
        )
}

fn metadata(
    component: &str,
    control_id: &str,
    text: Option<&str>,
    styles: Vec<(&str, Value)>,
    attributes: Vec<(&str, Value)>,
    widget: Option<WidgetSpec>,
) -> UiTemplateNodeMetadata {
    let mut attribute_map = values_map(attributes);
    if let Some(text) = text {
        attribute_map.insert("label".to_string(), value_string(text));
        attribute_map.insert("text".to_string(), value_string(text));
    }
    let widget_contract = widget
        .as_ref()
        .map(|widget| UiWidgetContract {
            behavior: widget.behavior,
            checked_property: widget.checked_property.map(str::to_string),
            ..UiWidgetContract::default()
        })
        .unwrap_or_default();
    UiTemplateNodeMetadata {
        component: component.to_string(),
        control_id: Some(control_id.to_string()),
        attributes: attribute_map,
        style_overrides: values_map(styles),
        bindings: widget.map(|widget| widget.bindings).unwrap_or_default(),
        widget: widget_contract,
        ..UiTemplateNodeMetadata::default()
    }
}

fn values_map(values: Vec<(&str, Value)>) -> BTreeMap<String, Value> {
    values
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

fn binding(path: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: format!("{path}/{event:?}"),
        event,
        route: Some(path.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

pub(super) fn stretch_box() -> BoxConstraints {
    BoxConstraints {
        width: stretch_axis(1.0),
        height: stretch_axis(1.0),
    }
}

pub(super) fn fixed_width_box(width: f32) -> BoxConstraints {
    BoxConstraints {
        width: fixed_axis(width),
        height: stretch_axis(1.0),
    }
}

pub(super) fn fixed_height_box(height: f32) -> BoxConstraints {
    BoxConstraints {
        width: stretch_axis(1.0),
        height: fixed_axis(height),
    }
}

pub(super) fn fixed_size_box(width: f32, height: f32) -> BoxConstraints {
    BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
    }
}

fn fixed_axis(value: f32) -> AxisConstraint {
    let value = value.max(0.0);
    AxisConstraint {
        min: value,
        max: value,
        preferred: value,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn stretch_axis(weight: f32) -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: -1.0,
        preferred: 0.0,
        priority: 0,
        weight: weight.max(0.0),
        stretch_mode: StretchMode::Stretch,
    }
}

fn value_string(value: &str) -> Value {
    Value::String(value.to_string())
}

fn value_float(value: f32) -> Value {
    Value::Float(value as f64)
}
