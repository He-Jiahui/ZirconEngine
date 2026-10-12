use super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_asset_detail_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_browser_asset_details_pointer_scrolled(
        move |x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.browser_asset_details_pointer_scrolled(x, y, delta, width, height);
            });
        },
    );
}
