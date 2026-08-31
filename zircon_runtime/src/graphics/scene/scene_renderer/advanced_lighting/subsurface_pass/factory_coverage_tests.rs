#[test]
fn sss_shared_bundle_routes_all_native_creates_through_pass_capability() {
    let pipelines = include_str!("pipelines.rs");
    let executors = include_str!("executors.rs");

    assert!(pipelines.contains("RenderPassGpuRecordingContext"));
    assert!(pipelines.contains("RenderPassGpuResourceFactory"));
    assert!(pipelines.contains("F: RenderPassGpuResourceFactory + ?Sized"));
    assert!(pipelines.contains("C: RenderPassGpuRecordingContext"));
    assert_eq!(pipelines.matches(".create_").count(), 12);
    assert!(!pipelines.contains("device.create_"));

    assert!(executors.contains("C: RenderPassGpuRecordingContext"));
    assert_eq!(
        executors
            .matches("let mut native = gpu.native_context()")
            .count(),
        3
    );
    assert!(executors.contains("context.resource_factory()"));
    assert!(!executors.contains("SubsurfacePipelines::new(device"));
    assert_eq!(executors.matches("gpu.device").count(), 3);
    assert_eq!(executors.matches("gpu.device_epoch()").count(), 3);
    assert!(executors.contains("fn buffer<'a>("));
    assert!(executors.contains("Result<wgpu::BufferBinding<'a>, String>"));
    let buffer_helper = &executors[executors
        .find("fn buffer<'a>(")
        .expect("SSS buffer helper must retain the graph-resource lifetime")..];
    assert!(buffer_helper.contains("'a,"));
    assert!(!buffer_helper.contains(".cloned()"));
}
