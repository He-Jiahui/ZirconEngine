mod specs;

pub(super) use specs::EXTENSION_MODULE_WORKSPACE_CONTROLS;
use specs::{ActionControl, ExtensionNavigationSpec, EXTENSION_MODULE_NAVIGATION_SPECS};

const EMPTY_CONTROLS: &[&str] = &[];

pub(super) fn is_workbench_extension_action(action_id: &str) -> bool {
    workbench_extension_workspace_control_id(action_id).is_some()
        || workbench_extension_panel_tab_control_id(action_id).is_some()
        || workbench_extension_panel_row_control_id(action_id).is_some()
        || workbench_extension_panel_command_control_id(action_id).is_some()
        || workbench_extension_panel_field_action(action_id)
}

pub(super) fn workbench_extension_workspace_control_id(action_id: &str) -> Option<&'static str> {
    EXTENSION_MODULE_NAVIGATION_SPECS
        .iter()
        .find_map(|spec| (spec.open_action_id == action_id).then_some(spec.workspace_control_id))
}

pub(super) fn workbench_extension_panel_tab_control_id(action_id: &str) -> Option<&'static str> {
    EXTENSION_MODULE_NAVIGATION_SPECS
        .iter()
        .find_map(|spec| control_id_for_action(spec.tab_actions, action_id))
}

pub(super) fn workbench_extension_panel_tab_group(action_id: &str) -> &'static [&'static str] {
    group_for_action(action_id, |spec| spec.tab_actions, |spec| spec.tab_controls)
}

pub(super) fn workbench_extension_panel_row_control_id(action_id: &str) -> Option<&'static str> {
    EXTENSION_MODULE_NAVIGATION_SPECS
        .iter()
        .find_map(|spec| control_id_for_action(spec.row_actions, action_id))
}

pub(super) fn workbench_extension_panel_row_group(action_id: &str) -> &'static [&'static str] {
    group_for_action(action_id, |spec| spec.row_actions, |spec| spec.row_controls)
}

pub(super) fn workbench_extension_panel_command_control_id(
    action_id: &str,
) -> Option<&'static str> {
    EXTENSION_MODULE_NAVIGATION_SPECS
        .iter()
        .find_map(|spec| control_id_for_action(spec.command_actions, action_id))
}

pub(super) fn workbench_extension_panel_command_group(action_id: &str) -> &'static [&'static str] {
    group_for_action(
        action_id,
        |spec| spec.command_actions,
        |spec| spec.command_controls,
    )
}

pub(super) fn workbench_extension_panel_field_action(action_id: &str) -> bool {
    EXTENSION_MODULE_NAVIGATION_SPECS
        .iter()
        .any(|spec| spec.field_actions.contains(&action_id))
}

fn control_id_for_action(
    action_controls: &'static [ActionControl],
    action_id: &str,
) -> Option<&'static str> {
    action_controls
        .iter()
        .find_map(|action| (action.action_id == action_id).then_some(action.control_id))
}

fn group_for_action(
    action_id: &str,
    action_controls: fn(&'static ExtensionNavigationSpec) -> &'static [ActionControl],
    group_controls: fn(&'static ExtensionNavigationSpec) -> &'static [&'static str],
) -> &'static [&'static str] {
    EXTENSION_MODULE_NAVIGATION_SPECS
        .iter()
        .find_map(|spec| {
            action_controls(spec)
                .iter()
                .any(|action| action.action_id == action_id)
                .then_some(group_controls(spec))
        })
        .unwrap_or(EMPTY_CONTROLS)
}
