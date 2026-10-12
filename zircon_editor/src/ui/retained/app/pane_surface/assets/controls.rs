use super::*;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::PaneSurfaceHostContext;

// 资产控件事件必须带回触发它的窗口上下文，后续分发才会命中主窗或浮窗的正确 pane。
pub(super) fn wire_asset_control_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
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
}
