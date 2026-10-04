//! 反射活动保留页面或浮窗中的标签顺序及宿主身份，供适配器生成可发现节点。
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
#[path = "activity_collection/tests/single_pass_tests.rs"]
mod single_pass_tests;

/// 将文档树中的标签按原布局顺序归入给定宿主；split仅分组，不创造活动身份。
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

/// 保留实例和描述符双重身份；占位仍可见，但不应作为可调用活动。
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
