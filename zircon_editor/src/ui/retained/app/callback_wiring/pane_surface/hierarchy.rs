use super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_hierarchy_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_clicked(move |x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_clicked(x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_moved(move |x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_moved(x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_event(move |kind, button, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_event(kind, button, x, y, width, height);
        });
    });
}
