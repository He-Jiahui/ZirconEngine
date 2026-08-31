use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostDockOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::host_dock_overflow_menu::{
    contains, host_dock_overflow_popup_frame_with_state, host_dock_overflow_projection,
    host_dock_overflow_row_hit_in_popup,
};
use crate::ui::retained_host::host_contract::native_pointer::routing::{
    route_top_level_chrome, ChromePointerRoute,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn dispatch_host_dock_overflow_menu_primary_press(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: &HostDockOverflowMenuStateData,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    if !state.open {
        return None;
    }
    let Some(popup) = host_dock_overflow_popup_frame_with_state(presentation, state) else {
        ui.global::<UiHostContext>()
            .set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData::default());
        return None;
    };
    if let Some(hit) = host_dock_overflow_row_hit_in_popup(presentation, &popup, state, x, y) {
        let projection = host_dock_overflow_projection(presentation, state)?;
        let surface_key = projection.surface_key.into();
        let drawer = projection.drawer;
        ui.global::<UiHostContext>()
            .set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData::default());
        if drawer {
            ui.global::<UiHostContext>()
                .invoke_drawer_header_pointer_clicked(
                    surface_key,
                    hit.tab_index as i32,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                );
        } else {
            ui.global::<UiHostContext>()
                .invoke_document_tab_pointer_clicked(
                    surface_key,
                    hit.tab_index as i32,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                );
        }
        return Some(NativePointerDispatchResult::region_with_frame_update(
            union_extra_damage(popup, cleared_text_input_frame),
        ));
    }
    if contains(&popup, x, y) {
        return Some(match cleared_text_input_frame {
            Some(frame) => NativePointerDispatchResult::region(frame),
            None => NativePointerDispatchResult::idle(),
        });
    }
    let projection = host_dock_overflow_projection(presentation, state)?;
    if contains(&projection.anchor_frame, x, y) {
        return None;
    }
    if matches!(
        route_top_level_chrome(presentation, x, y),
        Some(ChromePointerRoute::DockOverflow { .. } | ChromePointerRoute::HostPageOverflow)
    ) {
        return None;
    }

    ui.global::<UiHostContext>()
        .set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData::default());
    Some(NativePointerDispatchResult::region_with_frame_update(
        union_extra_damage(popup, cleared_text_input_frame),
    ))
}

fn union_extra_damage(frame: FrameRect, extra: Option<FrameRect>) -> FrameRect {
    match extra {
        Some(extra) => union_frame(&frame, &extra),
        None => frame,
    }
}
