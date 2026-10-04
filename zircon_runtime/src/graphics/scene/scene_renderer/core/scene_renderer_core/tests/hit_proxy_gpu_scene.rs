#[test]
fn hit_proxy_gpu_scene_is_lazy_and_does_not_tax_normal_viewports() {
    let source = include_str!("../hit_proxy_gpu_scene.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("hit-proxy GPU scene test boundary");

    assert!(source.contains("gpu_scene: Option<GpuScene>"));
    assert!(source.contains("targets: Option<SceneHitProxyTargets>"));
    assert!(source.contains("frame_index.checked_add(1)?"));
    assert!(source.contains("get_or_insert_with"));
    assert!(source.contains("width: 1"));
    assert!(source.contains("height: 1"));
    assert!(!source.contains("impl Default for GpuScene"));
}
