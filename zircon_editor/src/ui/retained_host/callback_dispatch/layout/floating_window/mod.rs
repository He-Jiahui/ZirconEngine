mod dispatch;
#[cfg(test)]
#[path = "tests/resolution.rs"]
mod resolution;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use dispatch::{
    dispatch_builtin_floating_window_focus, dispatch_builtin_floating_window_focus_for_source,
};
