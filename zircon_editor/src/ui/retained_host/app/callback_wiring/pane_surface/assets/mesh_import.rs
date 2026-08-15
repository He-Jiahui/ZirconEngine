use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_mesh_import_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    pane_surface_host.on_mesh_import_path_edited(move |value: SharedString| {
        if let Some(host) = weak.upgrade() {
            let mut host = host.borrow_mut();
            let result =
                callback_dispatch::dispatch_mesh_import_path_edit(&host.runtime, value.to_string());
            host.apply_dispatch_result(result);
        }
    });
}
