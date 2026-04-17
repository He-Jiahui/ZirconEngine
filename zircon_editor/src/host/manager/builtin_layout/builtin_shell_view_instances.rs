use crate::layout::{ActivityDrawerSlot, MainPageId};
use crate::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

pub(super) fn builtin_shell_view_instances() -> Vec<ViewInstance> {
    vec![
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
    ]
}
