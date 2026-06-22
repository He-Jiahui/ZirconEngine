mod activation;
mod damage;
mod text_input;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use self::activation::dispatch_workbench_template_primary_button;
use self::text_input::dispatch_workbench_text_input_primary_button;

pub(super) fn dispatch_workbench_primary_button(
    ui: &UiHostWindow,
    hit: TemplateNodePointerHit,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    if let Some(result) =
        dispatch_workbench_text_input_primary_button(ui, &hit, cleared_text_input_frame.as_ref())
    {
        return result;
    }
    dispatch_workbench_template_primary_button(ui, hit, cleared_text_input_frame.as_ref())
}
