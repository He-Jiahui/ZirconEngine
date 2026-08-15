use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_inspector_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.inspector_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_reference_pointer_event(
        move |kind, button, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.inspector_reference_pointer_event(kind, button, x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_control_changed(
        move |control_id: SharedString, value: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_inspector_control_changed(control_id.as_str(), value.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_control_clicked(move |control_id: SharedString| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_inspector_control_clicked(control_id.as_str());
        });
    });
}
