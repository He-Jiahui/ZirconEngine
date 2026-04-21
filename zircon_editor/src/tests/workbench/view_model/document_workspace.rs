use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::{DocumentWorkspaceModel, WorkbenchViewModel};
use crate::ui::workbench::snapshot::{DocumentWorkspaceSnapshot, ViewContentKind};
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn default_preview_fixture_projects_drawers_and_document_workspace() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();

    let model = WorkbenchViewModel::build(&chrome);

    let left_top = model
        .drawer_ring
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert!(left_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Project));
    assert!(left_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Assets));
    assert!(left_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Hierarchy));

    let right_top = model
        .drawer_ring
        .drawers
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert!(right_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Inspector));

    let bottom_left = model
        .drawer_ring
        .drawers
        .get(&ActivityDrawerSlot::BottomLeft)
        .expect("bottom left drawer");
    assert!(bottom_left
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Console));

    match &model.document {
        DocumentWorkspaceModel::Workbench { workspace, .. } => match workspace {
            DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => {
                assert!(tabs
                    .iter()
                    .any(|tab| tab.content_kind == ViewContentKind::Scene));
                assert!(tabs
                    .iter()
                    .any(|tab| tab.content_kind == ViewContentKind::Game));
                assert_eq!(
                    active_tab.as_ref().map(|id| id.0.as_str()),
                    Some("editor.scene#1")
                );
            }
            DocumentWorkspaceSnapshot::Split { .. } => {
                panic!("preview fixture should use tab workspace")
            }
        },
        DocumentWorkspaceModel::Exclusive { .. } => {
            panic!("preview fixture should use workbench page")
        }
    }
}

#[test]
fn default_preview_fixture_exposes_hybrid_shell_tool_windows_and_empty_states() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();

    let model = WorkbenchViewModel::build(&chrome);

    let left_top = model
        .tool_windows
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top tool window");
    assert_eq!(left_top.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        left_top
            .tabs
            .iter()
            .map(|tab| tab.content_kind)
            .collect::<Vec<_>>(),
        vec![
            ViewContentKind::Project,
            ViewContentKind::Assets,
            ViewContentKind::Hierarchy,
        ]
    );
    assert_eq!(
        left_top.active_tab.as_ref().map(|id| id.0.as_str()),
        Some("editor.project#1")
    );

    let project_tab = left_top
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Project)
        .expect("project tab");
    assert!(!project_tab.closeable);
    assert_eq!(
        project_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No project open")
    );
    assert_eq!(
        project_tab
            .empty_state
            .as_ref()
            .and_then(|state| state.primary_action.as_ref())
            .map(|action| action.label.as_str()),
        Some("Open Project")
    );

    let right_top = model
        .tool_windows
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top tool window");
    assert_eq!(right_top.mode, ActivityDrawerMode::Pinned);
    let inspector_tab = right_top
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Inspector)
        .expect("inspector tab");
    assert_eq!(
        inspector_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("Nothing selected")
    );

    let bottom_left = model
        .tool_windows
        .get(&ActivityDrawerSlot::BottomLeft)
        .expect("bottom left tool window");
    let console_tab = bottom_left
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Console)
        .expect("console tab");
    assert_eq!(
        console_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No output yet")
    );

    assert_eq!(
        model
            .document_tabs
            .iter()
            .map(|tab| tab.content_kind)
            .collect::<Vec<_>>(),
        vec![ViewContentKind::Scene, ViewContentKind::Game]
    );
    assert!(model.document_tabs.iter().all(|tab| !tab.closeable));
    assert!(!model
        .document_tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::PrefabEditor));

    let scene_tab = model
        .document_tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Scene)
        .expect("scene tab");
    assert_eq!(
        scene_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No project open")
    );
}

#[test]
fn project_empty_state_remains_the_same_when_docked_to_the_right() {
    let mut fixture = default_preview_fixture();
    let left_top = fixture
        .layout
        .drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    left_top
        .tab_stack
        .tabs
        .retain(|instance_id| instance_id.0 != "editor.project#1");
    left_top.tab_stack.active_tab = Some(ViewInstanceId::new("editor.assets#1"));
    left_top.active_view = Some(ViewInstanceId::new("editor.assets#1"));

    let right_top = fixture
        .layout
        .drawers
        .get_mut(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    right_top
        .tab_stack
        .tabs
        .insert(0, ViewInstanceId::new("editor.project#1"));
    right_top.tab_stack.active_tab = Some(ViewInstanceId::new("editor.project#1"));
    right_top.active_view = Some(ViewInstanceId::new("editor.project#1"));
    right_top.mode = ActivityDrawerMode::Pinned;

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);

    let project_tab = model
        .tool_windows
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top tool window")
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Project)
        .expect("project tab");
    assert_eq!(
        project_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No project open")
    );
}

#[test]
fn scene_empty_state_actions_expose_typed_menu_bindings() {
    let mut fixture = default_preview_fixture();
    fixture.editor.scene_entries.clear();
    fixture.editor.project_open = true;

    let model = WorkbenchViewModel::build(&fixture.build_chrome());
    let scene_tab = model
        .document_tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Scene)
        .expect("scene tab");
    let empty_state = scene_tab.empty_state.as_ref().expect("scene empty state");

    assert_eq!(empty_state.title, "No active scene");
    assert!(matches!(
        empty_state
            .primary_action
            .as_ref()
            .and_then(|action| action.binding.as_ref())
            .map(|binding| binding.payload()),
        Some(EditorUiBindingPayload::MenuAction { action_id }) if action_id == "OpenScene"
    ));
    assert!(matches!(
        empty_state
            .secondary_action
            .as_ref()
            .and_then(|action| action.binding.as_ref())
            .map(|binding| binding.payload()),
        Some(EditorUiBindingPayload::MenuAction { action_id }) if action_id == "CreateScene"
    ));
}
