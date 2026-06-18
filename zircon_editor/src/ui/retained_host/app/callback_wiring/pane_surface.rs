use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

mod assets;
mod component_showcase;
mod ui_asset;
mod viewport;

pub(super) fn wire_pane_surface_callbacks(
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let pane_surface_host = ui.global::<PaneSurfaceHostContext>();

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

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_console_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.console_pointer_scrolled(x, y, delta, width, height);
        });
    });

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

    component_showcase::wire_component_showcase_callbacks(&pane_surface_host, ui, host);
    assets::wire_asset_callbacks(&pane_surface_host, ui, host);

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

    viewport::wire_viewport_callbacks(&pane_surface_host, ui, host);
    ui_asset::wire_ui_asset_callbacks(&pane_surface_host, ui, host);
}
