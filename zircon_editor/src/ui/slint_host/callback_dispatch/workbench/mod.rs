mod control;
mod menu_action;

#[cfg(test)]
pub(crate) use control::{dispatch_builtin_host_control, dispatch_builtin_host_menu_action};
#[cfg(test)]
pub(crate) use menu_action::slint_menu_action;
pub(crate) use menu_action::{
    dispatch_host_menu_action_with_template_fallback, dispatch_menu_action,
};
