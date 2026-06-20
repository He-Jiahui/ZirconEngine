use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use super::super::super::data::{HostDragStateData, HostWindowPresentationData, TabData};
use super::super::super::globals::UiHostContext;
use super::super::super::redraw::NativePointerDispatchResult;
use super::super::super::window::UiHostWindow;
use super::super::redraw_result::tab_drag_release_redraw;
use super::super::routing::ChromePointerRoute;
use super::super::{HOST_POINTER_DOWN, HOST_POINTER_MOVE, HOST_POINTER_UP};

const TAB_DRAG_START_DISTANCE_PX: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract) fn arm_native_tab_drag(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
    x: f32,
    y: f32,
) {
    let Some((tab, source_group)) = tab_drag_payload_for_route(presentation, route) else {
        return;
    };
    ui.global::<UiHostContext>()
        .set_drag_state(HostDragStateData {
            drag_tab_id: tab.id,
            drag_tab_title: tab.title,
            drag_tab_icon_key: tab.icon_key,
            drag_source_group: source_group,
            drag_pointer_x: x,
            drag_pointer_y: y,
            ..HostDragStateData::default()
        });
}

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_tab_drag_move(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let host = ui.global::<UiHostContext>();
    let mut drag_state = host.get_drag_state();
    if drag_state.drag_tab_id.is_empty() {
        return None;
    }
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::Drag);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::Drag);

    if !drag_state.drag_active {
        let distance_x = x - drag_state.drag_pointer_x;
        let distance_y = y - drag_state.drag_pointer_y;
        if distance_x.hypot(distance_y) < TAB_DRAG_START_DISTANCE_PX {
            return Some(NativePointerDispatchResult::idle());
        }
        drag_state.drag_active = true;
        drag_state.drag_pointer_x = x;
        drag_state.drag_pointer_y = y;
        host.set_drag_state(drag_state);
        host.invoke_host_drag_pointer_event(HOST_POINTER_DOWN, x, y);
        return Some(NativePointerDispatchResult::idle());
    }

    drag_state.drag_pointer_x = x;
    drag_state.drag_pointer_y = y;
    host.set_drag_state(drag_state);
    host.invoke_host_drag_pointer_event(HOST_POINTER_MOVE, x, y);
    Some(NativePointerDispatchResult::idle())
}

pub(in crate::ui::retained_host::host_contract) fn finish_native_tab_drag(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let host = ui.global::<UiHostContext>();
    let drag_state = host.get_drag_state();
    if drag_state.drag_tab_id.is_empty() {
        return None;
    }
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::Drag);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::Drag);
    if drag_state.drag_active {
        let presentation = ui.get_host_presentation();
        host.invoke_host_drag_pointer_event(HOST_POINTER_UP, x, y);
        let release_drag_state = host.get_drag_state();
        let redraw = tab_drag_release_redraw(&presentation, &release_drag_state);
        host.set_drag_state(HostDragStateData::default());
        return Some(redraw);
    }
    host.set_drag_state(HostDragStateData::default());
    Some(NativePointerDispatchResult::idle())
}

fn tab_drag_payload_for_route(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
) -> Option<(TabData, SharedString)> {
    match route {
        ChromePointerRoute::DocumentTab {
            surface_key,
            index,
            close,
            ..
        } => {
            if *close {
                return None;
            }
            if surface_key.as_str() == "document" {
                return presentation
                    .host_scene_data
                    .document_dock
                    .tabs
                    .row_data(*index)
                    .map(|tab| {
                        (
                            tab,
                            presentation
                                .host_scene_data
                                .document_dock
                                .surface_key
                                .clone(),
                        )
                    });
            }
            for row in 0..presentation
                .host_scene_data
                .floating_layer
                .floating_windows
                .row_count()
            {
                let window = presentation
                    .host_scene_data
                    .floating_layer
                    .floating_windows
                    .row_data(row)?;
                if window.window_id.as_str() == surface_key.as_str() {
                    return window
                        .tabs
                        .row_data(*index)
                        .map(|tab| (tab, window.target_group.clone()));
                }
            }
            None
        }
        ChromePointerRoute::DrawerHeaderTab {
            surface_key, index, ..
        } => match surface_key.as_str() {
            "left" => presentation
                .host_scene_data
                .left_dock
                .tabs
                .row_data(*index)
                .map(|tab| {
                    (
                        tab,
                        presentation.host_scene_data.left_dock.surface_key.clone(),
                    )
                }),
            "right" => presentation
                .host_scene_data
                .right_dock
                .tabs
                .row_data(*index)
                .map(|tab| {
                    (
                        tab,
                        presentation.host_scene_data.right_dock.surface_key.clone(),
                    )
                }),
            "bottom" => presentation
                .host_scene_data
                .bottom_dock
                .tabs
                .row_data(*index)
                .map(|tab| {
                    (
                        tab,
                        presentation.host_scene_data.bottom_dock.surface_key.clone(),
                    )
                }),
            _ => None,
        },
        ChromePointerRoute::ActivityRail { .. }
        | ChromePointerRoute::HostPageTab { .. }
        | ChromePointerRoute::FloatingWindowHeader { .. }
        | ChromePointerRoute::Resize => None,
    }
}
