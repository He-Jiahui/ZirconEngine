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
    pane_surface_host.on_scene_viewport_pointer_event(
        move |kind, button, x, y, delta, shift, control| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.scene_viewport_pointer_event(kind, button, x, y, delta, shift, control);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_game_viewport_pointer_event(
        move |kind, button, x, y, delta, shift, control| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.game_viewport_pointer_event(kind, button, x, y, delta, shift, control);
            });
        },
    );

    if ui.same_native_window(&host.borrow().ui) {
        let weak = Rc::downgrade(host);
        ui.on_game_rmb(move |input| {
            let host = weak.upgrade()?;
            if host.borrow().main_window_close_blocks_session_ui()
                || host.borrow().runtime.pending_play_label().is_some()
            {
                return None;
            }
            let mut host = host.borrow_mut();
            host.with_callback_source_window(None, |host| host.route_game_rmb(input))
        });
    }

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_viewport_toolbar_pointer_clicked(
        move |surface_key: SharedString, point_x, point_y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.viewport_toolbar_pointer_clicked(
                    surface_key.as_str(),
                    point_x,
                    point_y,
                    width,
                    height,
                );
            });
        },
    );
}
