use super::{bloom_target_format, POST_PROCESS_INTERMEDIATE_HDR_FORMAT};

#[test]
fn bloom_pipeline_targets_intermediate_hdr_resource_format() {
    assert_eq!(bloom_target_format(), POST_PROCESS_INTERMEDIATE_HDR_FORMAT);
    assert_ne!(bloom_target_format(), wgpu::TextureFormat::Rgba8UnormSrgb);
}

#[test]
fn split_post_pipelines_share_one_shader_module() {
    let build_source = include_str!("../build.rs");
    let split_pipeline_sources = [
        include_str!("../blur_pipeline.rs"),
        include_str!("../depth_of_field_pipeline.rs"),
        include_str!("../motion_blur_pipeline.rs"),
        include_str!("../post_process_pipeline.rs"),
        include_str!("../scene_composite_pipeline.rs"),
        include_str!("../screen_space_reflection_reflection_pyramid_pipeline.rs"),
        include_str!("../screen_space_reflection_reflection_pyramid_coarse_pipeline.rs"),
        include_str!("../screen_space_reflection_resolve_pipeline.rs"),
        include_str!("../screen_space_reflection_specular_occlusion_pipeline.rs"),
    ];
    let module_creation = ["device.create_shader_", "module"].concat();
    let layout_creation = ["device.create_pipeline_", "layout"].concat();
    let shared_module_binding =
        ["let post_process_shader = device.create_shader_", "module"].concat();
    let shared_layout_binding = [
        "let post_process_pipeline_layout = device.create_pipeline_",
        "layout",
    ]
    .concat();

    assert!(build_source.contains(&shared_module_binding));
    assert!(build_source.contains("&post_process_shader"));
    assert!(build_source.contains(&shared_layout_binding));
    assert!(build_source.contains("&post_process_pipeline_layout"));
    for source in split_pipeline_sources {
        assert!(source.contains("shader: &wgpu::ShaderModule"));
        assert!(!source.contains(&module_creation));
        assert!(source.contains("pipeline_layout: &wgpu::PipelineLayout"));
        assert!(!source.contains(&layout_creation));
    }
}
