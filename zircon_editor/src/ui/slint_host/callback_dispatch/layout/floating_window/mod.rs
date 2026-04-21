mod dispatch;
mod resolution;
#[cfg(test)]
mod tests;

pub(crate) use dispatch::{
    dispatch_builtin_floating_window_focus, dispatch_builtin_floating_window_focus_for_source,
};
pub(crate) use resolution::resolve_builtin_floating_window_close_instances;
