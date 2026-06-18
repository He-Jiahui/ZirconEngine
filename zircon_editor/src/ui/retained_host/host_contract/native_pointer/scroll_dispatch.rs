use crate::ui::retained_host::host_contract::globals::{PaneSurfaceHostContext, UiHostContext};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use super::menu_geometry::{menu_damage_frame, menu_handles_point, menu_popup_handles_point};
use super::routing::{route_pointer_to_pane, PanePointerTarget};
use super::{VIEWPORT_POINTER_BUTTON_NONE, VIEWPORT_POINTER_SCROLL};

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_scroll(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
    delta: f32,
) -> NativePointerDispatchResult {
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::IdleHover);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::IdleHover);

    let presentation = ui.get_host_presentation();
    if menu_handles_point(&presentation, x, y) || menu_popup_handles_point(&presentation, x, y) {
        ui.global::<UiHostContext>()
            .invoke_menu_pointer_scrolled(x, y, delta);
        return NativePointerDispatchResult::region(menu_damage_frame(&presentation));
    }

    if let Some(pointer) = route_pointer_to_pane(&presentation, x, y) {
        let damage_frame = pointer.frame.clone();
        let pane_host = ui.global::<PaneSurfaceHostContext>();
        match pointer.target {
            PanePointerTarget::Hierarchy => pane_host.invoke_hierarchy_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Welcome => pane_host.invoke_welcome_recent_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Console => pane_host.invoke_console_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Inspector => pane_host.invoke_inspector_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::BrowserAssetDetails => pane_host
                .invoke_browser_asset_details_pointer_scrolled(
                    pointer.local_x,
                    pointer.local_y,
                    delta,
                    pointer.width,
                    pointer.height,
                ),
            PanePointerTarget::AssetTree(mode) => pane_host.invoke_asset_tree_pointer_scrolled(
                mode,
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetContent(mode) => pane_host
                .invoke_asset_content_pointer_scrolled(
                    mode,
                    pointer.local_x,
                    pointer.local_y,
                    delta,
                    pointer.width,
                    pointer.height,
                ),
            PanePointerTarget::AssetReference(mode, list_kind) => pane_host
                .invoke_asset_reference_pointer_scrolled(
                    mode,
                    list_kind,
                    pointer.local_x,
                    pointer.local_y,
                    delta,
                    pointer.width,
                    pointer.height,
                ),
            PanePointerTarget::Viewport(_) => {
                pane_host.invoke_viewport_pointer_event(
                    VIEWPORT_POINTER_SCROLL,
                    VIEWPORT_POINTER_BUTTON_NONE,
                    pointer.local_x,
                    pointer.local_y,
                    delta,
                );
                return NativePointerDispatchResult::idle();
            }
            PanePointerTarget::TemplateNode(_)
            | PanePointerTarget::ViewportToolbar(_)
            | PanePointerTarget::UiAsset
            | PanePointerTarget::Other => {}
        }
        return NativePointerDispatchResult::region(damage_frame);
    }

    NativePointerDispatchResult::idle()
}
