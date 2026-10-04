use winit::dpi::PhysicalSize;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::surface_resize_changes_viewport;

#[test]
fn duplicate_surface_resize_is_a_no_op_after_minimum_size_normalization() {
    let current = ZrRuntimeViewportSizeV1::new(1, 720);

    assert!(!surface_resize_changes_viewport(
        current,
        PhysicalSize::new(0, 720),
    ));
    assert!(surface_resize_changes_viewport(
        current,
        PhysicalSize::new(2, 720),
    ));
}
