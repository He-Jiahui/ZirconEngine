use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;

use super::super::release::dispatch_workbench_release_button;

pub(super) fn dispatch_released_workbench_button(
    hit: TemplateNodePointerHit,
) -> NativePointerDispatchResult {
    dispatch_workbench_release_button(hit)
}
