use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;

use super::validate_post_process_construction;

#[test]
fn post_process_construction_validation_scope_reports_its_operation() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };

    let result = validate_post_process_construction(&backend.device, "test-invalid-shader", || {
        backend
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-post-process-invalid-test-shader"),
                source: wgpu::ShaderSource::Wgsl("not valid WGSL".into()),
            })
    });

    assert!(matches!(
        result,
        Err(GraphicsError::WgpuValidation(message))
            if message.contains("post-process test-invalid-shader construction")
    ));
}
