use super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn wire_component_showcase_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_activated(
        move |control_id: SharedString, action_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_activated(
                    control_id.as_str(),
                    action_id.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_drag_delta(
        move |control_id: SharedString, action_id: SharedString, delta: f32| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_drag_delta(
                    control_id.as_str(),
                    action_id.as_str(),
                    f64::from(delta),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_edited(
        move |control_id: SharedString, action_id: SharedString, value: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_edited(
                    control_id.as_str(),
                    action_id.as_str(),
                    value.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_context_requested(
        move |control_id: SharedString, action_id: SharedString, x: f32, y: f32| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_context_requested(
                    control_id.as_str(),
                    action_id.as_str(),
                    f64::from(x),
                    f64::from(y),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_option_selected(
        move |control_id: SharedString, action_id: SharedString, option_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_option_selected(
                    control_id.as_str(),
                    action_id.as_str(),
                    option_id.as_str(),
                );
            });
        },
    );
}
