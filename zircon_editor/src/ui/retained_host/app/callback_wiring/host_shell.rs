mod chrome;
mod drag_resize;
mod menu;
mod runtime;
mod settings;

use super::*;
use crate::ui::retained_host::UiHostContext;

pub(super) fn wire_host_shell_callbacks(ui: &UiHostWindow, host: &Rc<RefCell<RetainedEditorHost>>) {
    let host_shell = ui.global::<UiHostContext>();
    runtime::wire_host_shell_runtime_callbacks(&host_shell, host);
    menu::wire_host_shell_menu_callbacks(&host_shell, host);
    settings::wire_host_shell_settings_callbacks(&host_shell, host);
    chrome::wire_host_shell_chrome_callbacks(&host_shell, host);
    drag_resize::wire_host_shell_drag_resize_callbacks(&host_shell, host);
}
