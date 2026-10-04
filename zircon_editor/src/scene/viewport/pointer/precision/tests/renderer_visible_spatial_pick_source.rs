#[test]
fn renderer_visible_source_queries_only_returned_owners_at_event_time() {
    let source = include_str!("../renderer_visible_spatial_pick_source.rs")
        .split_once("#[cfg(test)]")
        .map_or(
            include_str!("../renderer_visible_spatial_pick_source.rs"),
            |(production, _)| production,
        );

    assert!(source.contains(".query_ray("));
    assert!(source.contains("renderables_by_owner.get(&owner)"));
    assert!(!source.contains("render_meshes()"));
    assert!(source.contains("Arc::clone(&self.renderables_by_owner)"));
    assert!(source.contains("collections::HashMap"));
    assert!(source.contains("Arc<HashMap<u64, ViewportRenderablePickCandidate>>"));
    assert!(
        !source.contains("BTreeMap"),
        "renderer query output already defines deterministic owner order; the event-time index must not add logarithmic ordered-map work"
    );
    assert!(source
        .contains("profile_scope!(\"editor\", \"viewport.pointer\", \"visible_spatial_query\")"));
    assert!(source.contains("visible_spatial_query_visited_node_count"));
    assert!(source.contains("visible_spatial_query_candidate_count"));
    assert!(source.contains("visible_spatial_query_hit_count"));
    assert!(source.contains("visible_spatial_query_projected_candidate_count"));
    assert!(source.contains("visible_spatial_owner_map_entry_count"));
    assert!(source.contains("visible_spatial_owner_map_candidate_copy_payload_bytes"));
    assert!(
        !source.contains("visible_spatial_projection_context_build_count"),
        "the refresh owner records zero-or-one generation construction for every sample"
    );
}

#[test]
fn renderer_visible_source_builds_projection_only_when_adopting_a_generation() {
    let source = include_str!("../renderer_visible_spatial_pick_source.rs")
        .split_once("#[cfg(test)]")
        .map_or(
            include_str!("../renderer_visible_spatial_pick_source.rs"),
            |(production, _)| production,
        );
    let (_, event_time_query) = source
        .split_once("fn candidates_at")
        .expect("renderer-visible source must expose the event-time query");

    assert!(source.contains("projection: ViewportProjectionContext"));
    assert!(
        !event_time_query.contains("ViewportProjectionContext::new"),
        "pointer events must reuse the projection captured with their generation"
    );
}
