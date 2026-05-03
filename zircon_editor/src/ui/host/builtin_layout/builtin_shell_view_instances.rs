use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

use super::super::editor_capabilities::EditorCapabilitySnapshot;
use super::super::editor_subsystems::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS;

pub(super) fn builtin_shell_view_instances(
    snapshot: &EditorCapabilitySnapshot,
) -> Vec<ViewInstance> {
    let mut instances = vec![
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.project#1"),
            descriptor_id: ViewDescriptorId::new("editor.project"),
            title: "Project".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.assets#1"),
            descriptor_id: ViewDescriptorId::new("editor.assets"),
            title: "Assets".to_string(),
            serializable_payload: serde_json::json!({ "root": "crate://" }),
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.module_plugins#1"),
            descriptor_id: ViewDescriptorId::new("editor.module_plugins"),
            title: "Plugin Manager".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftBottom),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.hierarchy#1"),
            descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
            title: "Hierarchy".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.inspector#1"),
            descriptor_id: ViewDescriptorId::new("editor.inspector"),
            title: "Inspector".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.console#1"),
            descriptor_id: ViewDescriptorId::new("editor.console"),
            title: "Console".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::BottomLeft),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.runtime_diagnostics#1"),
            descriptor_id: ViewDescriptorId::new("editor.runtime_diagnostics"),
            title: "Runtime Diagnostics".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::BottomRight),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.build_export_desktop#1"),
            descriptor_id: ViewDescriptorId::new("editor.build_export_desktop"),
            title: "Desktop Export".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::BottomRight),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.game#1"),
            descriptor_id: ViewDescriptorId::new("editor.game"),
            title: "Game".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            descriptor_id: ViewDescriptorId::new("editor.scene"),
            title: "Scene".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
    ];
    instances.retain(|instance| match instance.descriptor_id.0.as_str() {
        "editor.runtime_diagnostics" => snapshot.is_enabled(EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS),
        _ => true,
    });
    instances
}
