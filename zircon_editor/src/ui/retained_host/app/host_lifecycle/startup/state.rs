mod construction;
mod interaction;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) use construction::{
    StartupHostConstruction, construct_startup_host,
};
