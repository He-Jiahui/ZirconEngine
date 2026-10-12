use super::*;

pub(super) fn wire_host_shell_menu_callbacks(
    host_shell: &UiHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    host_shell.on_menu_pointer_clicked(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_clicked(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_menu_pointer_moved(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_moved(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_menu_pointer_scrolled(move |x, y, delta| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_scrolled(x, y, delta);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_activity_rail_pointer_clicked(move |side, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .activity_rail_pointer_clicked(side.as_str(), x, y);
        }
    });
}
