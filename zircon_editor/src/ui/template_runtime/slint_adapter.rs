use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::ui::component::{UiComponentDescriptorRegistry, UiValue};
use zircon_runtime::ui::{binding::UiEventKind, event_ui::UiRouteId, layout::UiFrame};

use super::SlintUiHostModel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlintUiHostComponentKind {
    Root,
    Toolbar,
    IconButton,
    ActivityRail,
    DocumentHost,
    HorizontalBox,
    TabStrip,
    PaneSurface,
    StatusBar,
    VerticalBox,
    Label,
    Unknown,
}

impl SlintUiHostComponentKind {
    pub fn from_component(component: &str) -> Self {
        match component {
            "UiHostWindow" => Self::Root,
            "UiHostToolbar" => Self::Toolbar,
            "IconButton" => Self::IconButton,
            "ActivityRail" => Self::ActivityRail,
            "DocumentHost" => Self::DocumentHost,
            "HorizontalBox" => Self::HorizontalBox,
            "DocumentTabs" => Self::TabStrip,
            "PaneSurface" => Self::PaneSurface,
            "StatusBar" => Self::StatusBar,
            "VerticalBox" => Self::VerticalBox,
            "Label" => Self::Label,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Root => "Root",
            Self::Toolbar => "Toolbar",
            Self::IconButton => "IconButton",
            Self::ActivityRail => "ActivityRail",
            Self::DocumentHost => "DocumentHost",
            Self::HorizontalBox => "HorizontalBox",
            Self::TabStrip => "TabStrip",
            Self::PaneSurface => "PaneSurface",
            Self::StatusBar => "StatusBar",
            Self::VerticalBox => "VerticalBox",
            Self::Label => "Label",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SlintUiHostValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<SlintUiHostValue>),
    Table(BTreeMap<String, SlintUiHostValue>),
    Datetime(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlintUiHostRouteProjection {
    pub binding_id: String,
    pub event_kind: UiEventKind,
    pub route_id: Option<UiRouteId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiHostNodeModel {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub kind: SlintUiHostComponentKind,
    pub component: String,
    pub control_id: Option<String>,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub z_index: i32,
    pub text: Option<String>,
    pub icon: Option<String>,
    pub component_role: Option<String>,
    pub value_text: Option<String>,
    pub validation_level: Option<String>,
    pub validation_message: Option<String>,
    pub popup_open: bool,
    pub has_popup_anchor: bool,
    pub popup_anchor_x: f64,
    pub popup_anchor_y: f64,
    pub selection_state: Option<String>,
    pub options_text: Option<String>,
    pub options: Vec<String>,
    pub collection_items: Vec<String>,
    pub menu_items: Vec<String>,
    pub accepted_drag_payloads: Vec<String>,
    pub drop_source_summary: Option<String>,
    pub checked: bool,
    pub expanded: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub dragging: bool,
    pub drop_hovered: bool,
    pub active_drag_target: bool,
    pub disabled: bool,
    pub properties: BTreeMap<String, SlintUiHostValue>,
    pub style_tokens: BTreeMap<String, String>,
    pub routes: Vec<SlintUiHostRouteProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiHostProjection {
    pub document_id: String,
    pub nodes: Vec<SlintUiHostNodeModel>,
}

impl SlintUiHostProjection {
    pub fn node_by_control_id(&self, control_id: &str) -> Option<&SlintUiHostNodeModel> {
        self.nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(control_id))
    }
}

#[derive(Default)]
pub struct SlintUiHostAdapter;

impl SlintUiHostAdapter {
    pub fn build_projection(host_model: &SlintUiHostModel) -> SlintUiHostProjection {
        let component_registry = UiComponentDescriptorRegistry::editor_showcase();
        SlintUiHostProjection {
            document_id: host_model.document_id.clone(),
            nodes: host_model
                .nodes
                .iter()
                .map(|node| {
                    let component_descriptor = component_registry.descriptor(&node.component);
                    let properties = node
                        .attributes
                        .iter()
                        .map(|(key, value)| (key.clone(), map_value(value)))
                        .collect::<BTreeMap<_, _>>();
                    let disabled = bool_attribute(&node.attributes, "disabled").unwrap_or(false)
                        || bool_attribute(&node.attributes, "enabled") == Some(false);
                    let popup_anchor_x = float_attribute(&node.attributes, "popup_anchor_x");
                    let popup_anchor_y = float_attribute(&node.attributes, "popup_anchor_y");
                    let has_popup_anchor = popup_anchor_x.is_some() && popup_anchor_y.is_some();
                    SlintUiHostNodeModel {
                        node_id: node.node_id.clone(),
                        parent_id: node.parent_id.clone(),
                        kind: SlintUiHostComponentKind::from_component(&node.component),
                        component: node.component.clone(),
                        control_id: node.control_id.clone(),
                        frame: node.frame,
                        clip_frame: node.clip_frame,
                        z_index: node.z_index,
                        text: extract_non_empty_string(&properties, "text")
                            .or_else(|| extract_non_empty_string(&properties, "label")),
                        icon: extract_string(&properties, "icon"),
                        properties,
                        style_tokens: node.style_tokens.clone(),
                        component_role: component_descriptor
                            .map(|descriptor| descriptor.role.clone()),
                        value_text: string_attribute(&node.attributes, "value_text").or_else(
                            || {
                                node.attributes
                                    .get("value")
                                    .or_else(|| node.attributes.get("items"))
                                    .or_else(|| node.attributes.get("entries"))
                                    .map(UiValue::from_toml)
                                    .map(|value| value.display_text())
                            },
                        ),
                        validation_level: string_attribute(&node.attributes, "validation_level")
                            .or_else(|| {
                                component_descriptor.map(|_| {
                                    if disabled { "disabled" } else { "normal" }.to_string()
                                })
                            }),
                        validation_message: string_attribute(
                            &node.attributes,
                            "validation_message",
                        ),
                        popup_open: bool_attribute(&node.attributes, "popup_open").unwrap_or(false),
                        has_popup_anchor,
                        popup_anchor_x: popup_anchor_x.unwrap_or(0.0),
                        popup_anchor_y: popup_anchor_y.unwrap_or(0.0),
                        selection_state: string_attribute(&node.attributes, "selection_state")
                            .or_else(|| {
                                bool_attribute(&node.attributes, "multiple").map(|multiple| {
                                    if multiple {
                                        "multi".to_string()
                                    } else {
                                        "single".to_string()
                                    }
                                })
                            }),
                        options_text: options_text_attribute(&node.attributes, "options"),
                        options: options_attribute(&node.attributes, "options"),
                        collection_items: string_array_attribute(
                            &node.attributes,
                            "collection_items",
                        ),
                        menu_items: string_array_attribute(&node.attributes, "menu_items"),
                        accepted_drag_payloads: component_descriptor
                            .map(|descriptor| {
                                descriptor
                                    .drop_policy
                                    .accepts
                                    .iter()
                                    .map(|kind| kind.as_str().to_string())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        drop_source_summary: string_attribute(
                            &node.attributes,
                            "drop_source_summary",
                        ),
                        checked: bool_attribute(&node.attributes, "checked")
                            .or_else(|| bool_attribute(&node.attributes, "value"))
                            .unwrap_or(false),
                        expanded: bool_attribute(&node.attributes, "expanded").unwrap_or(false),
                        focused: bool_attribute(&node.attributes, "focused").unwrap_or(false),
                        hovered: bool_attribute(&node.attributes, "hovered").unwrap_or(false),
                        pressed: bool_attribute(&node.attributes, "pressed").unwrap_or(false),
                        dragging: bool_attribute(&node.attributes, "dragging").unwrap_or(false),
                        drop_hovered: bool_attribute(&node.attributes, "drop_hovered")
                            .unwrap_or(false),
                        active_drag_target: bool_attribute(&node.attributes, "active_drag_target")
                            .unwrap_or(false),
                        disabled,
                        routes: node
                            .bindings
                            .iter()
                            .map(|binding| SlintUiHostRouteProjection {
                                binding_id: binding.binding_id.clone(),
                                event_kind: binding.event_kind,
                                route_id: binding.route_id,
                            })
                            .collect(),
                    }
                })
                .collect(),
        }
    }
}

fn string_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    attributes
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn bool_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<bool> {
    attributes.get(key).and_then(Value::as_bool)
}

fn float_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<f64> {
    attributes.get(key).and_then(Value::as_float)
}

fn options_text_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    let options = options_attribute(attributes, key);
    if options.is_empty() {
        None
    } else {
        Some(options.join(", "))
    }
}

fn options_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Vec<String> {
    string_array_attribute(attributes, key)
}

fn string_array_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Vec<String> {
    attributes
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn extract_string(properties: &BTreeMap<String, SlintUiHostValue>, key: &str) -> Option<String> {
    match properties.get(key) {
        Some(SlintUiHostValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn extract_non_empty_string(
    properties: &BTreeMap<String, SlintUiHostValue>,
    key: &str,
) -> Option<String> {
    extract_string(properties, key).filter(|value| !value.is_empty())
}

fn map_value(value: &Value) -> SlintUiHostValue {
    match value {
        Value::String(value) => SlintUiHostValue::String(value.clone()),
        Value::Integer(value) => SlintUiHostValue::Integer(*value),
        Value::Float(value) => SlintUiHostValue::Float(*value),
        Value::Boolean(value) => SlintUiHostValue::Bool(*value),
        Value::Datetime(value) => SlintUiHostValue::Datetime(value.to_string()),
        Value::Array(values) => SlintUiHostValue::Array(values.iter().map(map_value).collect()),
        Value::Table(values) => SlintUiHostValue::Table(
            values
                .iter()
                .map(|(key, value)| (key.clone(), map_value(value)))
                .collect(),
        ),
    }
}
