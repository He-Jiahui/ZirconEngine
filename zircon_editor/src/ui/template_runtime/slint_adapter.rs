use std::collections::BTreeMap;

use toml::Value;
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
            "WorkbenchShell" => Self::Root,
            "UiHostWindow" => Self::Root,
            "UiHostToolbar" => Self::Toolbar,
            "UiHostIconButton" => Self::IconButton,
            "ActivityRail" => Self::ActivityRail,
            "DocumentHost" => Self::DocumentHost,
            "HorizontalBox" => Self::HorizontalBox,
            "DocumentTabs" => Self::TabStrip,
            "PaneSurface" => Self::PaneSurface,
            "StatusBar" => Self::StatusBar,
            "VerticalBox" => Self::VerticalBox,
            "UiHostLabel" => Self::Label,
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
        SlintUiHostProjection {
            document_id: host_model.document_id.clone(),
            nodes: host_model
                .nodes
                .iter()
                .map(|node| {
                    let properties = node
                        .attributes
                        .iter()
                        .map(|(key, value)| (key.clone(), map_value(value)))
                        .collect::<BTreeMap<_, _>>();
                    SlintUiHostNodeModel {
                        node_id: node.node_id.clone(),
                        parent_id: node.parent_id.clone(),
                        kind: SlintUiHostComponentKind::from_component(&node.component),
                        component: node.component.clone(),
                        control_id: node.control_id.clone(),
                        frame: node.frame,
                        clip_frame: node.clip_frame,
                        z_index: node.z_index,
                        text: extract_string(&properties, "text")
                            .or_else(|| extract_string(&properties, "label")),
                        icon: extract_string(&properties, "icon"),
                        properties,
                        style_tokens: node.style_tokens.clone(),
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

fn extract_string(properties: &BTreeMap<String, SlintUiHostValue>, key: &str) -> Option<String> {
    match properties.get(key) {
        Some(SlintUiHostValue::String(value)) => Some(value.clone()),
        _ => None,
    }
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
