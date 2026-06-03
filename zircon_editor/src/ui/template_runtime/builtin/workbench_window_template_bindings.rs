use std::collections::BTreeMap;

use crate::scene::viewport::{GridMode, SceneViewportTool};
use crate::ui::binding::{
    DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, SelectionCommand,
    ViewportCommand,
};

pub(super) fn workbench_window_template_bindings() -> BTreeMap<String, EditorUiBinding> {
    let mut bindings = BTreeMap::new();
    insert_click(
        &mut bindings,
        "MenuAction",
        "OpenMainMenu",
        EditorUiBindingPayload::menu_action("OpenMainMenu"),
    );
    insert_click(
        &mut bindings,
        "MenuAction",
        "NewAsset",
        EditorUiBindingPayload::menu_action("NewAsset"),
    );
    insert_click(
        &mut bindings,
        "MenuAction",
        "OpenProject",
        EditorUiBindingPayload::menu_action("OpenProject"),
    );
    insert_click(
        &mut bindings,
        "MenuAction",
        "SaveProject",
        EditorUiBindingPayload::menu_action("SaveProject"),
    );

    insert_click(
        &mut bindings,
        "Tool",
        "Select",
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(SceneViewportTool::Drag)),
    );
    insert_click(
        &mut bindings,
        "Tool",
        "Move",
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(SceneViewportTool::Move)),
    );
    insert_click(
        &mut bindings,
        "Tool",
        "Rotate",
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(
            SceneViewportTool::Rotate,
        )),
    );
    insert_click(
        &mut bindings,
        "Tool",
        "Scale",
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(
            SceneViewportTool::Scale,
        )),
    );
    insert_click(
        &mut bindings,
        "Tool",
        "ToggleSnap",
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetGridMode(
            GridMode::VisibleAndSnap,
        )),
    );

    insert_click(
        &mut bindings,
        "Run",
        "Play",
        EditorUiBindingPayload::menu_action("EnterPlayMode"),
    );
    insert_click(
        &mut bindings,
        "Run",
        "OpenModeMenu",
        EditorUiBindingPayload::menu_action("OpenRunModeMenu"),
    );

    insert_click(
        &mut bindings,
        "DockCommand",
        "ActivateScene",
        EditorUiBindingPayload::dock_command(DockCommand::FocusView {
            instance_id: "editor.scene#1".to_string(),
        }),
    );
    insert_click(
        &mut bindings,
        "DockCommand",
        "ActivateHierarchy",
        EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
            slot: "left_top".to_string(),
            instance_id: "editor.hierarchy#1".to_string(),
        }),
    );
    insert_click(
        &mut bindings,
        "DockCommand",
        "ActivateGraph",
        EditorUiBindingPayload::dock_command(DockCommand::FocusView {
            instance_id: "editor.graph#1".to_string(),
        }),
    );
    insert_click(
        &mut bindings,
        "DockCommand",
        "ActivateAssets",
        EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
            slot: "left_top".to_string(),
            instance_id: "editor.assets#1".to_string(),
        }),
    );
    insert_click(
        &mut bindings,
        "DockCommand",
        "ActivateAudio",
        EditorUiBindingPayload::dock_command(DockCommand::FocusView {
            instance_id: "editor.audio#1".to_string(),
        }),
    );
    insert_click(
        &mut bindings,
        "DockCommand",
        "ActivateCode",
        EditorUiBindingPayload::dock_command(DockCommand::FocusView {
            instance_id: "editor.code#1".to_string(),
        }),
    );
    insert_click(
        &mut bindings,
        "DockCommand",
        "OpenLayoutMenu",
        EditorUiBindingPayload::menu_action("OpenLayoutMenu"),
    );

    insert_click(
        &mut bindings,
        "Hierarchy",
        "OpenFilter",
        EditorUiBindingPayload::menu_action("OpenHierarchyFilter"),
    );
    insert_click(
        &mut bindings,
        "Hierarchy",
        "AddEntity",
        EditorUiBindingPayload::menu_action("CreateNode.Cube"),
    );
    for control_id in [
        "SelectEntity",
        "SelectRoot",
        "SelectEnvironment",
        "SelectLevel",
        "SelectProps",
        "SelectPlayer",
        "SelectAudio",
        "SelectSlot07",
        "SelectSlot08",
        "SelectSlot09",
        "SelectSlot10",
    ] {
        insert_click(
            &mut bindings,
            "Hierarchy",
            control_id,
            EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
                node_id: 0,
            }),
        );
    }

    insert_click(
        &mut bindings,
        "Inspector",
        "AddComponent",
        EditorUiBindingPayload::menu_action("AddComponent"),
    );
    insert_inspector_transform_axis_bindings(&mut bindings);
    insert_click(
        &mut bindings,
        "PanelTab",
        "SceneTreeScene",
        EditorUiBindingPayload::menu_action("SelectSceneTreeSceneTab"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "SceneTreeLayers",
        EditorUiBindingPayload::menu_action("SelectSceneTreeLayersTab"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "InspectorMain",
        EditorUiBindingPayload::menu_action("SelectInspectorMainTab"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "InspectorHistory",
        EditorUiBindingPayload::menu_action("SelectInspectorHistoryTab"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "ComponentDrawerComponents",
        EditorUiBindingPayload::menu_action("SelectComponentDrawerComponentsTab"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "ComponentDrawerConsole",
        EditorUiBindingPayload::menu_action("SelectComponentDrawerConsoleTab"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Primary",
        EditorUiBindingPayload::editor_operation("ComponentLab.Primary"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Secondary",
        EditorUiBindingPayload::editor_operation("ComponentLab.Secondary"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Tertiary",
        EditorUiBindingPayload::editor_operation("ComponentLab.Tertiary"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Outline",
        EditorUiBindingPayload::editor_operation("ComponentLab.Outline"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "IconButton",
        EditorUiBindingPayload::editor_operation("ComponentLab.IconButton"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "DeleteButton",
        EditorUiBindingPayload::editor_operation("ComponentLab.DeleteButton"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "ButtonDropdownOpen",
        EditorUiBindingPayload::menu_action("OpenComponentLabButtonDropdown"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "ButtonDropdownSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabButtonDropdownOption"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputTextEdit",
        EditorUiBindingPayload::menu_action("ComponentLab.ValueChanged.InputText"),
    );
    insert_submit(
        &mut bindings,
        "ComponentLab",
        "InputTextCommit",
        EditorUiBindingPayload::menu_action("ComponentLab.Commit.InputText"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputFocusedEdit",
        EditorUiBindingPayload::menu_action("ComponentLab.ValueChanged.InputFocused"),
    );
    insert_submit(
        &mut bindings,
        "ComponentLab",
        "InputFocusedCommit",
        EditorUiBindingPayload::menu_action("ComponentLab.Commit.InputFocused"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "InputDropdownOpen",
        EditorUiBindingPayload::menu_action("OpenComponentLabInputDropdown"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputDropdownSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabInputDropdownOption"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputSegmentedSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabInputSegment"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "IconToggleSegmentedSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabIconToggleSegment"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "LabsTabOne",
        EditorUiBindingPayload::menu_action("SelectComponentLabLabsTabOne"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "LabsTabTwo",
        EditorUiBindingPayload::menu_action("SelectComponentLabLabsTabTwo"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "LabsTabThree",
        EditorUiBindingPayload::menu_action("SelectComponentLabLabsTabThree"),
    );
    insert_toggle(
        &mut bindings,
        "ComponentLab",
        "CheckboxOnToggle",
        EditorUiBindingPayload::menu_action("ToggleComponentLabCheckboxOn"),
    );
    insert_toggle(
        &mut bindings,
        "ComponentLab",
        "CheckboxOffToggle",
        EditorUiBindingPayload::menu_action("ToggleComponentLabCheckboxOff"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "RadioOnChange",
        EditorUiBindingPayload::menu_action("SelectComponentLabRadioOn"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "RadioOffChange",
        EditorUiBindingPayload::menu_action("SelectComponentLabRadioOff"),
    );
    insert_toggle(
        &mut bindings,
        "ComponentLab",
        "ToggleSwitch",
        EditorUiBindingPayload::menu_action("ToggleComponentLabSwitch"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "ListItemSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabListItem"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "ListSelectedSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabListSelected"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "TableItemSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabTableItem"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "TableSelectedSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabTableSelected"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "TableTailSelect",
        EditorUiBindingPayload::menu_action("SelectComponentLabTableTail"),
    );
    insert_inspector_component_property_bindings(&mut bindings);
    super::workbench_module_template_bindings::insert_workbench_module_bindings(&mut bindings);
    insert_click(
        &mut bindings,
        "Workbench",
        "ToggleTheme",
        EditorUiBindingPayload::menu_action("ToggleTheme"),
    );

    bindings
}

fn insert_inspector_transform_axis_bindings(bindings: &mut BTreeMap<String, EditorUiBinding>) {
    for group in ["Position", "Rotation", "Scale"] {
        for axis in ["X", "Y", "Z"] {
            let edit_control_id = format!("Transform{group}{axis}Edit");
            let edit_action = format!("Inspector.Transform.{group}{axis}.Edit");
            insert_change(
                bindings,
                "Inspector",
                &edit_control_id,
                EditorUiBindingPayload::menu_action(edit_action),
            );

            let commit_control_id = format!("Transform{group}{axis}Commit");
            let commit_action = format!("Inspector.Transform.{group}{axis}.Commit");
            insert_submit(
                bindings,
                "Inspector",
                &commit_control_id,
                EditorUiBindingPayload::menu_action(commit_action),
            );
        }
    }
}

fn insert_inspector_component_property_bindings(bindings: &mut BTreeMap<String, EditorUiBinding>) {
    for index in 1..=4 {
        let slot = format!("ComponentProperty{index:02}");
        let edit_control_id = format!("{slot}Edit");
        let edit_action = format!("Inspector.{slot}.Edit");
        insert_change(
            bindings,
            "Inspector",
            &edit_control_id,
            EditorUiBindingPayload::menu_action(edit_action),
        );

        let commit_control_id = format!("{slot}Commit");
        let commit_action = format!("Inspector.{slot}.Commit");
        insert_submit(
            bindings,
            "Inspector",
            &commit_control_id,
            EditorUiBindingPayload::menu_action(commit_action),
        );
    }
}

fn insert_change(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    payload: EditorUiBindingPayload,
) {
    insert_event(
        bindings,
        view_id,
        control_id,
        EditorUiEventKind::Change,
        payload,
    );
}

fn insert_click(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    payload: EditorUiBindingPayload,
) {
    insert_event(
        bindings,
        view_id,
        control_id,
        EditorUiEventKind::Click,
        payload,
    );
}

fn insert_submit(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    payload: EditorUiBindingPayload,
) {
    insert_event(
        bindings,
        view_id,
        control_id,
        EditorUiEventKind::Submit,
        payload,
    );
}

fn insert_toggle(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    payload: EditorUiBindingPayload,
) {
    insert_event(
        bindings,
        view_id,
        control_id,
        EditorUiEventKind::Toggle,
        payload,
    );
}

fn insert_event(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    event_kind: EditorUiEventKind,
    payload: EditorUiBindingPayload,
) {
    bindings.insert(
        format!("{view_id}/{control_id}"),
        EditorUiBinding::new(view_id, control_id, event_kind, payload),
    );
}
