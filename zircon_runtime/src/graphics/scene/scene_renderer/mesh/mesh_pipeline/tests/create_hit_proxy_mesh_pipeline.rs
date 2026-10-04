#[test]
fn hit_proxy_pipeline_owns_exact_unblended_products_and_depth_visibility() {
    let source = include_str!("../create_hit_proxy_mesh_pipeline.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source section should exist");

    assert!(source.contains("TextureFormat::R32Uint"));
    assert!(source.contains("TextureFormat::Rgba32Float"));
    assert!(source.contains("TextureFormat::Rgba16Float"));
    assert_eq!(source.matches("blend: None").count(), 3);
    assert!(source.contains("depth_write_enabled: Some(true)"));
    assert!(source.contains("CompareFunction::LessEqual"));
    assert!(source.contains("front_face: super::mesh_front_face(key)"));
    assert!(source.contains("(!key.double_sided).then_some(wgpu::Face::Back)"));
    assert!(source.contains("entry_point: Some(\"fs_main\")"));
}
