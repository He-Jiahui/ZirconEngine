use super::super::super::data::HostWindowPresentationData;
use super::super::super::surface_hit_test::{self, TemplateNodePointerHit};

pub(in crate::ui::retained_host::host_contract::native_pointer) fn route_pointer_to_workbench_window(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    surface_hit_test::hit_test_workbench_window_template_node(presentation, x, y)
}
