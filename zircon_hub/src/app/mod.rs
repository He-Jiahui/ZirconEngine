mod binding;
mod localization;
mod quick_action;
mod runtime;
mod view_model;

slint::include_modules!();

pub fn run() -> Result<(), crate::HubError> {
    runtime::run()
}
