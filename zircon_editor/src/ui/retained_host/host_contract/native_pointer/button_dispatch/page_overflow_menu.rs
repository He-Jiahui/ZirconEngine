use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPageOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::host_page_overflow_menu::{
    host_page_overflow_popup_contains, host_page_overflow_popup_frame, host_page_overflow_row_hit,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn dispatch_host_page_overflow_menu_primary_press(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    if !presentation.host_page_overflow_menu_state.open {
        return None;
    }
    let popup = host_page_overflow_popup_frame(presentation)?;
    if let Some(hit) = host_page_overflow_row_hit(presentation, x, y) {
        ui.global::<UiHostContext>()
            .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData::default());
        ui.global::<UiHostContext>()
            .invoke_host_page_pointer_clicked(
                hit.page_index as i32,
                hit.frame.x,
                hit.frame.width,
                x - hit.frame.x,
                y - hit.frame.y,
            );
        return Some(NativePointerDispatchResult::region_with_frame_update(
            union_extra_damage(popup, cleared_text_input_frame),
        ));
    }

    if host_page_overflow_popup_contains(presentation, x, y) {
        return Some(NativePointerDispatchResult::region(popup));
    }
    if contains(
        &presentation.host_scene_data.page_chrome.overflow_frame,
        x,
        y,
    ) {
        return None;
    }

    ui.global::<UiHostContext>()
        .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData::default());
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

fn contains(frame: &FrameRect, x: f32, y: f32) -> bool {
    x >= frame.x && y >= frame.y && x <= frame.x + frame.width && y <= frame.y + frame.height
}
