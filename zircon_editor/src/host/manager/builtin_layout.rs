use std::collections::BTreeMap;

use crate::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, MainHostPageLayout,
    MainPageId, TabStackLayout, WorkbenchLayout,
};
use crate::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, ViewRegistry};

use super::editor_error::EditorError;
use super::editor_session_state::EditorSessionState;

pub(super) fn ensure_builtin_shell_instances(
    registry: &mut ViewRegistry,
    session: &mut EditorSessionState,
) -> Result<(), EditorError> {
    for instance in builtin_shell_view_instances() {
        let restored = if let Some(existing) = registry.instance(&instance.instance_id).cloned() {
            existing
        } else {
            registry
                .restore_instance(instance.clone())
                .map_err(EditorError::Registry)?
        };
        session
            .open_view_instances
            .insert(restored.instance_id.clone(), restored);
    }
    Ok(())
}

fn builtin_shell_view_instances() -> Vec<ViewInstance> {
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

pub(crate) fn builtin_hybrid_layout() -> WorkbenchLayout {
    WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            document_workspace: DocumentNode::Tabs(TabStackLayout {
                tabs: vec![
                    ViewInstanceId::new("editor.scene#1"),
                    ViewInstanceId::new("editor.game#1"),
                ],
                active_tab: Some(ViewInstanceId::new("editor.scene#1")),
            }),
        }],
        drawers: BTreeMap::from([
            (
                ActivityDrawerSlot::LeftTop,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::LeftTop,
                    tab_stack: TabStackLayout {
                        tabs: vec![
                            ViewInstanceId::new("editor.project#1"),
                            ViewInstanceId::new("editor.assets#1"),
                            ViewInstanceId::new("editor.hierarchy#1"),
                        ],
                        active_tab: Some(ViewInstanceId::new("editor.project#1")),
                    },
                    active_view: Some(ViewInstanceId::new("editor.project#1")),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 312.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::LeftBottom,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::LeftBottom,
                    tab_stack: TabStackLayout::default(),
                    active_view: None,
                    mode: ActivityDrawerMode::Collapsed,
                    extent: 288.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::RightTop,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::RightTop,
                    tab_stack: TabStackLayout {
                        tabs: vec![ViewInstanceId::new("editor.inspector#1")],
                        active_tab: Some(ViewInstanceId::new("editor.inspector#1")),
                    },
                    active_view: Some(ViewInstanceId::new("editor.inspector#1")),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 308.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::RightBottom,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::RightBottom,
                    tab_stack: TabStackLayout::default(),
                    active_view: None,
                    mode: ActivityDrawerMode::Collapsed,
                    extent: 288.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::BottomLeft,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::BottomLeft,
                    tab_stack: TabStackLayout {
                        tabs: vec![ViewInstanceId::new("editor.console#1")],
                        active_tab: Some(ViewInstanceId::new("editor.console#1")),
                    },
                    active_view: Some(ViewInstanceId::new("editor.console#1")),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 164.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::BottomRight,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::BottomRight,
                    tab_stack: TabStackLayout::default(),
                    active_view: None,
                    mode: ActivityDrawerMode::Collapsed,
                    extent: 224.0,
                    visible: true,
                },
            ),
        ]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}
