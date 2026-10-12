use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;

pub(super) fn dispatch_workbench_release_button(
    hit: TemplateNodePointerHit,
) -> NativePointerDispatchResult {
    NativePointerDispatchResult::region(hit.frame)
}
