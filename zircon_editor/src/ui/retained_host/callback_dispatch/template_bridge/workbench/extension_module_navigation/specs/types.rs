pub(in super::super) struct ActionControl {
    pub(in super::super) action_id: &'static str,
    pub(in super::super) control_id: &'static str,
}

pub(in super::super) struct ExtensionNavigationSpec {
    pub(in super::super) open_action_id: &'static str,
    pub(in super::super) workspace_control_id: &'static str,
    pub(in super::super) row_controls: &'static [&'static str],
    pub(in super::super) row_actions: &'static [ActionControl],
    pub(in super::super) command_controls: &'static [&'static str],
    pub(in super::super) command_actions: &'static [ActionControl],
    pub(in super::super) field_actions: &'static [&'static str],
}

pub(in super::super) const fn action(
    action_id: &'static str,
    control_id: &'static str,
) -> ActionControl {
    ActionControl {
        action_id,
        control_id,
    }
}

pub(in super::super) const fn spec(
    open_action_id: &'static str,
    workspace_control_id: &'static str,
    row_controls: &'static [&'static str],
    row_actions: &'static [ActionControl],
    command_controls: &'static [&'static str],
    command_actions: &'static [ActionControl],
    field_actions: &'static [&'static str],
) -> ExtensionNavigationSpec {
    ExtensionNavigationSpec {
        open_action_id,
        workspace_control_id,
        row_controls,
        row_actions,
        command_controls,
        command_actions,
        field_actions,
    }
}
