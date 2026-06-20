use super::*;

pub(super) fn wire_host_shell_drag_resize_callbacks(
    host_shell: &UiHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    host_shell.on_floating_window_header_pointer_clicked(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .floating_window_header_pointer_clicked(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_host_drag_pointer_event(move |kind, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().host_drag_pointer_event(kind, x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_host_resize_pointer_event(move |kind, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().host_resize_pointer_event(kind, x, y);
        }
    });
}
