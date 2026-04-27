mod builtin_shell_view_instances;
mod ensure_shell_instances;
mod hybrid_layout;
mod layout_drawers;
mod workbench_page;

pub(super) use ensure_shell_instances::ensure_builtin_shell_instances;
pub(crate) use hybrid_layout::{builtin_hybrid_layout, builtin_hybrid_layout_for_subsystems};
