use super::SceneRendererAdvancedPluginReadbacks;
use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
    RenderVirtualGeometryReadbackOutputs,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::RuntimePrepareExternalBufferBinding;

#[test]
fn advanced_plugin_readbacks_hold_neutral_plugin_renderer_outputs() {
    let outputs = RenderPluginRendererOutputs {
        virtual_geometry: RenderVirtualGeometryReadbackOutputs {
            page_table_entries: vec![1, 2, 3],
            ..RenderVirtualGeometryReadbackOutputs::default()
        },
        hybrid_gi: RenderHybridGiReadbackOutputs {
            completed_probe_ids: vec![7, 9],
            ..RenderHybridGiReadbackOutputs::default()
        },
        ..RenderPluginRendererOutputs::default()
    };

    let readbacks = SceneRendererAdvancedPluginReadbacks::from_outputs(outputs.clone());

    assert_eq!(readbacks.outputs, outputs);
    assert!(!readbacks.is_empty());
    assert!(SceneRendererAdvancedPluginReadbacks::new().is_empty());
}

#[test]
fn advanced_plugin_readbacks_hold_runtime_prepare_external_buffer_bindings() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-advanced-plugin-readbacks-external-buffer"),
        size: 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let readbacks = SceneRendererAdvancedPluginReadbacks::from_outputs_and_external_buffer_bindings(
        backend.device_profile(),
        RenderPluginRendererOutputs::default(),
        vec![RuntimePrepareExternalBufferBinding::new(
            "particles.gpu.counters",
            "particles.gpu.counters:test-runtime-prepare",
            &buffer,
        )],
    );

    assert!(!readbacks.is_empty());
    let bindings = readbacks
        .external_buffer_binding_packet()
        .expect("registered bindings must retain their device-qualified packet")
        .bindings();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].logical_name(), "particles.gpu.counters");
    assert_eq!(
        bindings[0].backing_name(),
        "particles.gpu.counters:test-runtime-prepare"
    );
}

#[test]
fn advanced_plugin_readbacks_register_through_the_product_diagnostic_router() {
    let source = include_str!("../scene_renderer_advanced_plugin_readbacks.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(source.contains("register_product_gpu_readbacks"));
    assert!(source.contains("request.register(backend)"));
    assert!(!source.contains("GpuReadbackQueue"));
    assert!(!source.contains("request_readback_external"));
}
