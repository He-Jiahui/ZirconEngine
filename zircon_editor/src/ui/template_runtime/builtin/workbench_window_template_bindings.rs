use std::collections::BTreeMap;

use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{GridMode, TransformHandleKind};
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
        EditorUiBindingPayload::menu_action("workbench.menu.main.open"),
    );
    insert_click(
        &mut bindings,
        "MenuAction",
        "NewAsset",
        EditorUiBindingPayload::menu_action("workbench.asset.create"),
    );
    insert_click(
        &mut bindings,
        "MenuAction",
        "OpenProject",
        EditorUiBindingPayload::menu_action("workbench.project.open"),
    );
    insert_click(
        &mut bindings,
        "MenuAction",
        "SaveProject",
        EditorUiBindingPayload::menu_action("workbench.project.save"),
    );
    insert_submit(
        &mut bindings,
        "CommandPalette",
        "Commit",
        EditorUiBindingPayload::editor_command("editor.command.palette"),
    );
    insert_change(
        &mut bindings,
        "CommandPalette",
        "QueryChanged",
        EditorUiBindingPayload::menu_action("editor.command_palette.query_changed"),
    );
    insert_change(
        &mut bindings,
        "CommandPalette",
        "WindowRequested",
        EditorUiBindingPayload::menu_action("editor.command_palette.window_requested"),
    );

    insert_click(
        &mut bindings,
        "Tool",
        "Select",
        EditorUiBindingPayload::viewport_command(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Select,
        )),
    );
    insert_click(
        &mut bindings,
        "Tool",
        "Move",
        EditorUiBindingPayload::viewport_command(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Move),
        )),
    );
    insert_click(
        &mut bindings,
        "Tool",
        "Rotate",
        EditorUiBindingPayload::viewport_command(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Rotate),
        )),
    );
    insert_click(
        &mut bindings,
        "Tool",
        "Scale",
        EditorUiBindingPayload::viewport_command(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Scale),
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
        EditorUiBindingPayload::menu_action("workbench.play_mode.enter"),
    );
    insert_click(
        &mut bindings,
        "Run",
        "OpenModeMenu",
        EditorUiBindingPayload::menu_action("workbench.run_mode.menu.open"),
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
        EditorUiBindingPayload::menu_action("workbench.layout.menu.open"),
    );

    insert_click(
        &mut bindings,
        "Hierarchy",
        "OpenFilter",
        EditorUiBindingPayload::menu_action("workbench.hierarchy.filter.open"),
    );
    insert_click(
        &mut bindings,
        "Hierarchy",
        "AddEntity",
        EditorUiBindingPayload::menu_action("workbench.scene.node.create.cube"),
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
        EditorUiBindingPayload::menu_action("workbench.inspector.component.add"),
    );
    insert_inspector_transform_axis_bindings(&mut bindings);
    insert_click(
        &mut bindings,
        "PanelTab",
        "SceneTreeScene",
        EditorUiBindingPayload::menu_action("scene_tree.scene_tab.select"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "SceneTreeLayers",
        EditorUiBindingPayload::menu_action("scene_tree.layers_tab.select"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "InspectorMain",
        EditorUiBindingPayload::menu_action("inspector.main_tab.select"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "InspectorHistory",
        EditorUiBindingPayload::menu_action("inspector.history_tab.select"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "ComponentDrawerComponents",
        EditorUiBindingPayload::menu_action("component_drawer.components_tab.select"),
    );
    insert_click(
        &mut bindings,
        "PanelTab",
        "ComponentDrawerConsole",
        EditorUiBindingPayload::menu_action("component_drawer.console_tab.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Primary",
        EditorUiBindingPayload::menu_action("component_lab.button.primary"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Secondary",
        EditorUiBindingPayload::menu_action("component_lab.button.secondary"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Tertiary",
        EditorUiBindingPayload::menu_action("component_lab.button.tertiary"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "Outline",
        EditorUiBindingPayload::menu_action("component_lab.button.outline"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "IconButton",
        EditorUiBindingPayload::menu_action("component_lab.button.icon"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "DeleteButton",
        EditorUiBindingPayload::menu_action("component_lab.button.delete"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "ButtonDropdownOpen",
        EditorUiBindingPayload::menu_action("component_lab.button_dropdown.open"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "ButtonDropdownSelect",
        EditorUiBindingPayload::menu_action("component_lab.button_dropdown.select"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputTextEdit",
        EditorUiBindingPayload::menu_action("component_lab.input_text.edit"),
    );
    insert_submit(
        &mut bindings,
        "ComponentLab",
        "InputTextCommit",
        EditorUiBindingPayload::menu_action("component_lab.input_text.commit"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputFocusedEdit",
        EditorUiBindingPayload::menu_action("component_lab.input_focused.edit"),
    );
    insert_submit(
        &mut bindings,
        "ComponentLab",
        "InputFocusedCommit",
        EditorUiBindingPayload::menu_action("component_lab.input_focused.commit"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputSearchEdit",
        EditorUiBindingPayload::menu_action("component_lab.input_search.edit"),
    );
    insert_submit(
        &mut bindings,
        "ComponentLab",
        "InputSearchCommit",
        EditorUiBindingPayload::menu_action("component_lab.input_search.commit"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "InputDropdownOpen",
        EditorUiBindingPayload::menu_action("component_lab.input_dropdown.open"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputDropdownSelect",
        EditorUiBindingPayload::menu_action("component_lab.input_dropdown.select"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "InputSegmentedSelect",
        EditorUiBindingPayload::menu_action("component_lab.input_segment.select"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "IconToggleSegmentedSelect",
        EditorUiBindingPayload::menu_action("component_lab.icon_toggle_segment.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "LabsTabOne",
        EditorUiBindingPayload::menu_action("component_lab.labs_tab_one.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "LabsTabTwo",
        EditorUiBindingPayload::menu_action("component_lab.labs_tab_two.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "LabsTabThree",
        EditorUiBindingPayload::menu_action("component_lab.labs_tab_three.select"),
    );
    insert_toggle(
        &mut bindings,
        "ComponentLab",
        "CheckboxOnToggle",
        EditorUiBindingPayload::menu_action("component_lab.checkbox_on.toggle"),
    );
    insert_toggle(
        &mut bindings,
        "ComponentLab",
        "CheckboxOffToggle",
        EditorUiBindingPayload::menu_action("component_lab.checkbox_off.toggle"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "RadioOnChange",
        EditorUiBindingPayload::menu_action("component_lab.radio_on.select"),
    );
    insert_change(
        &mut bindings,
        "ComponentLab",
        "RadioOffChange",
        EditorUiBindingPayload::menu_action("component_lab.radio_off.select"),
    );
    insert_toggle(
        &mut bindings,
        "ComponentLab",
        "ToggleSwitch",
        EditorUiBindingPayload::menu_action("component_lab.switch.toggle"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "ListItemSelect",
        EditorUiBindingPayload::menu_action("component_lab.list_item.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "ListSelectedSelect",
        EditorUiBindingPayload::menu_action("component_lab.list_selected.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "TableItemSelect",
        EditorUiBindingPayload::menu_action("component_lab.table_item.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "TableSelectedSelect",
        EditorUiBindingPayload::menu_action("component_lab.table_selected.select"),
    );
    insert_click(
        &mut bindings,
        "ComponentLab",
        "TableTailSelect",
        EditorUiBindingPayload::menu_action("component_lab.table_tail.select"),
    );
    insert_inspector_component_property_bindings(&mut bindings);
    super::workbench_module_template_bindings::insert_workbench_module_bindings(&mut bindings);
    super::workbench_extension_module_template_bindings::insert_workbench_extension_module_bindings(
        &mut bindings,
    );
    super::workbench_generated_bottom_template_bindings::insert_workbench_generated_bottom_bindings(
        &mut bindings,
    );
    insert_click(
        &mut bindings,
        "Workbench",
        "ToggleTheme",
        EditorUiBindingPayload::menu_action("workbench.theme.toggle"),
    );

    bindings
}

fn insert_inspector_transform_axis_bindings(bindings: &mut BTreeMap<String, EditorUiBinding>) {
    for group in ["Position", "Rotation", "Scale"] {
        for axis in ["X", "Y", "Z"] {
            let edit_control_id = format!("Transform{group}{axis}Edit");
            let edit_action = format!(
                "inspector.transform.{}_{}.edit",
                group.to_ascii_lowercase(),
                axis.to_ascii_lowercase()
            );
            insert_change(
                bindings,
                "Inspector",
                &edit_control_id,
                EditorUiBindingPayload::menu_action(edit_action),
            );

            let commit_control_id = format!("Transform{group}{axis}Commit");
            let commit_action = format!(
                "inspector.transform.{}_{}.commit",
                group.to_ascii_lowercase(),
                axis.to_ascii_lowercase()
            );
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
        let edit_action = format!("inspector.component_property_{index:02}.edit");
        insert_change(
            bindings,
            "Inspector",
            &edit_control_id,
            EditorUiBindingPayload::menu_action(edit_action),
        );

        let commit_control_id = format!("{slot}Commit");
        let commit_action = format!("inspector.component_property_{index:02}.commit");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_lab_search_input_has_edit_and_commit_bindings() {
        let bindings = workbench_window_template_bindings();

        assert_menu_binding(
            &bindings,
            "ComponentLab/InputSearchEdit",
            EditorUiEventKind::Change,
            "component_lab.input_search.edit",
        );
        assert_menu_binding(
            &bindings,
            "ComponentLab/InputSearchCommit",
            EditorUiEventKind::Submit,
            "component_lab.input_search.commit",
        );
    }

    #[test]
    fn workbench_command_palette_commit_binding_is_registered() {
        let bindings = workbench_window_template_bindings();
        let binding = bindings
            .get("CommandPalette/Commit")
            .expect("command palette commit binding should be registered");

        assert_eq!(binding.path().event_kind, EditorUiEventKind::Submit);
        assert_eq!(
            binding.payload(),
            &EditorUiBindingPayload::editor_command("editor.command.palette")
        );

        let query_binding = bindings
            .get("CommandPalette/QueryChanged")
            .expect("command palette query binding should be registered");
        assert_eq!(query_binding.path().event_kind, EditorUiEventKind::Change);
        assert_eq!(
            query_binding.payload(),
            &EditorUiBindingPayload::menu_action("editor.command_palette.query_changed")
        );

        let window_binding = bindings
            .get("CommandPalette/WindowRequested")
            .expect("command palette window request binding should be registered");
        assert_eq!(window_binding.path().event_kind, EditorUiEventKind::Change);
        assert_eq!(
            window_binding.payload(),
            &EditorUiBindingPayload::menu_action("editor.command_palette.window_requested")
        );
    }

    fn assert_menu_binding(
        bindings: &BTreeMap<String, EditorUiBinding>,
        binding_id: &str,
        event_kind: EditorUiEventKind,
        action_id: &str,
    ) {
        let binding = bindings
            .get(binding_id)
            .unwrap_or_else(|| panic!("{binding_id} should be registered"));

        assert_eq!(binding.path().event_kind, event_kind);
        assert_eq!(
            binding.payload(),
            &EditorUiBindingPayload::menu_action(action_id)
        );
    }
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
