use crate::rhi::RenderQueueClass;

use crate::wgpu_backend_caps;

#[test]
fn wgpu_caps_fall_back_to_graphics_and_copy_without_rt() {
    let caps = wgpu_backend_caps("wgpu-test", wgpu::Features::empty(), true);

    assert!(caps.supports_queue(RenderQueueClass::Graphics));
    assert!(caps.supports_queue(RenderQueueClass::Copy));
    assert!(!caps.acceleration_structures.supported);
}
