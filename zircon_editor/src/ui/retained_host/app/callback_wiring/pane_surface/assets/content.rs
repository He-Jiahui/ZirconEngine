use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_asset_content_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_clicked(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_clicked(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_event(
        move |surface_mode: SharedString, kind, button, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_event(
                    surface_mode.as_str(),
                    kind,
                    button,
                    x,
                    y,
                    width,
                    height,
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_moved(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_moved(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_scrolled(
        move |surface_mode: SharedString, x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_scrolled(
                    surface_mode.as_str(),
                    x,
                    y,
                    delta,
                    width,
                    height,
                );
            });
        },
    );
}
