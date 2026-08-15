use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_welcome_recent_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_recent_pointer_clicked(move |x, y, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_clicked(x, y, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_recent_pointer_moved(move |x, y, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_moved(x, y, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_recent_pointer_scrolled(move |x, y, delta, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_scrolled(x, y, delta, width, height);
        }
    });
}

pub(super) fn wire_welcome_control_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_control_changed(
        move |control_id: SharedString, value: SharedString| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().dispatch_welcome_surface_control(
                    control_id.as_str(),
                    UiEventKind::Change,
                    vec![UiBindingValue::string(value.as_str())],
                );
            }
        },
    );

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_control_clicked(move |control_id: SharedString| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().dispatch_welcome_surface_control(
                control_id.as_str(),
                UiEventKind::Click,
                Vec::new(),
            );
        }
    });
}
