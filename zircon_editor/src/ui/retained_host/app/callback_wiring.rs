use super::*;
use crate::ui::retained_host::UiHostContext;

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
    let host_shell = ui.global::<UiHostContext>();
    let weak = Rc::downgrade(host);
    host_shell.on_frame_requested(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().tick();
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_unhandled_keyboard_input(move |keyboard| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .dispatch_unhandled_native_keyboard_input(keyboard);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_close_prompt_action_clicked(move |action_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .close_prompt_action_clicked(action_id.as_str());
        }
    });

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
    host_shell.on_floating_window_header_pointer_clicked(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .floating_window_header_pointer_clicked(x, y);
        }
    });

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

    pane_surface::wire_pane_surface_callbacks(ui, host);
}
