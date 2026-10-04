use super::*;

mod host_shell;
mod pane_surface;

// 模板回调借用弱宿主引用，并在分发期间保留来源窗口；浮窗与主窗共用处理器时需要这个来源上下文。
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

pub(super) fn wire_callbacks(
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
    resize_source_window_id: Option<&MainPageId>,
) {
    host_shell::wire_host_shell_callbacks(ui, host, resize_source_window_id.cloned());
    pane_surface::wire_pane_surface_callbacks(ui, host);
}
