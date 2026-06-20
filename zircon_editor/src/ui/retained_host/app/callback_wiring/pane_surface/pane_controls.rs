use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_pane_control_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_surface_control_clicked(
        move |control_id: SharedString, action_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_pane_surface_control_clicked(control_id.as_str(), action_id.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_workbench_context_menu_requested(move |request| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_workbench_context_menu_requested(request);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_surface_control_edited(
        move |control_id: SharedString, binding_id: SharedString, value: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_pane_surface_control_edited(
                    control_id.as_str(),
                    binding_id.as_str(),
                    value.as_str(),
                );
            });
        },
    );
}
