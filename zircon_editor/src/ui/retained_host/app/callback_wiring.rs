use super::*;

mod host_shell;
mod pane_surface;

fn dispatch_with_callback_source(
    weak: &std::rc::Weak<std::cell::RefCell<RetainedEditorHost>>,
    source_ui: &UiHostWindow,
    callback: impl FnOnce(&mut RetainedEditorHost),
) {
    if let Some(host) = weak.upgrade() {
        let source_window_id = resolve_callback_source_window_id(&source_ui);
        host.borrow_mut()
            .with_callback_source_window(source_window_id, callback);
    }
}

pub(super) fn wire_callbacks(ui: &UiHostWindow, host: &Rc<RefCell<RetainedEditorHost>>) {
    host_shell::wire_host_shell_callbacks(ui, host);
    pane_surface::wire_pane_surface_callbacks(ui, host);
}
