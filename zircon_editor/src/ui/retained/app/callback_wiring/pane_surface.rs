use super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

#[path = "pane_surface/assets.rs"]
mod assets;
#[path = "../pane_surface/component_showcase.rs"]
mod component_showcase;
#[path = "pane_surface/console.rs"]
mod console;
#[path = "pane_surface/hierarchy.rs"]
mod hierarchy;
#[path = "pane_surface/inspector.rs"]
mod inspector;
#[path = "pane_surface/pane_controls.rs"]
mod pane_controls;
#[path = "pane_surface/ui_asset.rs"]
mod ui_asset;
#[path = "pane_surface/viewport.rs"]
mod viewport;
#[path = "pane_surface/welcome.rs"]
mod welcome;

// 宿主启动和新浮窗安装时统一注册 pane 回调；事件由模板上下文转交 RetainedEditorHost，避免 UI 层持有业务状态。
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
