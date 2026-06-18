use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_viewport_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_viewport_pointer_event(move |kind, button, x, y, delta| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.viewport_pointer_event(kind, button, x, y, delta);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_viewport_toolbar_pointer_clicked(
        move |surface_key: SharedString,
              control_id: SharedString,
              control_x,
              control_y,
              control_width,
              control_height,
              point_x,
              point_y| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.viewport_toolbar_pointer_clicked(
                    surface_key.as_str(),
                    control_id.as_str(),
                    control_x,
                    control_y,
                    control_width,
                    control_height,
                    point_x,
                    point_y,
                );
            });
        },
    );
}
