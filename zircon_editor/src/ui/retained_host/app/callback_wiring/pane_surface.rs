use super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

mod assets;
mod component_showcase;
mod console;
mod hierarchy;
mod inspector;
mod pane_controls;
mod ui_asset;
mod viewport;
mod welcome;

pub(super) fn wire_pane_surface_callbacks(
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let pane_surface_host = ui.global::<PaneSurfaceHostContext>();

    welcome::wire_welcome_recent_callbacks(&pane_surface_host, host);
    hierarchy::wire_hierarchy_callbacks(&pane_surface_host, ui, host);
    console::wire_console_callbacks(&pane_surface_host, ui, host);
    inspector::wire_inspector_callbacks(&pane_surface_host, ui, host);
    pane_controls::wire_pane_control_callbacks(&pane_surface_host, ui, host);
    component_showcase::wire_component_showcase_callbacks(&pane_surface_host, ui, host);
    assets::wire_asset_callbacks(&pane_surface_host, ui, host);
    welcome::wire_welcome_control_callbacks(&pane_surface_host, host);
    viewport::wire_viewport_callbacks(&pane_surface_host, ui, host);
    ui_asset::wire_ui_asset_callbacks(&pane_surface_host, ui, host);
}
