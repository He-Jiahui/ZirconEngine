use super::super::super::data::HostPresentationGeneration;
use super::super::super::surface_hit_test::{
    self, TemplateNodePointerHit, TemplateNodePointerMoveHit,
};

pub(in crate::ui::retained_host::host_contract) fn route_pointer_move_to_workbench_generation(
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerMoveHit<'_>> {
    surface_hit_test::hit_test_workbench_window_template_node_for_pointer_move_with_index(
        generation.structure(),
        generation.workbench_hit_index(),
        x,
        y,
    )
}

pub(in crate::ui::retained_host::host_contract) fn route_pointer_to_workbench_generation(
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    surface_hit_test::hit_test_workbench_window_template_node_with_index(
        generation.structure(),
        generation.workbench_hit_index(),
        x,
        y,
    )
}
