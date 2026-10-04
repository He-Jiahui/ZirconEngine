#[test]
fn render_timing_overlays_cached_scene_content_without_entering_cache_identity() {
    let session_extract = include_str!("../extract.rs");
    let extract_cache = include_str!("../extract_cache.rs");
    let cache_lookup = session_extract
        .find(".current_extract(&self.level, viewport_size)")
        .expect("session must resolve the scene extract through its cache");
    let timing_overlay = session_extract
        .find("cached.extract.set_timing(self.last_render_frame_timing)")
        .expect("session must overlay authoritative timing after cache lookup");

    assert!(cache_lookup < timing_overlay);
    assert!(!extract_cache.contains("RenderFrameTiming"));
    assert!(!extract_cache.contains("last_render_frame_timing"));
}
