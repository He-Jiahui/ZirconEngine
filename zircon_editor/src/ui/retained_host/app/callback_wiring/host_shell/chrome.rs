use super::*;

pub(super) fn wire_host_shell_chrome_callbacks(
    host_shell: &UiHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    host_shell.on_host_page_pointer_clicked(
        move |tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut()
                    .host_page_pointer_clicked(tab_index, tab_x, tab_width, point_x, point_y);
            }
        },
    );

    let weak = Rc::downgrade(host);
    host_shell.on_document_tab_pointer_clicked(
        move |surface_key, tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().document_tab_pointer_clicked(
                    surface_key.as_str(),
                    tab_index,
                    tab_x,
                    tab_width,
                    point_x,
                    point_y,
                );
            }
        },
    );

    let weak = Rc::downgrade(host);
    host_shell.on_document_tab_close_pointer_clicked(
        move |surface_key, tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().document_tab_close_pointer_clicked(
                    surface_key.as_str(),
                    tab_index,
                    tab_x,
                    tab_width,
                    point_x,
                    point_y,
                );
            }
        },
    );

    let weak = Rc::downgrade(host);
    host_shell.on_drawer_header_pointer_clicked(
        move |surface_key, tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().drawer_header_pointer_clicked(
                    surface_key.as_str(),
                    tab_index,
                    tab_x,
                    tab_width,
                    point_x,
                    point_y,
                );
            }
        },
    );
}
