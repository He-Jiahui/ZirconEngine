use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

pub(super) fn wire_ui_asset_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_retained_pane_input(
        move |instance_id: SharedString, source_frame, event| {
            // Public callbacks must still refer to a genuinely visible native canvas.
            if !source_ui.retained_pane_input_source_visible(instance_id.as_str()) {
                return;
            }
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                if host
                    .builtin_template_runtime
                    .owns_generated_bottom_pane(instance_id.as_str())
                {
                    host.dispatch_generated_bottom_pane_input(
                        instance_id.as_str(),
                        &source_frame,
                        event,
                    );
                } else {
                    host.dispatch_ui_asset_preview_input(
                        instance_id.as_str(),
                        &source_frame,
                        event,
                    );
                }
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_canvas_anchor_popup_open(move |open| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.open_canvas_anchor_popup_from_native_action(&source_ui, open);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_canvas_anchor_popup_event(move |event| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_canvas_anchor_popup_event(&source_ui, event);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_ui_asset_action(
        move |instance_id: SharedString, action_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_action(instance_id.as_str(), action_id.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_ui_asset_detail_event(
        move |instance_id: SharedString,
              detail_id: SharedString,
              action_id: SharedString,
              item_index,
              primary: SharedString,
              secondary: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_detail_event(
                    instance_id.as_str(),
                    detail_id.as_str(),
                    action_id.as_str(),
                    item_index,
                    primary.as_str(),
                    secondary.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_ui_asset_collection_event(
        move |instance_id: SharedString,
              collection_id: SharedString,
              event_kind: SharedString,
              item_index,
              palette_target: Option<
            crate::ui::retained_host::host_contract::data::UiAssetPaletteInputTarget,
        >,
              value: SharedString| {
            if collection_id.as_str() == "palette" {
                let Some(target) = palette_target.as_ref() else {
                    return;
                };
                if target.instance_id() != instance_id.as_str()
                    || !source_ui.palette_search_input_is_current(target, event_kind.as_str())
                {
                    return;
                }
                let Some(owner) = weak.upgrade() else {
                    return;
                };
                let registered = {
                    let owner = owner.borrow();
                    match resolve_callback_source_window_id(&source_ui) {
                        Some(id) => owner
                            .native_window_presenters
                            .window(&id)
                            .is_some_and(|window| window.same_host_instance(&source_ui)),
                        None => owner.ui.same_host_instance(&source_ui),
                    }
                };
                if !registered {
                    return;
                }
            }
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_collection_event(
                    instance_id.as_str(),
                    collection_id.as_str(),
                    event_kind.as_str(),
                    item_index,
                    palette_target.as_ref(),
                    value.as_str(),
                );
            });
        },
    );
}
