#[test]
fn renderer_snapshot_adoption_is_generation_bound_and_replaces_static_renderable_nodes() {
    let source = include_str!("../viewport_overlay_pointer_router_visible_spatial_query.rs");

    assert!(source.contains("snapshot.identity().world.raw() == world_generation"));
    assert!(source.contains("Vec::new().into()"));
    assert!(source.contains("RendererVisibleSpatialPickSource::new"));
    assert!(source.contains("current.with_snapshot"));
    let scene_sync = include_str!("../viewport_overlay_pointer_router_sync.rs");
    assert!(scene_sync.contains("self.renderer_visible_spatial_snapshot = None;"));
}

#[test]
fn unchanged_renderer_generation_reuses_its_projection_source() {
    let source = include_str!("../viewport_overlay_pointer_router_visible_spatial_query.rs")
        .split_once("#[cfg(test)]")
        .map_or(
            include_str!("../viewport_overlay_pointer_router_visible_spatial_query.rs"),
            |(production, _)| production,
        );

    assert!(source.contains("current.is_current_for"));
    assert!(source.contains("current.clone()"));
    assert!(source.contains("visible_spatial_source_reuse_count"));
    assert!(source.contains("visible_spatial_projection_context_build_count"));
    assert!(source.contains("let (source, source_changed, source_reused)"));
    assert!(source.contains("if source_changed && source.is_some()"));
    assert!(source.contains("current.with_snapshot"));
    assert!(source.contains("if source_changed {"));
}
