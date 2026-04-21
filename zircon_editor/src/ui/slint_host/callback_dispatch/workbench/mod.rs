mod control;
mod menu_action;

#[cfg(test)]
pub(crate) use control::{
    dispatch_builtin_workbench_control, dispatch_builtin_workbench_menu_action,
};
pub(crate) use menu_action::dispatch_workbench_menu_action_with_template_fallback;
#[cfg(test)]
pub(crate) use menu_action::{dispatch_menu_action, slint_menu_action};
