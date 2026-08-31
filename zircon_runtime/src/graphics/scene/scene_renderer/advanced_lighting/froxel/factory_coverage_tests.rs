#[test]
fn froxel_pipelines_route_all_native_creates_through_pass_capability() {
    let recording =
        include_str!("../../graph_execution/render_pass_execution_context/gpu/native.rs");
    let integrate = include_str!("integrate.rs");
    let scatter = include_str!("light_scatter.rs");
    let media = include_str!("media_inject.rs");
    let integrate_executor = include_str!("executors/integrate.rs");
    let scatter_executor = include_str!("executors/light_scatter.rs");
    let media_executor = include_str!("executors/media_inject.rs");

    assert!(recording.contains("trait RenderPassGpuRecordingContext"));
    assert!(recording.contains("type ResourceFactory: RenderPassGpuResourceFactory + ?Sized"));
    assert!(recording.contains("RenderPassGpuNativeContext"));

    for (source, expected_creates) in [(integrate, 6), (scatter, 8), (media, 8)] {
        assert!(source.contains("F: RenderPassGpuResourceFactory + ?Sized"));
        assert!(source.contains("C: RenderPassGpuRecordingContext"));
        assert_eq!(source.matches(".create_").count(), expected_creates);
        assert!(!source.contains("device.create_"));
    }

    for executor in [integrate_executor, scatter_executor, media_executor] {
        assert!(executor.contains("let mut native = gpu.native_context()"));
        assert!(executor.contains("native.resource_factory()"));
        assert!(executor.contains("&mut native"));
        assert!(!executor.contains("Pipeline::new(gpu.device)"));
        assert!(!executor.contains("pipeline.encode(\n            gpu.device"));
        assert!(!executor.contains("pipeline.encode_prepared(gpu.device"));
    }
}
