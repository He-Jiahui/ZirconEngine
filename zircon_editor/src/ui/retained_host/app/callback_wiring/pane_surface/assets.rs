use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_asset_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
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

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_control_changed(
        move |source: SharedString, control_id: SharedString, value: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_asset_control_changed(
                    source.as_str(),
                    control_id.as_str(),
                    value.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_control_clicked(
        move |source: SharedString, control_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_asset_control_clicked(source.as_str(), control_id.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_tree_pointer_clicked(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_tree_pointer_clicked(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_tree_pointer_moved(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_tree_pointer_moved(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_tree_pointer_scrolled(
        move |surface_mode: SharedString, x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_tree_pointer_scrolled(surface_mode.as_str(), x, y, delta, width, height);
            });
        },
    );

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

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_reference_pointer_clicked(
        move |surface_mode: SharedString, list_kind: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_reference_pointer_clicked(
                    surface_mode.as_str(),
                    list_kind.as_str(),
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
    pane_surface_host.on_asset_reference_pointer_event(
        move |surface_mode: SharedString,
              list_kind: SharedString,
              kind,
              button,
              x,
              y,
              width,
              height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_reference_pointer_event(
                    surface_mode.as_str(),
                    list_kind.as_str(),
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
    pane_surface_host.on_asset_reference_pointer_moved(
        move |surface_mode: SharedString, list_kind: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_reference_pointer_moved(
                    surface_mode.as_str(),
                    list_kind.as_str(),
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
    pane_surface_host.on_asset_reference_pointer_scrolled(
        move |surface_mode: SharedString, list_kind: SharedString, x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_reference_pointer_scrolled(
                    surface_mode.as_str(),
                    list_kind.as_str(),
                    x,
                    y,
                    delta,
                    width,
                    height,
                );
            });
        },
    );

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
