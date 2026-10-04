use std::{collections::HashMap, sync::LazyLock};

mod specs;

use specs::EXTENSION_MODULE_NAVIGATION_SPECS;
pub(super) use specs::EXTENSION_MODULE_WORKSPACE_CONTROLS;

const EMPTY_CONTROLS: &[&str] = &[];

pub(super) fn is_workbench_extension_action(action_id: &str) -> bool {
    extension_action_index().contains_key(action_id)
}

pub(super) fn workbench_extension_workspace_control_id(action_id: &str) -> Option<&'static str> {
    extension_action_index()
        .get(action_id)
        .and_then(|route| route.workspace_control_id)
}

pub(super) fn workbench_extension_panel_row_control_id(action_id: &str) -> Option<&'static str> {
    extension_action_index()
        .get(action_id)
        .and_then(|route| route.row_control_id)
}

pub(super) fn workbench_extension_panel_row_group(action_id: &str) -> &'static [&'static str] {
    extension_action_index()
        .get(action_id)
        .map(|route| route.row_controls)
        .unwrap_or(EMPTY_CONTROLS)
}

pub(super) fn workbench_extension_panel_command_control_id(
    action_id: &str,
) -> Option<&'static str> {
    extension_action_index()
        .get(action_id)
        .and_then(|route| route.command_control_id)
}

pub(super) fn workbench_extension_panel_field_action(action_id: &str) -> bool {
    extension_action_index()
        .get(action_id)
        .is_some_and(|route| route.field_action)
}

#[derive(Clone, Copy, Default)]
struct ExtensionActionRoute {
    workspace_control_id: Option<&'static str>,
    row_control_id: Option<&'static str>,
    row_controls: &'static [&'static str],
    command_control_id: Option<&'static str>,
    field_action: bool,
}

fn extension_action_index() -> &'static HashMap<&'static str, ExtensionActionRoute> {
    static INDEX: LazyLock<HashMap<&'static str, ExtensionActionRoute>> =
        LazyLock::new(build_extension_action_index);
    &INDEX
}

fn build_extension_action_index() -> HashMap<&'static str, ExtensionActionRoute> {
    let mut index = HashMap::<&'static str, ExtensionActionRoute>::new();
    for spec in EXTENSION_MODULE_NAVIGATION_SPECS {
        let _ = index
            .entry(spec.open_action_id)
            .or_default()
            .workspace_control_id
            .get_or_insert(spec.workspace_control_id);
        for action in spec.row_actions {
            let route = index.entry(action.action_id).or_default();
            let _ = route.row_control_id.get_or_insert(action.control_id);
            if route.row_controls.is_empty() {
                route.row_controls = spec.row_controls;
            }
        }
        for action in spec.command_actions {
            debug_assert!(spec.command_controls.contains(&action.control_id));
            let route = index.entry(action.action_id).or_default();
            let _ = route.command_control_id.get_or_insert(action.control_id);
        }
        for action_id in spec.field_actions {
            index.entry(action_id).or_default().field_action = true;
        }
    }
    index
}

#[cfg(test)]
#[path = "tests/extension_module_navigation_performance_tests.rs"]
mod performance_tests;
