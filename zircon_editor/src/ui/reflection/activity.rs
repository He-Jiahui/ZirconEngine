use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zircon_runtime_interface::ui::event_ui::{
    UiActionDescriptor, UiNodeId, UiPropertyDescriptor, UiStateFlags, UiValueType,
};

use super::builder::SnapshotBuilder;
use super::state_flags::visible_enabled_flags;
use super::value_type::infer_value_type;

#[cfg(test)]
mod capacity_tests;

const ACTIVITY_CORE_PROPERTY_COUNT: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorActivityKind {
    ActivityView,
    ActivityWindow,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorActivityHost {
    Drawer(String),
    DocumentPage(String),
    FloatingWindow(String),
    ExclusivePage(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorActivityReflection {
    pub instance_id: String,
    pub descriptor_id: String,
    pub title: String,
    pub kind: EditorActivityKind,
    pub host: EditorActivityHost,
    pub visible: bool,
    pub enabled: bool,
    pub dirty: bool,
    pub properties: BTreeMap<String, Value>,
    pub actions: Vec<UiActionDescriptor>,
}

pub(super) fn activity_node(
    builder: &mut SnapshotBuilder,
    activity: &EditorActivityReflection,
    node_path: String,
) -> UiNodeId {
    let mut properties = Vec::with_capacity(activity_property_capacity(activity));
    properties.extend([
        UiPropertyDescriptor::new(
            "descriptor_id",
            UiValueType::String,
            json!(activity.descriptor_id),
        ),
        UiPropertyDescriptor::new(
            "host",
            UiValueType::String,
            json!(host_name(&activity.host)),
        ),
        UiPropertyDescriptor::new(
            "kind",
            UiValueType::String,
            json!(kind_name(&activity.kind)),
        ),
    ]);
    properties.extend(activity.properties.iter().map(|(name, value)| {
        UiPropertyDescriptor::new(name.clone(), infer_value_type(value), value.clone())
    }));

    builder.push_node(
        node_path,
        match activity.kind {
            EditorActivityKind::ActivityView => "ActivityView",
            EditorActivityKind::ActivityWindow => "ActivityWindow",
        },
        activity.title.clone(),
        UiStateFlags {
            visible: activity.visible,
            enabled: activity.enabled,
            clickable: false,
            hoverable: false,
            focusable: true,
            pressed: false,
            checked: false,
            dirty: activity.dirty,
        },
        properties,
        activity.actions.clone(),
    )
}

fn activity_property_capacity(activity: &EditorActivityReflection) -> usize {
    ACTIVITY_CORE_PROPERTY_COUNT.saturating_add(activity.properties.len())
}

fn host_name(host: &EditorActivityHost) -> &'static str {
    match host {
        EditorActivityHost::Drawer(_) => "drawer",
        EditorActivityHost::DocumentPage(_) => "document_page",
        EditorActivityHost::FloatingWindow(_) => "floating_window",
        EditorActivityHost::ExclusivePage(_) => "exclusive_page",
    }
}

fn kind_name(kind: &EditorActivityKind) -> &'static str {
    match kind {
        EditorActivityKind::ActivityView => "activity_view",
        EditorActivityKind::ActivityWindow => "activity_window",
    }
}
