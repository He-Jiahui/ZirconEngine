#[test]
fn shell_geometry_uses_the_declared_root_scale_mode_for_every_conversion() {
    let source = include_str!("../builder.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("test module should remain isolated from shell recompute production code");

    assert!(production.contains("ResolutionContext::from_physical_size_with_scale_mode"));
    assert!(
        production.contains("compute_workbench_shell_geometry_with_region_defaults_and_scale_mode")
    );
    assert!(production.matches("self.shell_scale_mode").count() >= 2);
}

#[test]
fn stable_shell_content_reuses_mounted_layout_frames() {
    let source = include_str!("../builder.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("builder production source");

    assert!(production.contains("requested_shell_layout_reuse"));
    assert!(production.contains("shares_mounted_layout_frames_with(&geometry)"));
    assert!(production.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(production.contains("ui.shell_content.layout_cache_hit_count"));
    assert!(production.contains("ui.shell_content.layout_cache_geometry_fallback_count"));
    assert!(production.contains("reuse_shell_layout,"));
}
