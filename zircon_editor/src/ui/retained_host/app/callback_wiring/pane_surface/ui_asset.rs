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
              item_index| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_collection_event(
                    instance_id.as_str(),
                    collection_id.as_str(),
                    event_kind.as_str(),
                    item_index,
                );
            });
        },
    );
}
