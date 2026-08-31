use crate::ui::{
    EditorActivityHost, EditorActivityKind, EditorActivityReflection,
    EditorFloatingWindowReflectionModel,
};
use serde_json::Value;

use crate::ui::workbench::snapshot::{
    DocumentWorkspaceSnapshot, FloatingWindowSnapshot, ViewTabSnapshot,
};
use crate::ui::workbench::view::ViewKind;

use super::activity_actions::activity_actions_for_tab;
use super::name_mapping::content_kind_name;

#[cfg(test)]
#[path = "activity_collection/single_pass_tests.rs"]
mod single_pass_tests;

pub(super) fn collect_workspace_activities(
    workspace: &DocumentWorkspaceSnapshot,
    host: EditorActivityHost,
) -> Vec<EditorActivityReflection> {
    let mut activities = Vec::with_capacity(workspace_activity_count(workspace));
    collect_workspace_activities_into(workspace, &host, &mut activities);
    activities
}

fn workspace_activity_count(workspace: &DocumentWorkspaceSnapshot) -> usize {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            workspace_activity_count(first).saturating_add(workspace_activity_count(second))
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs.len(),
    }
}

fn collect_workspace_activities_into(
    workspace: &DocumentWorkspaceSnapshot,
    host: &EditorActivityHost,
    activities: &mut Vec<EditorActivityReflection>,
) {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            collect_workspace_activities_into(first, host, activities);
            collect_workspace_activities_into(second, host, activities);
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, .. } => {
            activities.extend(tabs.iter().map(|tab| activity_from_tab(tab, host.clone())));
        }
    }
}

pub(super) fn floating_window_model(
    window: &FloatingWindowSnapshot,
) -> EditorFloatingWindowReflectionModel {
    EditorFloatingWindowReflectionModel {
        window_id: window.window_id.0.clone(),
        title: window.title.clone(),
        activities: collect_workspace_activities(
            &window.workspace,
            EditorActivityHost::FloatingWindow(window.window_id.0.clone()),
        ),
    }
}

pub(super) fn activity_from_tab(
    tab: &ViewTabSnapshot,
    host: EditorActivityHost,
) -> EditorActivityReflection {
    let mut properties = std::collections::BTreeMap::from([
        ("icon_key".to_string(), Value::String(tab.icon_key.clone())),
        (
            "content_kind".to_string(),
            Value::String(content_kind_name(tab.content_kind).to_string()),
        ),
        ("placeholder".to_string(), Value::Bool(tab.placeholder)),
    ]);
    if let Value::Object(object) = &tab.serializable_payload {
        for (key, value) in object {
            properties.insert(format!("payload.{key}"), value.clone());
        }
    }

    EditorActivityReflection {
        instance_id: tab.instance_id.0.clone(),
        descriptor_id: tab.descriptor_id.0.clone(),
        title: tab.title.clone(),
        kind: match tab.kind {
            ViewKind::ActivityView => EditorActivityKind::ActivityView,
            ViewKind::ActivityWindow => EditorActivityKind::ActivityWindow,
        },
        host,
        visible: true,
        enabled: !tab.placeholder,
        dirty: tab.dirty,
        properties,
        actions: activity_actions_for_tab(tab),
    }
}
