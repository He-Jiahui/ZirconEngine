use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_optional_frames;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::host_contract::workbench_context_menu::workbench_context_menu_request_for_hit;

pub(super) fn dispatch_workbench_secondary_button(
    ui: &UiHostWindow,
    hit: TemplateNodePointerHit,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    let Some(request) = workbench_context_menu_request_for_hit(&hit, x, y) else {
        return NativePointerDispatchResult::idle();
    };
    ui.global::<PaneSurfaceHostContext>()
        .invoke_workbench_context_menu_requested(request);
    let damage = union_optional_frames(cleared_text_input_frame, Some(hit.frame.clone()))
        .unwrap_or_else(|| hit.frame.clone());
    NativePointerDispatchResult::region_with_frame_update(damage)
}
