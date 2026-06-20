use super::super::super::super::*;

pub(super) fn resolve_startup_shell_size(ui: &UiHostWindow) -> ShellSizePx {
    let bootstrap = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_read_window_bootstrap");
        ui.get_host_window_bootstrap()
    };
    ShellSizePx::new(
        bootstrap.shell_frame.width.max(1.0),
        bootstrap.shell_frame.height.max(1.0),
    )
}
