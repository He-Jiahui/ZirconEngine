use super::*;

pub(super) fn wire_host_shell_settings_callbacks(
    host_shell: &UiHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    host_shell.on_settings_window_scrolled(move |category_scroll_offset, setting_scroll_offset| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .settings_window_scrolled(category_scroll_offset, setting_scroll_offset);
        }
    });
}
