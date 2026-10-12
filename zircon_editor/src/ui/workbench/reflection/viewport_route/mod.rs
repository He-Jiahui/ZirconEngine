#[path = "default_command.rs"]
mod default_command;
#[path = "register.rs"]
mod register;

pub(crate) use register::register_viewport_route;
