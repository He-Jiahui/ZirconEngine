//! Process startup argument parsing and typed editor launch routing.

mod diagnostic_log_args;
#[cfg(feature = "target-editor-host")]
mod launch_args;

pub(crate) use diagnostic_log_args::parse_diagnostic_log_startup_args;
#[cfg(feature = "target-editor-host")]
pub(crate) use launch_args::{
    editor_startup_argument_error, EditorGuiStartupRequestArgs, EditorLaunchArgs, EditorLaunchRoute,
};
